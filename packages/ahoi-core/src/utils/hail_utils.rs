use super::*;

thread_local! {
    static HAIL_DISPATCHER: RefCell<Option<Box<dyn HailDispatcher>>> = RefCell::new(None);
}

/// Call this on top sphere
pub fn set_local_hail_dispatcher(dispatcher: impl HailDispatcher + 'static) {
    HAIL_DISPATCHER.with_borrow_mut(|x| {
        let _ = x.replace(Box::new(dispatcher));
    });
}

/// HailDispatcher logic
pub trait HailDispatcher {
    /// Dispatch collected hails all at once
    fn dispatch_hails(&self, hails: HailsMap);
}

// NOTE: "collect then dispatch", rather than "dispatch each one" - for efficiency
pub(crate) fn dispatch_hails(hails: IntMap<SphereId, Box<dyn Any>>) {
    HAIL_DISPATCHER.with_borrow(|dispatcher| {
        let dispatcher = dispatcher.as_ref().expect("Set hail dispatcher");
        dispatcher.dispatch_hails(hails);
    });
}

/// Hail Converter trait: Convert `raw-value<T>` into an HailValue type
/// * Value convertion must succeed. Not using Result or Option result.
pub trait HailConverter<T>: Sized {
    type HailValue;

    const NONE: Self::HailValue;

    /// raw-value to hail-value
    fn from_raw_value(raw_value: &T) -> Self::HailValue;

    /// hail-value to raw-value
    fn into_raw_value(hail_value: Self::HailValue) -> T;

    fn __from_option_raw_value(
        raw_value: Option<Ref<'static, T>>,
        allow_none: bool,
    ) -> Self::HailValue {
        match raw_value {
            Some(raw_value) => Self::from_raw_value(raw_value.deref()),
            None => {
                if allow_none {
                    Self::NONE
                } else {
                    unreachable!()
                }
            }
        }
    }
}
