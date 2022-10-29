//! This module implements the scoped map.

use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

/// Scoped map that supports inserting and removing lots of key-val pairs
/// at once.
pub struct ScopedMap<K, V> {
    stack: Vec<Vec<(K, V)>>,
}

impl<K: PartialEq + Clone + Hash + Eq, V: Clone> Default for ScopedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: PartialEq + Clone + Hash + Eq, V: Clone> ScopedMap<K, V> {
    pub fn new() -> Self {
        ScopedMap { stack: Vec::new() }
    }

    pub fn push(&mut self) {
        self.stack.push(Vec::new());
    }

    pub fn pop(&mut self) {
        if !self.is_empty() {
            self.stack.pop();
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn insert(&mut self, key: &K, val: &V) {
        assert!(!self.is_empty());
        let scope = self.stack.last_mut().unwrap();
        for pair in scope {
            if pair.0 == *key {
                (*pair).1 = val.clone();
                return;
            }
        }

        self.stack
            .last_mut()
            .unwrap()
            .push((key.clone(), val.clone()));
    }

    pub fn flatten(&self) -> HashMap<K, V> {
        let mut map: HashMap<K, V> = HashMap::new();

        for scope in self.stack.iter() {
            for pair in scope.iter() {
                map.insert(pair.0.clone(), pair.1.clone());
            }
        }

        map
    }

    pub fn get(&self, key: &K) -> Option<V> {
        // For each scope, in reverse:
        for scope in self.stack.iter().rev() {
            for pair in scope {
                if pair.0 == *key {
                    return Option::Some(pair.1.clone());
                }
            }
        }
        Option::None
    }

    pub fn has(&self, key: &K) -> bool {
        matches!(self.get(key), Option::Some(_))
    }
}

#[test]
fn test_scoped_map() {
    let mut map: ScopedMap<usize, usize> = ScopedMap::new();

    assert!(map.is_empty());
    map.push();
    assert_eq!(map.len(), 1);

    map.insert(&1, &1);
    map.insert(&2, &2);
    map.insert(&3, &3);

    assert_eq!(map.get(&1).unwrap(), 1);
    assert_eq!(map.get(&2).unwrap(), 2);
    assert_eq!(map.get(&3).unwrap(), 3);

    assert!(map.has(&1));
    assert!(map.has(&2));
    assert!(map.has(&3));

    map.push();

    assert!(map.has(&1));
    assert!(map.has(&2));
    assert!(map.has(&3));

    map.insert(&1, &4);
    map.insert(&2, &5);
    map.insert(&3, &6);

    assert_eq!(map.get(&1).unwrap(), 4);
    assert_eq!(map.get(&2).unwrap(), 5);
    assert_eq!(map.get(&3).unwrap(), 6);

    map.pop();
    assert_eq!(map.get(&1).unwrap(), 1);
    assert_eq!(map.get(&2).unwrap(), 2);
    assert_eq!(map.get(&3).unwrap(), 3);
    map.pop();
    assert!(!map.has(&1));
    assert!(!map.has(&2));
    assert!(!map.has(&3));
}

#[test]
fn test_scoped_map2() {
    let mut map: ScopedMap<usize, usize> = ScopedMap::new();
    map.push();
    map.insert(&1, &1);
    map.push();
    map.insert(&1, &2);
    map.insert(&2, &3);
    map.push();

    let flat = map.flatten();
    assert!(flat.contains_key(&1));
    assert!(flat.contains_key(&2));
    assert!(!flat.contains_key(&3));
    assert_eq!(*flat.get(&1).unwrap(), 2);
}
