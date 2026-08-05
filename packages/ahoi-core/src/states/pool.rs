use super::*;
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

/// StateId
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateId(DefaultKey);

impl std::hash::Hash for StateId {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u64(self.0.data().as_ffi());
    }
}

impl nohash_hasher::IsEnabled for StateId {}

impl From<StateId> for u64 {
    fn from(id: StateId) -> Self {
        id.0.data().as_ffi()
    }
}

impl From<u64> for StateId {
    fn from(id: u64) -> Self {
        Self(DefaultKey::from(KeyData::from_ffi(id)))
    }
}

impl From<StateId> for DefaultKey {
    fn from(id: StateId) -> Self {
        id.0
    }
}

impl Deref for StateId {
    type Target = DefaultKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct Slot(RefCell<State>);

struct Pool {
    // Each Slot is heap-allocated (Box<Slot>) so its address is stable even when
    // the SlotMap's internal Vec grows and reallocates. This is required for safety:
    // pool::get / pool::get_mut return Ref<'static> / RefMut<'static> that borrow a
    // Slot's RefCell. If a subsequent pool::insert triggered a Vec reallocation while
    // one of those guards was live, the guard's internal pointer would dangle. By boxing
    // each Slot the pointed-to RefCell never moves, so the 'static borrow remains valid.
    slots: *mut SlotMap<DefaultKey, Box<Slot>>,
}

thread_local! {
    static POOL: RefCell<Pool> = RefCell::new(Pool::default());
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.slots)) }
    }
}

impl Default for Pool {
    fn default() -> Self {
        Self {
            slots: Box::into_raw(Box::new(SlotMap::with_capacity(20))),
        }
    }
}

/// Inser a state and return allocated index
pub(crate) fn insert_state(state: State) -> StateId {
    POOL.with_borrow_mut(|pool| {
        let slots = unsafe { &mut *pool.slots };
        let key = slots.insert(Box::new(Slot(RefCell::new(state))));
        StateId(key)
    })
}

/// Remove a state by given id
/// * Return None if the id is not present
pub(crate) fn remove_state(id: StateId) -> Option<State> {
    POOL.with_borrow_mut(|pool| {
        let Slot(e) = *unsafe { &mut *pool.slots }.remove(id.0)?;
        Some(e.into_inner())
    })
}

/// Borrowing-states Error type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorrowError {
    /// State is disposed from pool (not present in pool)
    Disposed,
    /// Conflict of borrow guard
    BorrowConflict,
}

/// Get Ref guarded state. Return None if state not exists
/// * `RefCell::borrow` is `#[track_caller]`, so keeping the attribute on this
///   whole chain makes a `BorrowError` name the user's read instead of this line.
#[cfg_attr(debug_assertions, track_caller)]
pub(crate) fn get_state(id: StateId) -> Result<Ref<'static, State>, BorrowError> {
    let Some(slot) = POOL.with_borrow(|pool| unsafe { &*pool.slots }.get(id.0).map(|b| b.as_ref()))
    else {
        return Err(BorrowError::Disposed);
    };
    slot.0.try_borrow().map_err(|_| BorrowError::BorrowConflict)
}

/// Get RefMut guarded state. Return None if state not exists
/// * See [`get_state`]: this is where `BorrowMutError` is raised.
#[cfg_attr(debug_assertions, track_caller)]
pub(crate) fn get_mut_state(id: StateId) -> Result<RefMut<'static, State>, BorrowError> {
    let Some(slot) = POOL.with_borrow(|pool| unsafe { &*pool.slots }.get(id.0).map(|b| b.as_ref()))
    else {
        return Err(BorrowError::Disposed);
    };
    slot.0
        .try_borrow_mut()
        .map_err(|_| BorrowError::BorrowConflict)
}

/// Number of live slots in the pool (test-only; used to assert no state leaks).
#[cfg(test)]
pub(crate) fn slots_count() -> usize {
    POOL.with_borrow(|pool| unsafe { &*pool.slots }.len())
}
