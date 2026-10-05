//! Shared contiguous Robin Hood storage for OCCT flat collections.
use super::HashPolicy;
#[derive(Clone)]
pub(super) struct FlatSlot<K, V> {
    pub key: K,
    pub value: V,
    hash: usize,
    probe: usize,
}
pub(super) struct FlatTable<K, V, H> {
    pub slots: Vec<Option<FlatSlot<K, V>>>,
    pub count: usize,
    pub hasher: H,
}
pub(super) struct Inserted {
    pub is_new: bool,
    pub original: usize,
    pub last: usize,
}
pub(super) struct FlatItems<'a, K, V> {
    pub slots: &'a [Option<FlatSlot<K, V>>],
    pub index: usize,
}
impl<K, V, H: HashPolicy<K>> FlatTable<K, V, H> {
    pub fn new(hasher: H, hint: usize) -> Self {
        let mut table = Self {
            slots: Vec::new(),
            count: 0,
            hasher,
        };
        if hint > 0 {
            table.reserve(hint);
        }
        table
    }
    pub fn reserve(&mut self, count: usize) {
        let capacity = capacity_for(count);
        if capacity > self.slots.len() {
            self.rehash(capacity);
        }
    }
    pub fn prepare(&mut self) {
        if capacity_for(self.count + 1) > self.slots.len() {
            self.rehash((self.slots.len() * 2).max(8));
        }
    }
    pub fn lookup(&self, key: &K) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let hash = self.hasher.hash(key);
        let mask = self.slots.len() - 1;
        let mut index = hash & mask;
        let mut probe = 0;
        loop {
            let slot = self.slots[index].as_ref()?;
            if probe > slot.probe {
                return None;
            }
            if slot.hash == hash && self.hasher.equal(&slot.key, key) {
                return Some(index);
            }
            probe += 1;
            index = (index + 1) & mask;
        }
    }
    pub fn insert(&mut self, key: K, value: V, replace_key: bool, replace_value: bool) -> Inserted {
        self.prepare();
        if let Some(index) = self.lookup(&key) {
            let slot = self.slots[index].as_mut().unwrap();
            if replace_key {
                slot.key = key;
            }
            if replace_value {
                slot.value = value;
            }
            return Inserted {
                is_new: false,
                original: index,
                last: index,
            };
        }
        let hash = self.hasher.hash(&key);
        self.insert_unchecked(FlatSlot {
            key,
            value,
            hash,
            probe: 0,
        })
    }
    fn insert_unchecked(&mut self, mut pending: FlatSlot<K, V>) -> Inserted {
        let mask = self.slots.len() - 1;
        let mut index = pending.hash & mask;
        let mut original = None;
        loop {
            let Some(stored) = self.slots[index].as_mut() else {
                self.slots[index] = Some(pending);
                self.count += 1;
                return Inserted {
                    is_new: true,
                    original: original.unwrap_or(index),
                    last: index,
                };
            };
            if pending.probe > stored.probe {
                std::mem::swap(stored, &mut pending);
                original.get_or_insert(index);
            }
            pending.probe += 1;
            index = (index + 1) & mask;
        }
    }
    pub fn remove(&mut self, key: &K) -> bool {
        let Some(index) = self.lookup(key) else {
            return false;
        };
        self.slots[index] = None;
        self.count -= 1;
        self.shift_delete(index);
        true
    }
    fn shift_delete(&mut self, mut index: usize) {
        let mask = self.slots.len() - 1;
        let mut next = (index + 1) & mask;
        while self.slots[next].as_ref().is_some_and(|slot| slot.probe > 0) {
            let mut slot = self.slots[next].take().unwrap();
            slot.probe -= 1;
            self.slots[index] = Some(slot);
            index = next;
            next = (next + 1) & mask;
        }
    }
    fn rehash(&mut self, capacity: usize) {
        let fresh = (0..capacity).map(|_| None).collect();
        let previous = std::mem::replace(&mut self.slots, fresh);
        self.count = 0;
        for mut slot in previous.into_iter().flatten() {
            slot.probe = 0;
            self.insert_unchecked(slot);
        }
    }
    pub fn items(&self) -> FlatItems<'_, K, V> {
        let mut iter = FlatItems {
            slots: &self.slots,
            index: 0,
        };
        iter.seek();
        iter
    }
}
impl<K, V, H> FlatTable<K, V, H> {
    pub fn clear(&mut self, release: bool) {
        self.count = 0;
        if release {
            self.slots = Vec::new();
            return;
        }
        for slot in &mut self.slots {
            *slot = None;
        }
    }
}
impl<K: Clone, V: Clone, H: Clone> FlatTable<K, V, H> {
    fn cloned_slots(&self) -> Vec<Option<FlatSlot<K, V>>> {
        if self.count == 0 {
            return Vec::new();
        }
        self.slots.clone()
    }
}
impl<K: Clone, V: Clone, H: Clone> Clone for FlatTable<K, V, H> {
    fn clone(&self) -> Self {
        // OCCT constructs myHasher before allocating and copying the occupied slots.
        let hasher: H = self.hasher.clone();
        Self {
            slots: self.cloned_slots(),
            count: self.count,
            hasher,
        }
    }
    fn clone_from(&mut self, source: &Self) {
        // Pinned OCCT 3d097a0 copy assignment clears before the policy assignment hook.
        self.clear(true);
        self.hasher.clone_from(&source.hasher);
        self.slots = source.cloned_slots();
        self.count = source.count;
    }
}
fn capacity_for(count: usize) -> usize {
    count
        .checked_mul(16)
        .expect("flat map count exceeds address space")
        .div_ceil(13)
        .max(8)
        .checked_next_power_of_two()
        .expect("flat map capacity exceeds address space")
}
mod iteration;
