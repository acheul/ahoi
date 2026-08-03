//! Shared wasm example crate for the framework playgrounds (solid, react, ...).
//!
//! Division of labour:
//! - key/data **types** → any exporter you like; here: ts-rs (`#[derive(TS)]`,
//!   exported to `bindings/` by the `export_bindings_*` tests)
//! - key **return types** → ahoi's `#[derive(Rets)]` + `#[ret(..)]`,
//!   exported to `bindings/Rets.ts` by the `generate` test
//! - **values** on the wire → `SerdeWasmBindgenConverter`
//!
//! The keys are kept small but each one verifies a distinct bridge feature:
//!
//! | key                 | verifies                                        |
//! |---------------------|-------------------------------------------------|
//! | `Hail::Count`       | writable hail (JS ⇄ Rust round-trip)            |
//! | `Hail::Doubled`     | read-only hail over a memo                      |
//! | `Hail::Item(i)`     | *writable* path-derived hail (selective         |
//! |                     | propagation + write into a derived path)        |
//! | `Hail::CountX10`    | async resource → hail dispatch from a batch     |
//! | `Hail::LastFruit`   | enum value on the wire (externally tagged)      |
//! | `Hail::FruitCounts` | map value on the wire (JS `Map`)                |
//! | `Hail::PanelInfo`   | context under a nested pier + sphere cleanup    |
//! | `Tell::Increase`    | tell with a return value                        |
//! | `Tell::PushItem`    | tell without a return value                     |
//! | `Tell::SetFruit`    | enum argument (JS → Rust)                       |
//! | `Tell::AddCount`    | context-stored sync `Callback`                  |
//! | `Tell::StartTicker` | async `Action` mutating state over time         |
//! | `Tell::StopTicker`  | cancelling a running `Action`                   |

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
    ahoi::js_bridge::TsFile::new()
        .import("Fruit", "./Fruit")
        .with::<Hail>()
        .with::<Tell>()
        .export("./bindings/Rets.ts");
}

// ── data ───────────────────────────────────────────────────────────────────

#[derive(Stock, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub count: i32,
    pub items: Vec<i32>,
    pub last_fruit: Option<Fruit>,
    pub fruit_counts: HashMap<String, u32>,
}

#[derive(TS, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ts(export)]
pub enum Fruit {
    Apple,
    Banana(String),
}

impl Fruit {
    fn name(&self) -> &'static str {
        match self {
            Fruit::Apple => "Apple",
            Fruit::Banana(_) => "Banana",
        }
    }
}

/// Context provided by the `Panel` pier only.
#[derive(Clone, Copy)]
struct PanelInfo(Stock<String>);

/// Sync callback stored in context: adds `n` to the count.
#[derive(Clone, Copy)]
struct AddCount(Callback<i32, ()>);

/// Async action stored in context: adds `x` to the count every second until
/// cancelled.
#[derive(Clone, Copy)]
struct Ticker(Action<i32, ()>);

// ── Pier ───────────────────────────────────────────────────────────────────

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export)]
pub enum Pier {
    Top,
    Panel,
}

wasm_bindgen_enrol_sphere!(@pier, Pier, run_pier, Converter);

fn run_pier(key: Pier) {
    match key {
        Pier::Top => {
            set_js_hail_dispatcher();

            let state = Stock::new(State {
                count: 0,
                items: vec![10],
                last_fruit: None,
                fruit_counts: Default::default(),
            });
            provide_context(state);

            let add_count = Callback::new(move |n: i32| {
                *state.count().write() += n;
            });
            provide_context(AddCount(add_count));

            let ticker: Action<i32, ()> = Action::new(move |x| async move {
                loop {
                    gloo_timers::future::TimeoutFuture::new(1_000).await;
                    *state.count().write() += x;
                }
            });
            provide_context(Ticker(ticker));
        }
        Pier::Panel => {
            let info = Stock::new(String::from("hello from panel"));
            provide_context(PanelInfo(info));
        }
    }
}

// ── Hail ───────────────────────────────────────────────────────────────────

#[derive(Rets, TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(i32)]
    Doubled,
    #[ret(Vec<i32>)]
    Items,
    #[ret(Option<i32>)]
    Item(usize),
    #[ret(Option<i32>)]
    CountX10,
    #[ret(Option<Fruit>)]
    LastFruit,
    #[ret(HashMap<String, u32>)]
    FruitCounts,
    #[ret(String)]
    PanelInfo,
}

wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);

fn run_hail(key: Hail) -> JsValue {
    let state = || use_context::<Stock<State>>().unwrap();
    match key {
        Hail::Count => state().count().set_hail::<Converter>(),
        Hail::Doubled => {
            let doubled = state().count().memo(|c| *c * 2);
            doubled.set_read_hail::<Converter>()
        }
        Hail::Items => state().items().set_hail::<Converter>(),
        Hail::Item(index) => state().items().get(index).set_hail::<Converter>(),
        Hail::CountX10 => {
            let state = state();
            let resource: Resource<i32> = Resource::new(move || async move {
                gloo_timers::future::TimeoutFuture::new(500).await;
                let count = state.count().read();
                *count * 10
            });
            resource.set_read_hail::<Converter>()
        }
        Hail::LastFruit => state().last_fruit().set_hail::<Converter>(),
        Hail::FruitCounts => state().fruit_counts().set_hail::<Converter>(),
        Hail::PanelInfo => {
            let PanelInfo(info) = use_context::<PanelInfo>().unwrap();
            info.set_hail::<Converter>()
        }
    }
}

// ── Tell ───────────────────────────────────────────────────────────────────

#[derive(Rets, TS, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ts(export)]
pub enum Tell {
    #[ret(i32)]
    Increase,
    PushItem(i32),
    #[ret(Option<i32>)]
    PopItem,
    #[ret(u32)]
    SetFruit(Fruit),
    AddCount(i32),
    StartTicker(i32),
    StopTicker,
}

wasm_bindgen_tell!(Tell, run_tell, Converter);

fn run_tell(tell: Tell) -> JsValue {
    let state = use_context::<Stock<State>>().unwrap();
    match tell {
        Tell::Increase => {
            let new_count = {
                let mut count = state.count().write();
                *count += 1;
                *count
            };
            serde_wasm_bindgen::to_value(&new_count).unwrap()
        }
        Tell::PushItem(v) => {
            state.items().write().push(v);
            JsValue::undefined()
        }
        Tell::PopItem => {
            let popped = state.items().write().pop();
            serde_wasm_bindgen::to_value(&popped).unwrap()
        }
        Tell::SetFruit(fruit) => {
            let name = fruit.name().to_string();
            *state.last_fruit().write() = Some(fruit);
            let new_count = {
                let mut counts = state.fruit_counts().write();
                let n = counts.entry(name).or_insert(0);
                *n += 1;
                *n
            };
            serde_wasm_bindgen::to_value(&new_count).unwrap()
        }
        Tell::AddCount(n) => {
            let AddCount(callback) = use_context::<AddCount>().unwrap();
            callback.call(n);
            JsValue::undefined()
        }
        Tell::StartTicker(x) => {
            let Ticker(ticker) = use_context::<Ticker>().unwrap();
            let _ = ticker.call(x);
            JsValue::undefined()
        }
        Tell::StopTicker => {
            let Ticker(ticker) = use_context::<Ticker>().unwrap();
            let _ = ticker.cancel();
            JsValue::undefined()
        }
    }
}
