//! The Rust side of every live example in the book.
//!
//! One crate, one wasm module, shared by all demo pages.
//!
//! The `// #region <name>` markers are not decoration: `<Example name="..." />`
//! slices this file by those names, so what a page shows is literally what
//! compiles here. Add a demo by adding keys, never by pasting Rust into a
//! markdown file.
//!
//! The key enums and their runners are single shared containers, so there is
//! one region covering the whole file rather than one per demo.

use ahoi::js_bridge::SerdeWasmBindgenConverter as Converter;
use ahoi::js_bridge::*;
use ahoi::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Regenerates the ret maps for the JS side. Run with `cargo test`.
#[test]
fn generate() {
    ahoi::js_bridge::TsFile::new()
        .with::<Hail>()
        .with::<Tell>()
        .export("./bindings/Rets.ts");
}

// #region all
#[derive(Stock, Serialize, Deserialize)]
pub struct State {
    count: i32,
    items: Vec<i32>,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export)]
pub enum Pier {
    Top,
}

wasm_bindgen_enrol_sphere!(@pier, Pier, run_pier, Converter);

fn run_pier(key: Pier) {
    match key {
        Pier::Top => {
            set_js_hail_dispatcher();
            provide_context(Stock::new(State {
                count: 0,
                items: vec![10, 20],
            }));
        }
    }
}

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
}

wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);

fn run_hail(key: Hail) -> JsValue {
    let state = use_context::<Stock<State>>().unwrap();
    match key {
        // read-write: JS can write back into the stock
        Hail::Count => state.count().set_hail::<Converter>(),
        // read-only, and recomputed only when `count` actually changes
        Hail::Doubled => state.count().memo(|c| *c * 2).set_read_hail::<Converter>(),
        Hail::Items => state.items().set_read_hail::<Converter>(),
        // path-derived and writable; absent indexes arrive as `undefined`
        Hail::Item(index) => state.items().get(index).set_hail::<Converter>(),
    }
}

#[derive(Rets, TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export)]
pub enum Tell {
    #[ret(i32)]
    Increase,
    // no `#[ret]` — returns undefined
    PushItem(i32),
    #[ret(Option<i32>)]
    PopItem,
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
        Tell::PushItem(value) => {
            state.items().write().push(value);
            JsValue::undefined()
        }
        Tell::PopItem => {
            let popped = state.items().write().pop();
            serde_wasm_bindgen::to_value(&popped).unwrap()
        }
    }
}
// #endregion all
