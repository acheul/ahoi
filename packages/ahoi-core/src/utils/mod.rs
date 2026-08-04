use super::*;

pub(crate) mod hail_utils;
pub(crate) mod location;
pub(crate) mod path;

// Consistent HashMaker over Local Thread
thread_local! {
    static HASHSER: hashbrown::DefaultHashBuilder = hashbrown::DefaultHashBuilder::default();
}

/// Consistent hash value over Local Thread
pub fn get_hash<K: Hash + ?Sized>(key: &K) -> u64 {
    HASHSER.with(|state| {
        let mut hasher = state.build_hasher();
        key.hash(&mut hasher);
        return hasher.finish();
    })
}
