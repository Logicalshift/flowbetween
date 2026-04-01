// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash};

///
/// A basic LRU cache, used for things like ShapeIds which may end up requiring a lot of space
///
pub struct IdCache<K: Eq + Hash + Clone, V> {
    map:        HashMap<K, V>,
    order:      VecDeque<K>,
    capacity:   usize,
}

impl<K: Eq + Hash + Clone, V> IdCache<K, V> {
    ///
    /// Creates a new LRU cache that will contain a certain number of items
    ///
    pub fn new(capacity: usize) -> Self {
        Self {
            map:        HashMap::new(),
            order:      VecDeque::new(),
            capacity,
        }
    }

    ///
    /// Retrieves an item from the cache (without changing its priority)
    ///
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.map.contains_key(key) {
            self.order.push_back(key.clone());
            self.map.get(key)
        } else {
            None
        }
    }

    ///
    /// Caches an item, replacing the existing item if it's already present
    ///
    /// As for 'get()', existing items don't have their priority changed
    ///
    pub fn insert(&mut self, key: K, value: V) {
        if self.map.contains_key(&key) {
            // Update existing: replace value and promote
            self.map.insert(key.clone(), value);
            self.order.push_back(key);
        } else {
            // Evict oldest if at capacity
            if self.map.len() >= self.capacity {
                if let Some(oldest) = self.order.pop_front() {
                    self.map.remove(&oldest);
                }
            }
            self.map.insert(key.clone(), value);
            self.order.push_back(key);
        }
    }

    ///
    /// Removes an item from the cache
    ///
    pub fn remove(&mut self, key: &K) {
        if self.map.remove(key).is_some() {
            // Remove from the order (can be slow as we iterate through all of the existing elements)
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
            }
        }
    }

    ///
    /// Iterates through the cached items, keeping the values that match the predicate
    ///
    pub fn retain(&mut self, mut f: impl FnMut(&K, &V) -> bool) {
        self.map.retain(|k, v| f(k, v));
        self.order.retain(|k| self.map.contains_key(k));
    }
}
