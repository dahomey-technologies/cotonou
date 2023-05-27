use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    hash::Hash, iter::FusedIterator,
};

type Order = u32;

#[derive(Clone)]
pub struct QueueMap<T> {
    queue: BTreeMap<Order, T>,
    map: HashMap<T, Order>,
    next_order: u32,
}

impl<T> QueueMap<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            queue: BTreeMap::new(),
            map: HashMap::new(),
            next_order: 0,
        }
    }

    pub fn insert(&mut self, t: T) -> Option<T> {
        let order = self.next_order;
        self.next_order += 1;
        self.map.insert(t.clone(), order);
        self.queue.insert(order, t)
    }

    pub fn remove<K>(&mut self, k: &K) -> Option<T>
    where
        T: Borrow<K>,
        K: Hash + Eq + ?Sized,
    {
        self.map
            .remove(k)
            .and_then(|order| self.queue.remove(&order))
    }
    
    pub fn clear(&mut self) {
        self.queue.clear();
        self.map.clear();
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            inner: self.queue.values()
        }
    }
}

pub struct Iter<'a, T> 
{
    inner: std::collections::btree_map::Values<'a, u32, T>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn count(self) -> usize
        where
            Self: Sized, {
        self.inner.len()
    }
}

impl<T> FusedIterator for Iter<'_, T> {}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        Iter { inner: self.inner.clone() }
    }
}
