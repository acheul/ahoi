//! Playground wasm crate: exercises the whole bridge without Tsain.
//!
//! Division of labour:
//! - key/data **types** → any exporter you like; here: ts-rs (`#[derive(TS)]`,
//!   exported to `bindings/` by the `export_bindings_*` tests)
//! - key **return types** → ahoi's `#[derive(Rets)]` + `#[ret(..)]`,
//!   exported to `bindings/Keys.ts` by the `generate` test
//! - **values** on the wire → `SerdeWasmBindgenConverter`

use ahoi::js_bridge::SerdeWasmBindgenConverter as Converter;
use ahoi::js_bridge::*;
use ahoi::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
    gloo_console::log!("DEV MODE: set_panic_hook")
}

// run this test to (re)generate the ret maps for the JS side
#[test]
fn generate() {
    ahoi::ts::TsFile::new()
        .import("Fruit", "./Fruit")
        .with::<Hail>()
        .with::<Tell>()
        .export("./bindings/Keys.ts");
}

// data
#[derive(Stock, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub count: i32,
    pub items: Vec<i32>,
    pub fruits: HashMap<String, Fruit>,
}

#[derive(Stock, TS, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ts(export)]
pub enum Fruit {
    Apple,
    Banana(String),
}

#[derive(Clone, Copy)]
struct FruitsContext(Stock<HashMap<String, Fruit>>);

#[derive(Clone, Copy)]
struct FruitsEven(Stock<bool>);

#[derive(Clone, Copy)]
struct Info(Stock<String>);

#[derive(Clone, Copy)]
struct AddCountCb(Callback<i32, ()>);

#[derive(Clone, Copy)]
struct AddCountAction(Action<i32, ()>);

#[derive(Clone, Copy)]
struct CountResource(Resource<i32>);

/// Pier Key
#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export)]
pub enum Pier {
    Top,
    Comp,
    About,
}

wasm_bindgen_enrol_sphere!(@pier, Pier, run_pier, Converter);

fn run_pier(key: Pier) -> () {
    match key {
        Pier::Top => {
            // Set Local Hail Dispatcher
            let _ = set_js_hail_dispatcher();

            // Provide State
            let state = Stock::new(State {
                count: 0,
                items: vec![10i32],
                fruits: Default::default(),
            });
            provide_context(state);

            // provide fruits_even (test effect reactivity)
            let fruits: Stock<_> = state.fruits().into();
            let _ = provide_context(FruitsContext(fruits));

            // (test of effect)
            let fruits_even: Stock<bool> = Stock::new(fruits.peek().len() % 2 == 0);

            let _ = Effect::new(move || {
                let len = fruits.read().len();
                let is_even = len % 2 == 0;
                gloo_console::log!(&format!("effect len: {}", len));
                *fruits_even.write() = is_even;
            });
            let _ = provide_context(FruitsEven(fruits_even));
        }
        Pier::Comp => {
            let info = Stock::new(String::from("Component"));
            provide_context(Info(info));

            let state = use_context::<Stock<State>>().unwrap();

            // Callback: count 에 n 을 더하는 sync 콜백
            let add_count = Callback::new(move |n: i32| {
                *state.count().write() += n;
            });
            provide_context(AddCountCb(add_count));

            // Action: 1_000ms 마다 count 에 n 더하는 async Action
            let action: Action<i32, ()> = Action::new(move |x| async move {
                loop {
                    gloo_timers::future::TimeoutFuture::new(1_000).await;
                    let mut count = state.count().write();
                    *count += x;
                }
            });
            provide_context(AddCountAction(action));

            // Resource: count 가 바뀔 때마다 count * 10 을 자동 fetch (1_000ms latency 부여)
            let count_res: Resource<i32> = Resource::new(move || async move {
                gloo_timers::future::TimeoutFuture::new(1_000).await;
                let c = state.count().read();
                *c * 10
            });
            provide_context(CountResource(count_res));
        }
        Pier::About => {
            let info: Stock<String> = Stock::new(String::from("information"));
            provide_context(Info(info));
        }
    }
}

/// Hail Key
#[derive(Rets, TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(Vec<i32>)]
    Items,
    #[ret(Option<i32>)]
    Item(usize),
    #[ret(usize)]
    ItemLength,
    #[ret(Vec<(String, Fruit)>)]
    Fruits,
    #[ret(Option<Fruit>)]
    Fruit(String),
    #[ret(bool)]
    FruitsEven,
    #[ret(String)]
    CompInfo,
    #[ret(String)]
    AboutInfo,
    #[ret(Option<i32>)]
    CountResourceValue,
}

wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);

fn run_hail(key: Hail) -> JsValue {
    match key {
        Hail::Count => {
            // use state
            let state = use_context::<Stock<State>>().unwrap();
            state.count().set_hail::<Converter>()
        }
        Hail::Items => {
            // use state
            let state = use_context::<Stock<State>>().unwrap();
            // derive items
            let items = state.items();
            // set bridge
            items.set_hail::<Converter>()
        }
        Hail::Item(index) => {
            // use state
            let state = use_context::<Stock<State>>().unwrap();
            // derive items
            let items = state.items();
            // derive item
            let item = items.get(index);
            // set bridge
            item.set_hail::<Converter>()
        }
        Hail::ItemLength => {
            // use state
            let state = use_context::<Stock<State>>().unwrap();
            // derive items
            let items = state.items();
            // memo length
            let length = items.memo(|e| e.len());
            // set bridge
            length.set_read_hail::<Converter>()
        }
        Hail::Fruits => {
            // use fruits
            let FruitsContext(fruits) = use_context::<FruitsContext>().unwrap();

            // memo
            let fruits = fruits.memo(|e| {
                e.iter()
                    .map(|(a, b)| (a.clone(), b.clone()))
                    .collect::<Vec<(String, Fruit)>>()
            });
            // set bridge
            fruits.set_read_hail::<Converter>()
        }
        Hail::Fruit(name) => {
            // use fruits
            let FruitsContext(fruits) = use_context::<FruitsContext>().unwrap();
            // derive fruit
            let fruit = fruits.get(name);
            // set bridge
            fruit.set_hail::<Converter>()
        }
        Hail::FruitsEven => {
            // use fruits_even
            let FruitsEven(fruits_even) = use_context::<FruitsEven>().unwrap();
            // set bridge
            fruits_even.set_hail::<Converter>()
        }
        Hail::CompInfo => {
            let Info(info) = use_context::<Info>().unwrap();
            info.set_hail::<Converter>()
        }
        Hail::AboutInfo => {
            let Info(info) = use_context::<Info>().unwrap();
            info.set_hail::<Converter>()
        }
        Hail::CountResourceValue => {
            let CountResource(resource) = use_context::<CountResource>().unwrap();
            resource.set_read_hail::<Converter>()
        }
    }
}

/// Tell
#[derive(Rets, TS, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ts(export)]
pub enum Tell {
    #[ret(usize)]
    IncreaseCount,
    InsertItem(usize, i32),
    #[ret(bool)]
    PopItem,
    #[ret(bool)]
    InsertFruit(String, Fruit),
    SetCompInfo(String),
    SetAboutInfo(String),
    AddCount(i32),
    CallAddCountAction(i32),
    CancelAddCountAction,
    ExSpawnBatch,
}

wasm_bindgen_tell!(Tell, run_tell, Converter);

fn run_tell(tell: Tell) -> JsValue {
    match tell {
        Tell::IncreaseCount => {
            let count = {
                // js<->rs 비용에 비하면 매번 use context 부르는 비용은 negligible
                let state = use_context::<Stock<State>>().unwrap();
                let mut count = state.count().write();
                *count += 1;
                *count
            };
            serde_wasm_bindgen::to_value(&count).unwrap()
        }
        Tell::InsertItem(index, v) => {
            {
                let state = use_context::<Stock<State>>().unwrap();
                state.items().write().insert(index, v);
            };
            JsValue::undefined()
        }

        Tell::PopItem => {
            let state = use_context::<Stock<State>>().unwrap();
            let popped = state.items().write().pop().is_some();
            serde_wasm_bindgen::to_value(&popped).unwrap()
        }

        Tell::InsertFruit(name, fruit) => {
            // use fruits
            let FruitsContext(fruits) = use_context::<FruitsContext>().unwrap();
            let is_new = fruits.write().insert(name, fruit).is_none();
            serde_wasm_bindgen::to_value(&is_new).unwrap()
        }
        Tell::SetCompInfo(new_info) => {
            let Info(info) = use_context::<Info>().unwrap();
            *info.write() = new_info;
            JsValue::undefined()
        }
        Tell::SetAboutInfo(new_info) => {
            let Info(info) = use_context::<Info>().unwrap();
            *info.write() = new_info;
            JsValue::undefined()
        }
        Tell::AddCount(n) => {
            let AddCountCb(cb) = use_context::<AddCountCb>().unwrap();
            cb.call(n);
            JsValue::undefined()
        }
        Tell::CallAddCountAction(n) => {
            let AddCountAction(action) = use_context::<AddCountAction>().unwrap();
            let _ = action.call(n);
            JsValue::undefined()
        }
        Tell::CancelAddCountAction => {
            let AddCountAction(action) = use_context::<AddCountAction>().unwrap();
            let _ = action.cancel();
            JsValue::undefined()
        }
        Tell::ExSpawnBatch => {
            // Fire-and-forget: detach so the task survives this synchronous handler
            // returning (otherwise the handle drops and aborts it before it runs).
            ahoi::spawn_batch(async move {
                let sphere_id = ahoi::current_sphere_id();
                gloo_console::log!(&format!("Msg In spawn_batch: {:?}", sphere_id));
            })
            .detach();
            let sphere_id = ahoi::current_sphere_id();
            gloo_console::log!(&format!("Msg Out Of spawn_batch: {:?}", sphere_id));
            JsValue::undefined()
        }
    }
}
