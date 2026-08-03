use super::*;
use serde::{Serialize, de::DeserializeOwned};

// "serde-wasm-bindgen"

pub use serde_wasm_bindgen_converter::SerdeWasmBindgenConverter;

mod serde_wasm_bindgen_converter {
    use super::*;
    pub struct SerdeWasmBindgenConverter;

    impl<T: Serialize + DeserializeOwned> HailConverter<T> for SerdeWasmBindgenConverter {
        type HailValue = JsValue;

        const NONE: Self::HailValue = JsValue::undefined();

        fn from_raw_value(raw_value: &T) -> Self::HailValue {
            serde_wasm_bindgen::to_value(raw_value).unwrap()
        }

        fn into_raw_value(hail_value: Self::HailValue) -> T {
            serde_wasm_bindgen::from_value(hail_value).unwrap()
        }
    }
}
