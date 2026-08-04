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
    /// How many times the `Label` memo's body has actually run.
    ///
    /// Reactive on purpose: it has to travel in the same dispatch as everything
    /// else, so the JS side sees it settle together with the value it explains.
    label_runs: u32,
}

/// An async value derived from `count`, created once per pier so both the value
/// hail and the loading hail observe the same resource.
#[derive(Clone, Copy)]
struct TenTimes(Resource<i32>);

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
            let state = Stock::new(State {
                count: 0,
                items: vec![10, 20],
                label_runs: 0,
            });
            provide_context(state);

            // Reads `count` inside the async block on purpose: a resource
            // re-enters its tracking context on every poll, so this still
            // registers as a dependency and drives the refetch.
            let ten_times = Resource::new(move || async move {
                gloo_timers::future::TimeoutFuture::new(600).await;
                *state.count().read() * 10
            });
            provide_context(TenTimes(ten_times));
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
    #[ret(i32)]
    Parity,
    #[ret(String)]
    Label,
    #[ret(u32)]
    LabelRuns,
    /// `undefined` until the first fetch lands.
    #[ret(Option<i32>)]
    TenTimes,
    #[ret(bool)]
    TenTimesLoading,
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
        Hail::Parity => state.count().memo(|c| *c % 2).set_read_hail::<Converter>(),
        // Chained on top of `Parity`: it only re-runs when the parity value
        // itself changes, which is what the demo makes visible.
        Hail::Label => {
            let parity = state.count().memo(|c| *c % 2);
            Memo::new(move || {
                *state.label_runs().write() += 1;
                let label = if *parity.read() == 0 { "even" } else { "odd" };
                label.to_string()
            })
            .set_read_hail::<Converter>()
        }
        Hail::LabelRuns => state.label_runs().set_read_hail::<Converter>(),
        Hail::TenTimes => {
            let TenTimes(resource) = use_context::<TenTimes>().unwrap();
            resource.set_read_hail::<Converter>()
        }
        Hail::TenTimesLoading => {
            let TenTimes(resource) = use_context::<TenTimes>().unwrap();
            Memo::new(move || resource.pending()).set_read_hail::<Converter>()
        }
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
    /// Adds `n` to the count.
    Bump(i32),
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
        Tell::Bump(n) => {
            *state.count().write() += n;
            JsValue::undefined()
        }
    }
}
// #endregion all
