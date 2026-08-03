pub use ahoi_core::{self, *};

#[cfg(feature = "js")]
pub mod js_bridge;

#[cfg(feature = "js")]
pub use ahoi_rets_macro::Rets;

// Extends ahoi-core's macro-support surface (glob-imported above) with the
// items the `Rets` derive references; an explicit module shadows the
// glob-re-exported one, so it must re-export ahoi-core's items too.
#[doc(hidden)]
pub mod __macro_support {
    pub use ahoi_core::__macro_support::*;

    #[cfg(feature = "js")]
    pub use crate::js_bridge::ts::TsDecl;
}

#[cfg(test)]
mod test_ts;
