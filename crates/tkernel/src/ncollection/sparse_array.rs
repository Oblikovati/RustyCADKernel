// Created on: 2006-11-23
// Created by: Andrey BETENEV
// Copyright (c) 2006-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Sparse block allocation with stable occupied slots and ascending index traversal.
use std::collections::BTreeMap;
/// Sparse indexed objects; `SparseArray::new(8).set_value(1000, 42)`.
#[derive(Clone)]
pub struct SparseArray<T> {
    increment: usize,
    blocks: BTreeMap<usize, Box<[Option<T>]>>,
    count: usize,
    // Length of the native block-pointer table (`myNbBlocks`); `capacity()` reports it
    // times the increment, so its growth rule is observable.
    block_slots: usize,
}
impl<T> SparseArray<T> {
    /// Selects the number of slots allocated per block; `SparseArray::<i32>::new(8)`.
    pub fn new(increment: usize) -> Self {
        assert!(
            increment > 0,
            "SparseArray increment {increment}; expected a positive block size"
        );
        Self {
            increment,
            blocks: BTreeMap::new(),
            count: 0,
            block_slots: 0,
        }
    }
    /// Addressable slots before the block table must grow, as native
    /// `myNbBlocks * myBlockSize`; `SparseArray::<i32>::new(8).capacity() == 0`.
    pub fn capacity(&self) -> usize {
        self.block_slots * self.increment
    }
    // The native allocData rule: the table starts at `increment` entries and doubles
    // until it covers `block`.
    fn reserve_block(&mut self, block: usize) {
        if block < self.block_slots {
            return;
        }
        let mut slots = if self.block_slots == 0 {
            self.increment
        } else {
            self.block_slots
        };
        while block >= slots {
            slots *= 2;
        }
        self.block_slots = slots;
    }
    /// Returns the number of occupied slots; `array.len() == 2`.
    pub fn len(&self) -> usize {
        self.count
    }
    /// Returns the occupied extent; `array.extent() == 2`.
    pub fn extent(&self) -> usize {
        self.count
    }
    /// Checks that every slot is unoccupied; `array.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// Constructs or replaces an indexed object; `array.set_value(5, 55)`.
    pub fn set_value(&mut self, index: usize, value: T) -> &mut T {
        self.reserve_block(index / self.increment);
        let block = self
            .blocks
            .entry(index / self.increment)
            .or_insert_with(|| {
                std::iter::repeat_with(|| None)
                    .take(self.increment)
                    .collect()
            });
        let slot = &mut block[index % self.increment];
        if slot.is_none() {
            self.count += 1;
        }
        slot.insert(value)
    }
    /// Borrows an occupied slot; `array.value(5) == Some(&55)`.
    pub fn value(&self, index: usize) -> Option<&T> {
        self.blocks.get(&(index / self.increment))?[index % self.increment].as_ref()
    }
    /// Borrows an occupied slot exclusively; `*array.change_value(5).unwrap() = 10`.
    pub fn change_value(&mut self, index: usize) -> Option<&mut T> {
        self.blocks.get_mut(&(index / self.increment))?[index % self.increment].as_mut()
    }
    /// Checks whether a slot is constructed; `array.has_value(5)`.
    pub fn has_value(&self, index: usize) -> bool {
        self.value(index).is_some()
    }
    /// Destroys an occupied slot; `array.unset_value(5)`.
    pub fn unset_value(&mut self, index: usize) -> bool {
        let Some(block) = self.blocks.get_mut(&(index / self.increment)) else {
            return false;
        };
        if block[index % self.increment].take().is_none() {
            return false;
        }
        // As natively, a block whose last object is destroyed is freed.
        if block.iter().all(Option::is_none) {
            self.blocks.remove(&(index / self.increment));
        }
        self.count -= 1;
        true
    }
    /// Frees all blocks and objects; `array.clear()`.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.count = 0;
        self.block_slots = 0;
    }
    /// Visits occupied indices in ascending order; `array.entries().map(|(index, _)| index)`.
    pub fn entries(&self) -> impl Iterator<Item = (usize, &T)> {
        let increment = self.increment;
        self.blocks.iter().flat_map(move |(&number, block)| {
            block.iter().enumerate().filter_map(move |(offset, slot)| {
                slot.as_ref()
                    .map(|value| (number * increment + offset, value))
            })
        })
    }
    /// Visits occupied values in index order; `array.iter().sum::<i32>()`.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.entries().map(|(_, value)| value)
    }
    /// Visits occupied values exclusively; `array.iter_mut().for_each(|v| *v += 1)`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.blocks
            .values_mut()
            .flat_map(|block| block.iter_mut().filter_map(Option::as_mut))
    }
    /// Visits occupied slots exclusively with their indices, as the native non-const
    /// `Iterator` (`Key()` plus `ChangeValue()`); `array.entries_mut().for_each(|(i, v)| *v += i)`.
    pub fn entries_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        let increment = self.increment;
        self.blocks.iter_mut().flat_map(move |(&number, block)| {
            block
                .iter_mut()
                .enumerate()
                .filter_map(move |(offset, slot)| {
                    slot.as_mut()
                        .map(|value| (number * increment + offset, value))
                })
        })
    }
    /// Swaps contents and block geometry; `array.exchange(&mut other)`.
    pub fn exchange(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Data-map spelling of insertion; `array.bind(5, 55)`.
    pub fn bind(&mut self, index: usize, value: T) -> &mut T {
        self.set_value(index, value)
    }
    /// Data-map spelling of occupancy; `array.is_bound(5)`.
    pub fn is_bound(&self, index: usize) -> bool {
        self.has_value(index)
    }
    /// Data-map spelling of lookup; `array.find(5)`.
    pub fn find(&self, index: usize) -> Option<&T> {
        self.value(index)
    }
    /// Data-map spelling of exclusive lookup; `array.change_find(5)`.
    pub fn change_find(&mut self, index: usize) -> Option<&mut T> {
        self.change_value(index)
    }
    /// Data-map spelling of removal; `array.unbind(5)`.
    pub fn unbind(&mut self, index: usize) -> bool {
        self.unset_value(index)
    }
}
impl<T: Clone> SparseArray<T> {
    /// Replaces occupied slots, clearing obsolete destination slots; `destination.assign(&source)`.
    pub fn assign(&mut self, source: &Self) {
        if self.increment != source.increment {
            self.clear();
            self.increment = source.increment;
        }
        self.blocks
            .retain(|index, _| source.blocks.contains_key(index));
        for &index in source.blocks.keys() {
            self.reserve_block(index);
        }
        for (&index, source_block) in &source.blocks {
            let block = self.blocks.entry(index).or_insert_with(|| {
                std::iter::repeat_with(|| None)
                    .take(self.increment)
                    .collect()
            });
            block.clone_from_slice(source_block);
        }
        self.count = source.count;
    }
}
impl<T> std::ops::Index<usize> for SparseArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.value(index)
            .unwrap_or_else(|| panic!("SparseArray index {index}; expected an occupied slot"))
    }
}
impl<T> std::ops::IndexMut<usize> for SparseArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.change_value(index)
            .unwrap_or_else(|| panic!("SparseArray index {index}; expected an occupied slot"))
    }
}
