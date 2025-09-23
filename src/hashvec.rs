use std::{
    borrow::Borrow,
    cmp::Ordering,
    collections::HashMap,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::{Deref, Index},
};

#[derive(Debug)]
pub struct OpaqueIndex<T> {
    index: usize,
    _p: PhantomData<T>,
}

impl<T> OpaqueIndex<T> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            _p: PhantomData,
        }
    }
}

// Manual Clone impl so T doesn't inherit the requirement
impl<T> Clone for OpaqueIndex<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for OpaqueIndex<T> {}

impl<T> Deref for OpaqueIndex<T> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

impl<T> PartialEq for OpaqueIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for OpaqueIndex<T> {}

impl<T> PartialOrd for OpaqueIndex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for OpaqueIndex<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Hash for OpaqueIndex<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

/// A lookup table with both HashMap and Vec lookups
#[derive(Debug)]
pub struct HashVec<K, V> {
    vec: Vec<V>,
    hm: HashMap<K, OpaqueIndex<V>>,
}

// Vector-like lookup
impl<K, V> Index<OpaqueIndex<V>> for HashVec<K, V>
where
    K: Eq + Hash,
{
    type Output = V;

    fn index(&self, index: OpaqueIndex<V>) -> &Self::Output {
        &self.vec[*index]
    }
}

impl<K, V> HashVec<K, V>
where
    K: Eq + Hash,
{
    /// Get the cheap-to-use opaque key
    pub fn opaque<Q>(&self, key: &Q) -> OpaqueIndex<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.hm[key]
    }

    /// Get the cheap-to-use opaque key
    pub fn get_opaque<Q>(&self, key: &Q) -> Option<OpaqueIndex<V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.hm.get(key).copied()
    }

    /// Hashmap-like lookup
    pub fn by_key<Q>(&self, key: &Q) -> &V
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        &self[self.opaque(key)]
    }

    // Insert a new pair into the HashVec
    // Must not already exist
    // Returns the opaque index for the newly inserted value
    pub fn insert(&mut self, key: K, value: V) -> OpaqueIndex<V> {
        let index = OpaqueIndex::new(self.vec.len());
        self.vec.push(value);
        let existed = self.hm.insert(key, index);
        assert!(existed.is_none(), "Key already existed!");
        index
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.hm.contains_key(key)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.vec.iter_mut()
    }
}

impl<K, V> Default for HashVec<K, V> {
    fn default() -> Self {
        Self {
            vec: vec![],
            hm: HashMap::new(),
        }
    }
}

impl<I, K, V> From<I> for HashVec<K, V>
where
    I: IntoIterator<Item = (K, V)>,
    K: Eq + Hash,
{
    fn from(value: I) -> Self {
        value.into_iter().fold(Self::default(), |mut hv, (k, v)| {
            hv.insert(k, v);
            hv
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::hashvec::{HashVec, OpaqueIndex};

    #[test]
    fn test_hashvec() {
        let values = 0..10;
        let keys = values.clone().map(|v| format!("{v:?}")).collect::<Vec<_>>();

        let mut hash_vec = HashVec::<String, i32>::default();

        for (i, (k, v)) in keys.into_iter().zip(values).enumerate() {
            hash_vec.vec.push(v);
            hash_vec.hm.insert(k, OpaqueIndex::new(i));
        }

        assert_eq!(hash_vec.by_key("3"), &3);
        assert_eq!(hash_vec.opaque("3").index, 3);
        assert_eq!(hash_vec[OpaqueIndex::new(3)], 3);
    }
}
