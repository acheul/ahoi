//! Stock Derive Ext for Vec, std HashMap, and hashbrown HashMap
use super::*;

#[derive(Clone, Copy)]
pub struct GetNextKey<K = usize> {
    key: K,
}

// Vec Ext

impl<T> MapNextOpt<Vec<T>, T> for GetNextKey {
    fn as_ref<'a>(&self, vec: &'a Vec<T>) -> Option<&'a T> {
        vec.get(self.key)
    }
    fn as_mut<'a>(&self, vec: &'a mut Vec<T>) -> Option<&'a mut T> {
        vec.get_mut(self.key)
    }
}

// hashbrown HashMap Ext

impl<K: Eq + Hash + Clone, V, S: BuildHasher> MapNextOpt<hashbrown::HashMap<K, V, S>, V>
    for GetNextKey<K>
{
    fn as_ref<'a>(&self, map: &'a hashbrown::HashMap<K, V, S>) -> Option<&'a V> {
        map.get(&self.key)
    }
    fn as_mut<'a>(&self, map: &'a mut hashbrown::HashMap<K, V, S>) -> Option<&'a mut V> {
        map.get_mut(&self.key)
    }
}

// std HashMap Ext

impl<K: Eq + Hash + Clone, V, S: BuildHasher> MapNextOpt<std::collections::HashMap<K, V, S>, V>
    for GetNextKey<K>
{
    fn as_ref<'a>(&self, map: &'a std::collections::HashMap<K, V, S>) -> Option<&'a V> {
        map.get(&self.key)
    }
    fn as_mut<'a>(&self, map: &'a mut std::collections::HashMap<K, V, S>) -> Option<&'a mut V> {
        map.get_mut(&self.key)
    }
}

// `get` on each stock type: an optional derive, so writable stocks yield
// OptStock and read-only ones yield OptReadStock.

macro_rules! impl_collection_get {
    ($ty:ident => $out:ident) => {
        impl<T: 'static, Pipe> $ty<Vec<T>, Pipe> {
            pub fn get(self, index: usize) -> $out<T, ChainedPipe<Pipe, GetNextKey, Vec<T>, T>> {
                self.derive_opt(index as u64, GetNextKey { key: index })
            }
        }

        impl<K: Eq + Hash + Clone + 'static, V: 'static, S: BuildHasher, Pipe>
            $ty<hashbrown::HashMap<K, V, S>, Pipe>
        {
            pub fn get(
                self,
                key: K,
            ) -> $out<V, ChainedPipe<Pipe, GetNextKey<K>, hashbrown::HashMap<K, V, S>, V>> {
                self.derive_opt(get_hash(&key), GetNextKey { key })
            }
        }

        impl<K: Eq + Hash + Clone + 'static, V: 'static, S: BuildHasher, Pipe>
            $ty<std::collections::HashMap<K, V, S>, Pipe>
        {
            pub fn get(
                self,
                key: K,
            ) -> $out<V, ChainedPipe<Pipe, GetNextKey<K>, std::collections::HashMap<K, V, S>, V>>
            {
                self.derive_opt(get_hash(&key), GetNextKey { key })
            }
        }
    };
}

impl_collection_get!(Stock => OptStock);
impl_collection_get!(OptStock => OptStock);
impl_collection_get!(ReadStock => OptReadStock);
impl_collection_get!(OptReadStock => OptReadStock);
