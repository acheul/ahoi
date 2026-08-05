//! Tsain variant of the playground wasm crate, shaped as a small todo app.
//!
//! [Tsain](https://crates.io/crates/tsain) covers both halves of the bridge
//! at once, so the division of labour collapses to one crate feature:
//!
//! - **values** on the wire → `TsainConverter` (tsain's positional-array serde)
//! - key/data **types** → `#[derive(Tsain)]`, exported to `bindings/Tsain.ts`
//!   by the `generate` test — together with the factory functions and getters
//!   JS needs to build and read the array format
//! - key **return types** → `#[tsain(brand(ret = ..))]` on the key variants;
//!   the JS adapter resolves `HailRet`/`TellRet` from that `ret` brand, so
//!   there is no `#[derive(Rets)]` and no `TsFile` step at all
//!
//! The keys form a todo app, but each one still earns its place:
//!
//! | key                | shows                                            |
//! |--------------------|--------------------------------------------------|
//! | `Hail::UserName`   | writable hail (JS ⇄ Rust round-trip)             |
//! | `Hail::Filter`     | writable hail carrying a tsain enum              |
//! | `Hail::Todos`      | `Vec` of tsain structs on the wire               |
//! | `Hail::OpenCount`  | read-only hail over a memo                       |
//! | `Hail::Motto`      | context under the nested `User` pier + cleanup   |
//! | `Tell::AddTodo`    | tell with a payload, no return                   |
//! | `Tell::ToggleTodo` | tell mutating one element of a `Vec`             |
//! | `Tell::RemoveTodo` | `ret` brand returning `Option<Todo>`             |
//! | `Tell::ClearDone`  | `ret` brand returning a number                   |

use ahoi::js_bridge::{TsainConverter as Converter, *};
use ahoi::*;
use serde::{Deserialize, Serialize};
use tsain::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

// run this test to (re)generate `bindings/Tsain.ts`: every Tsain-derived type
// below, plus its factory functions and getters. This is the only generation
// step — ret types travel as brands on the key variants themselves.
#[test]
fn generate() {
    tsain::TsScript::export("./bindings/Tsain.ts");
}

// ── data ───────────────────────────────────────────────────────────────────

#[derive(Stock, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub user_name: String,
    pub filter: Filter,
    pub todo: Vec<Todo>,
}

#[derive(Stock, Tsain, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub status: Status,
}

#[derive(Stock, Tsain, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Status {
    Open,
    Done,
}

#[derive(Tsain, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Open,
    Done,
}

/// Context provided by the `User` pier only.
#[derive(Clone, Copy)]
struct Motto(Stock<String>);

// ── Pier ───────────────────────────────────────────────────────────────────

#[derive(Tsain, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pier {
    Top,
    User,
}

wasm_bindgen_enrol_sphere!(@pier, Pier, run_pier, Converter);

fn run_pier(key: Pier) {
    match key {
        Pier::Top => {
            set_js_hail_dispatcher();

            let state = Stock::new(State {
                user_name: String::from("Sailor"),
                filter: Filter::All,
                todo: vec![
                    Todo {
                        id: 1,
                        text: String::from("Chart the course"),
                        status: Status::Done,
                    },
                    Todo {
                        id: 2,
                        text: String::from("Hoist the sails"),
                        status: Status::Open,
                    },
                ],
            });
            provide_context(state);
        }
        Pier::User => {
            // Scoped to the profile section: cleared whenever that section
            // unmounts, while `State` above lives as long as `Top` does.
            let motto = Stock::new(String::from("ahoy, world"));
            provide_context(Motto(motto));
        }
    }
}

// ── Hail ───────────────────────────────────────────────────────────────────

#[derive(Tsain, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Hail {
    #[tsain(brand(ret = String))]
    UserName,
    #[tsain(brand(ret = Filter))]
    Filter,
    #[tsain(brand(ret = Vec<Todo>))]
    Todos,
    #[tsain(brand(ret = u32))]
    OpenCount,
    #[tsain(brand(ret = String))]
    Motto,
}

wasm_bindgen_enrol_sphere!(@hail, Hail, run_hail, Converter);

fn run_hail(key: Hail) -> JsValue {
    let state = || use_context::<Stock<State>>().unwrap();
    match key {
        Hail::UserName => state().user_name().set_hail::<Converter>(),
        Hail::Filter => state().filter().set_hail::<Converter>(),
        Hail::Todos => state().todo().set_hail::<Converter>(),
        Hail::OpenCount => {
            let open = state()
                .todo()
                .memo(|todo| todo.iter().filter(|t| t.status == Status::Open).count() as u32);
            open.set_read_hail::<Converter>()
        }
        Hail::Motto => {
            let Motto(motto) = use_context::<Motto>().unwrap();
            motto.set_hail::<Converter>()
        }
    }
}

// ── Tell ───────────────────────────────────────────────────────────────────

#[derive(Tsain, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tell {
    AddTodo(String),
    ToggleTodo(u32),
    #[tsain(brand(ret = Option<Todo>))]
    RemoveTodo(u32),
    #[tsain(brand(ret = u32))]
    ClearDone,
}

wasm_bindgen_tell!(Tell, run_tell, Converter);

fn run_tell(tell: Tell) -> JsValue {
    let state = use_context::<Stock<State>>().unwrap();
    match tell {
        Tell::AddTodo(text) => {
            let mut todo = state.todo().write();
            let id = todo.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            todo.push(Todo {
                id,
                text,
                status: Status::Open,
            });
            JsValue::undefined()
        }
        Tell::ToggleTodo(id) => {
            let mut todo = state.todo().write();
            if let Some(t) = todo.iter_mut().find(|t| t.id == id) {
                t.status = match t.status {
                    Status::Open => Status::Done,
                    Status::Done => Status::Open,
                };
            }
            JsValue::undefined()
        }
        Tell::RemoveTodo(id) => {
            let removed = {
                let mut todo = state.todo().write();
                todo.iter()
                    .position(|t| t.id == id)
                    .map(|index| todo.remove(index))
            };
            tsain::to_value(&removed).unwrap()
        }
        Tell::ClearDone => {
            let cleared = {
                let mut todo = state.todo().write();
                let before = todo.len();
                todo.retain(|t| t.status == Status::Open);
                (before - todo.len()) as u32
            };
            tsain::to_value(&cleared).unwrap()
        }
    }
}
