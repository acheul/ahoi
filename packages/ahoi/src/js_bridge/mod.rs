use crate::*;
pub use js_sys;
pub use wasm_bindgen;
use wasm_bindgen::prelude::*;

#[cfg(feature = "serde-wasm-bindgen")]
pub mod converters;
#[cfg(feature = "serde-wasm-bindgen")]
pub use converters::*;

/// Version of the wasm⇄JS bridge ABI (the `pier`/`hail`/`clear`/`write`/`tell`
/// surface). The npm-side bridge checks this at init to catch crate/npm
/// version skew. Bump only when the exported function signatures or the
/// `__AHOI__.hail` callback contract change.
pub const ABI_VERSION: u32 = 1;

/// Exported for the JS bridge's runtime ABI check.
#[wasm_bindgen]
pub fn abi_version() -> u32 {
    ABI_VERSION
}

/// Dispatch Bridge Hail
#[wasm_bindgen]
extern "C" {
    // Binds to the `hail`: It must exists in JS
    #[wasm_bindgen(js_namespace = "__AHOI__", js_name = "hail")]
    fn hail(a: js_sys::Array);
}

pub struct JsValueHailDispatcher;

pub fn set_js_hail_dispatcher() {
    set_local_hail_dispatcher(JsValueHailDispatcher);
}

impl HailDispatcher for JsValueHailDispatcher {
    fn dispatch_hails(&self, hails: HailsMap) {
        let array: js_sys::Array = hails
            .into_iter()
            .flat_map(|(id, value)| {
                let id: JsValue = id.into();
                let value = *value.downcast::<JsValue>().unwrap();
                [id, value]
            })
            .collect();
        hail(array);
    }
}

/// Write Hail Value
#[wasm_bindgen]
pub fn write(sphere_id: u32, hail_value: JsValue) {
    crate::hooks::write_hail(sphere_id, hail_value);
}

/// Expands into a wasm-bindgen sphere-enrolment function. Invoke once per kind:
///
/// - `@pier`: enrols a Pier (scope-like) sphere. The runner only sets up the
///   sphere (context, effects, …) and returns nothing; the export returns the
///   new sphere id.
///   ```rust, ignore
///       #[wasm_bindgen]
///       pub fn pier(par_sphere_id: Option<u32>, key: wasm_bindgen::JsValue) -> u32;
///   ```
///
/// - `@hail`: enrols a Hail (reactive value channel) sphere. The runner returns
///   the initial hail value; the export returns `[sphere_id, initial_value]`.
///   ```rust, ignore
///       #[wasm_bindgen]
///       pub fn hail(par_sphere_id: u32, key: wasm_bindgen::JsValue) -> js_sys::Array;
///   ```
///
/// #### macro parameters
/// * `$pier_key`/`$hail_key:ty`: the Pier / Hail key enum
/// * `$runner:ident`: `@pier` → `fn(PierKey)`, `@hail` → `fn(HailKey) -> JsValue`
/// * `$converter:ty`: Type of an [HailConverter]
#[macro_export]
macro_rules! wasm_bindgen_enrol_sphere {
    (@pier, $pier_key:ty, $runner:ident, $converter:ty) => {
        #[wasm_bindgen]
        pub fn pier(par_sphere_id: Option<u32>, key: wasm_bindgen::JsValue) -> u32 {
            // 1. decode the key
            let key: $pier_key = <$converter as HailConverter<$pier_key>>::into_raw_value(key);

            // 2. make sphere
            let (sphere_id, _): (u32, ()) = ahoi::make_sphere(par_sphere_id, || $runner(key));

            // 3. Return sphere_id
            return sphere_id;
        }
    };
    (@hail, $hail_key:ty, $runner:ident, $converter:ty) => {
        #[wasm_bindgen]
        pub fn hail(par_sphere_id: u32, key: wasm_bindgen::JsValue) -> js_sys::Array {
            // 1. decode the key
            let key: $hail_key = <$converter as HailConverter<$hail_key>>::into_raw_value(key);

            // 2. make sphere
            let (sphere_id, value): (u32, wasm_bindgen::JsValue) =
                ahoi::make_sphere(Some(par_sphere_id), || $runner(key));

            // 3. Return (sphere-id, initial-hail-value)
            return js_sys::Array::of2(&sphere_id.into(), &value);
        }
    };
}

/// Clear Sphere
#[wasm_bindgen]
pub fn clear(sphere_id: u32) {
    ahoi_core::clear_sphere(sphere_id);
}

/// Expanded into `tell` wasm-bindgen function:
///  ```rust, ignore
///     #[wasm_bindgen]
///     pub fn tell(sphere_id: u32, key: wasm_bindgen::JsValue) -> wasm_bindgen::JsValue;
/// ```
/// #### macro parameters
/// * `$tell_key:ty`: Type of Tell enum
/// * `$runner:ident`: `fn(TellKey) -> JsValue`
/// * `$converter:ty`: Type of an [HailConverter]
#[macro_export]
macro_rules! wasm_bindgen_tell {
    ($tell_key:ty, $runner:ident, $converter:ty) => {
        #[wasm_bindgen]
        pub fn tell(sphere_id: u32, key: wasm_bindgen::JsValue) -> wasm_bindgen::JsValue {
            // 1. decode the key
            let key: $tell_key = <$converter as HailConverter<$tell_key>>::into_raw_value(key);

            // 2. run tell & return res
            let res: wasm_bindgen::JsValue = ahoi::batch_with_sphere(sphere_id, || $runner(key));
            return res;
        }
    };
}
