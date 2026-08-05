use super::*;
use serde::{Serialize, de::DeserializeOwned};

// "serde-wasm-bindgen"

#[cfg(feature = "serde-wasm-bindgen")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde-wasm-bindgen")))]
pub use serde_wasm_bindgen_converter::SerdeWasmBindgenConverter;

#[cfg(feature = "serde-wasm-bindgen")]
mod serde_wasm_bindgen_converter {
    use super::*;
    pub struct SerdeWasmBindgenConverter;

    impl<T: Serialize + DeserializeOwned> HailConverter<T> for SerdeWasmBindgenConverter {
        type HailValue = wasm_bindgen::JsValue;

        const NONE: Self::HailValue = wasm_bindgen::JsValue::undefined();

        fn from_raw_value(raw_value: &T) -> Self::HailValue {
            serde_wasm_bindgen::to_value(raw_value).unwrap()
        }

        fn into_raw_value(hail_value: Self::HailValue) -> T {
            serde_wasm_bindgen::from_value(hail_value).unwrap()
        }
    }
}

// "serde_json"

#[cfg(feature = "serde_json")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde_json")))]
pub use serde_json_converter::SerdeJsonConverter;

#[cfg(feature = "serde_json")]
mod serde_json_converter {
    use super::*;
    pub struct SerdeJsonConverter;

    impl<T: Serialize + DeserializeOwned> HailConverter<T> for SerdeJsonConverter {
        type HailValue = serde_json::Value;

        const NONE: Self::HailValue = serde_json::Value::Null;

        fn from_raw_value(raw_value: &T) -> Self::HailValue {
            serde_json::to_value(raw_value).unwrap()
        }

        fn into_raw_value(hail_value: Self::HailValue) -> T {
            serde_json::from_value(hail_value).unwrap()
        }
    }
}
