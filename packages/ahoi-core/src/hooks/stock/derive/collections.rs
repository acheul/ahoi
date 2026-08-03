//! Stock Derive Ext for Vec, std HashMap, and hashbrown HashMap
use super::*;

#[derive(Clone, Copy)]
pub struct GetNextKey<K = usize> {
    key: K,
}

// Vec Ext

impl<T> MapNext<Vec<T>, T> for GetNextKey {
    fn as_ref<'a>(&self, vec: &'a Vec<T>) -> Option<&'a T> {
        vec.get(self.key)
    }
    fn as_mut<'a>(&self, vec: &'a mut Vec<T>) -> Option<&'a mut T> {
        vec.get_mut(self.key)
    }
}

impl<T: 'static, Pipe, const OPT: bool> Stock<Vec<T>, Pipe, OPT> {
    pub fn get(self, index: usize) -> Stock<T, ChainedPipe<Pipe, GetNextKey, Vec<T>, T>, true> {
        self.try_derive(index as u64, GetNextKey { key: index })
    }
}

// hashbrown HashMap Ext

impl<K: Eq + Hash + Clone, V, S: BuildHasher> MapNext<hashbrown::HashMap<K, V, S>, V>
    for GetNextKey<K>
{
    fn as_ref<'a>(&self, map: &'a hashbrown::HashMap<K, V, S>) -> Option<&'a V> {
        map.get(&self.key)
    }
    fn as_mut<'a>(&self, map: &'a mut hashbrown::HashMap<K, V, S>) -> Option<&'a mut V> {
        map.get_mut(&self.key)
    }
}

impl<K: Eq + Hash + Clone + 'static, V: 'static, S: BuildHasher, Pipe, const OPT: bool>
    Stock<hashbrown::HashMap<K, V, S>, Pipe, OPT>
{
    pub fn get(
        self,
        key: K,
    ) -> Stock<V, ChainedPipe<Pipe, GetNextKey<K>, hashbrown::HashMap<K, V, S>, V>, true> {
        self.try_derive(get_hash(&key), GetNextKey { key })
    }
}

// std HashMap Ext

impl<K: Eq + Hash + Clone, V, S: BuildHasher> MapNext<std::collections::HashMap<K, V, S>, V>
    for GetNextKey<K>
{
    fn as_ref<'a>(&self, map: &'a std::collections::HashMap<K, V, S>) -> Option<&'a V> {
        map.get(&self.key)
    }
    fn as_mut<'a>(&self, map: &'a mut std::collections::HashMap<K, V, S>) -> Option<&'a mut V> {
        map.get_mut(&self.key)
    }
}

impl<K: Eq + Hash + Clone + 'static, V: 'static, S: BuildHasher, Pipe, const OPT: bool>
    Stock<std::collections::HashMap<K, V, S>, Pipe, OPT>
{
    pub fn get(
        self,
        key: K,
    ) -> Stock<V, ChainedPipe<Pipe, GetNextKey<K>, std::collections::HashMap<K, V, S>, V>, true>
    {
        self.try_derive(get_hash(&key), GetNextKey { key })
    }
}
