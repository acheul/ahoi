use hashbrown::{HashMap, HashSet};
type IntMap<K, V> = HashMap<K, V, nohash_hasher::BuildNoHashHasher<K>>;
type IntSet<T> = HashSet<T, nohash_hasher::BuildNoHashHasher<T>>;
type IntIndexMap<K, V> = indexmap::IndexMap<K, V, nohash_hasher::BuildNoHashHasher<K>>;

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

mod utils;
use utils::path::*;

mod states;
use states::runtime;

pub mod hooks;

pub mod ts;

#[cfg(test)]
mod tests;

// exports for proc macro

#[doc(hidden)]
pub mod __macro_support {
    pub use crate::hooks::{
        ChainedPipe, GetNext, GetNextOpt, MapNext, Memo, OptReadStock, OptStock, Pipeline,
        ReadStock, Stock,
    };
    pub use crate::ts::TsDecl;
}

// exports for end users

pub use ahoi_stock_macro::{Stock, stock};
pub use ahoi_ts_macro::Rets;

/// Type Alias for `hashbrown::HashMap<SphereId, Box<dyn Any>, nohash_hasher::BuildNoHashHasher<K>>`;
pub type HailsMap = IntMap<SphereId, Box<dyn Any>>; // export for HailDispatcher

pub use utils::{
    get_hash,
    hail_utils::{HailConverter, HailDispatcher, set_local_hail_dispatcher},
};

pub use states::{
    pool::StateId,
    runtime::propagation::{batch, batch_with_sphere},
    runtime::sphere::{
        SphereId, clear_sphere, current_sphere_id, make_sphere, make_top_sphere, provide_context,
        use_context,
    },
};

pub use hooks::*;
