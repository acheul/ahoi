//! The Rust side of every live example in the book.
//!
//! One crate, one wasm module, shared by all demo pages.
//!
//! The `// #region <name>` markers are not decoration: `<Example name="..." />`
//! slices this file by those names, so what a page shows is literally what
//! compiles here. Add a demo by adding a region, never by pasting Rust into a
//! markdown file.

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

// #region counter
#[derive(Stock, Serialize, Deserialize)]
pub struct State {
    count: i32,
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
            provide_context(Stock::new(State { count: 0 }));
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
}

wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);

fn run_hail(key: Hail) -> JsValue {
    let state = use_context::<Stock<State>>().unwrap();
    match key {
        // read-write: JS can write back into the stock
        Hail::Count => state.count().set_hail::<Converter>(),
        // read-only, and recomputed only when `count` actually changes
        Hail::Doubled => state.count().memo(|c| *c * 2).set_read_hail::<Converter>(),
    }
}

#[derive(Rets, TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export)]
pub enum Tell {
    #[ret(i32)]
    Increase,
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
    }
}
// #endregion counter
