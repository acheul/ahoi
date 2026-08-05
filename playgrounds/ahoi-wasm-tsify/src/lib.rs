//! Tsify variant of the playground wasm crate (`ahoi-wasm` uses ts-rs).
//!
//! Exists to verify the "converter-agnostic" claim of the type-export story:
//! any exporter can supply the key/data types, and ahoi only contributes the
//! ret maps. Division of labour here:
//!
//! - key/data **types** → [Tsify](https://github.com/madonoharu/tsify),
//!   declaration-only (`#[derive(Tsify)]`): the declarations are embedded in
//!   the wasm-pack generated `pkg/*.d.ts`, so there is no separate bindings
//!   step for them
//! - key **return types** → ahoi's `#[derive(Rets)]` + `#[ret(..)]`,
//!   exported to `bindings/Rets.ts` by the `generate` test (same as ever)
//! - **values** on the wire → `SerdeWasmBindgenConverter`, untouched: the
//!   bridge ABI passes `JsValue`s, so Tsify's wasm-abi machinery is not used
//!
//! The keys are a subset of `ahoi-wasm`'s — just enough to cover the TS
//! declaration shapes an exporter must get right:
//!
//! | key                | declaration shape it verifies                  |
//! |--------------------|------------------------------------------------|
//! | `Hail::Count`      | unit variant (string literal), writable hail   |
//! | `Hail::Doubled`    | unit variant, read-only hail over a memo       |
//! | `Hail::Item(i)`    | tuple variant (`{ Item: number }`)             |
//! | `Hail::LastFruit`  | data enum in a ret (`Fruit | undefined`)       |
//! | `Tell::Increase`   | tell with a return value                       |
//! | `Tell::PushItem`   | tuple variant tell, no return value            |
//! | `Tell::SetFruit`   | data enum as argument (JS → Rust)              |

use ahoi::js_bridge::SerdeWasmBindgenConverter as Converter;
use ahoi::js_bridge::*;
use ahoi::*;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// run this test to (re)generate the ret maps for the JS side.
// `Fruit` is imported from the wasm pkg's `.d.ts`, where Tsify put it —
// there is no `bindings/Fruit.ts` in this setup.
#[test]
fn generate() {
    ahoi::js_bridge::TsFile::new()
        .import("Fruit", "../pkg/ahoi_wasm_tsify")
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
}

#[derive(Tsify, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Fruit {
    Apple,
    Banana(String),
}

// ── Pier ───────────────────────────────────────────────────────────────────

#[derive(Tsify, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pier {
    Top,
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
            });
            provide_context(state);
        }
    }
}

// ── Hail ───────────────────────────────────────────────────────────────────

#[derive(Rets, Tsify, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hail {
    #[ret(i32)]
    Count,
    #[ret(i32)]
    Doubled,
    #[ret(Vec<i32>)]
    Items,
    #[ret(Option<i32>)]
    Item(usize),
    #[ret(Option<Fruit>)]
    LastFruit,
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
        Hail::LastFruit => state().last_fruit().set_hail::<Converter>(),
    }
}

// ── Tell ───────────────────────────────────────────────────────────────────

#[derive(Rets, Tsify, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tell {
    #[ret(i32)]
    Increase,
    PushItem(i32),
    #[ret(Option<i32>)]
    PopItem,
    #[ret(Fruit)]
    SetFruit(Fruit),
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
            *state.last_fruit().write() = Some(fruit.clone());
            serde_wasm_bindgen::to_value(&fruit).unwrap()
        }
    }
}
