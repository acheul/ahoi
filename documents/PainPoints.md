# 💥 Pain Points

and why use Ahoi?

|                    | Js Framework + Wasm | Rust Framework | JS Framework + Ahoi |
| ------------------ | ------------------- | -------------- | ------------------- |
| **Components**     | JS                  | Rust 💥        | JS                  |
|                    |                     |                | ↕                   |
| **Reactive State** | JS                  | Rust           | Rust                |
|                    | ↕ 💥                |                |                     |
| **Rust-side Data** | Rust                | Rust           | Rust                |

- 💥 Js Framework + Wasm: maintain communication between JS reactive state & rust-side data all by hand.

- 💥 Rust Framework: should handle everything in rust, including ones which JS fits better! (ex. event handling)

- Ahoi removes these pain points. Use rust for rust, JS for JS.
