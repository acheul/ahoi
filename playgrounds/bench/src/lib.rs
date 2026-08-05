//! Bench wasm crate — reactivity micro-benchmarks for core development.
//!
//! Not a framework showcase: the JS side drives `AhoiStorage` directly
//! (no solid/react), so numbers isolate the ahoi core + bridge. Each key is
//! shaped for one measurement:
//!
//! | key               | measures                                          |
//! |-------------------|---------------------------------------------------|
//! | `Tell::Noop`      | pure boundary cost (JS→wasm→serde→batch→return)   |
//! | `Hail::Cell(i)` + write | write-hail round-trip through a derived path |
//! | `Tell::Bump(i)`   | tell round-trip mutating one derived cell         |
//! | `Tell::WriteAll`  | fan-out: one parent write → N cell dispatches     |
//! | `Hail::Chain(d)` + `Tell::SetSrc` | propagation through a d-deep memo chain |
//! | enrol/clear (JS side) | hail sphere setup / teardown cost             |

use ahoi::js_bridge::SerdeWasmBindgenConverter as Converter;
use ahoi::js_bridge::*;
use ahoi::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

pub const CELLS: usize = 1024;

#[derive(Stock, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchState {
    pub cells: Vec<i32>,
    pub src: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pier {
    Bench,
}

wasm_bindgen_enrol_sphere!(@pier, Pier, run_pier, Converter);

fn run_pier(key: Pier) {
    match key {
        Pier::Bench => {
            set_js_hail_dispatcher();
            let state = Stock::new(BenchState {
                cells: vec![0; CELLS],
                src: 0,
            });
            provide_context(state);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hail {
    /// cells[i], writable (derived path)
    Cell(u32),
    /// leaf of a `d`-deep memo chain over `src`
    Chain(u32),
}

wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);

fn run_hail(key: Hail) -> JsValue {
    let state = use_context::<Stock<BenchState>>().unwrap();
    match key {
        Hail::Cell(i) => state.cells().get(i as usize).set_hail::<Converter>(),
        Hail::Chain(depth) => {
            let src = state.src();
            let mut memo = Memo::new(move || *src.read() + 1);
            for _ in 1..depth {
                let prev = memo;
                memo = Memo::new(move || *prev.read() + 1);
            }
            memo.set_read_hail::<Converter>()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tell {
    /// does nothing — isolates the boundary cost
    Noop,
    /// cells[i] += 1
    Bump(u32),
    /// every cell += 1 via one parent write (fan-out to all Cell hails)
    WriteAll,
    /// src = v (drives the memo chain)
    SetSrc(i32),
}

wasm_bindgen_tell!(Tell, run_tell, Converter);

fn run_tell(tell: Tell) -> JsValue {
    let state = use_context::<Stock<BenchState>>().unwrap();
    match tell {
        Tell::Noop => JsValue::undefined(),
        Tell::Bump(i) => {
            *state.cells().get(i as usize).write().unwrap() += 1;
            JsValue::undefined()
        }
        Tell::WriteAll => {
            let mut cells = state.cells().write();
            for c in cells.iter_mut() {
                *c += 1;
            }
            JsValue::undefined()
        }
        Tell::SetSrc(v) => {
            *state.src().write() = v;
            JsValue::undefined()
        }
    }
}
