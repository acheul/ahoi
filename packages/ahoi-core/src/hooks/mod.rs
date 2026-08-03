use super::*;

mod stock;
pub use stock::*;

mod runners;
pub use runners::*;

mod hail;
pub use hail::*;

/// Helper of Clone Option<Ref<'_, T>>
pub trait OptionRef<T> {
    fn cloned(&self) -> Option<T>;
    fn copied(&self) -> Option<T>
    where
        T: Copy;
}

impl<T: Clone> OptionRef<T> for Option<Ref<'_, T>> {
    fn cloned(&self) -> Option<T> {
        match self {
            Some(e) => Some((*e).clone()),
            None => None,
        }
    }
    fn copied(&self) -> Option<T>
    where
        T: Copy,
    {
        match self {
            Some(e) => Some(**e),
            None => None,
        }
    }
}
