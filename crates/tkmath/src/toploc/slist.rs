// Created on: 1993-02-26
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/TopLoc/TopLoc_SListOfItemLocation.cxx and
// TopLoc_SListNodeOfItemLocation.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{ItemLocation, Location};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
/// `TopLoc_SListNodeOfItemLocation`: one shared cell of a location list; `SListNodeOfItemLocation::new(item, &tail)`.
pub struct SListNodeOfItemLocation {
    base: StandardTransient,
    // The tail is kept as a Location so that Location::NextLocation can lend it, as the
    // native code reinterprets the tail list as a TopLoc_Location.
    tail: Location,
    value: ItemLocation,
}
rustycad_tkernel::impl_transient!(
    SListNodeOfItemLocation,
    "TopLoc_SListNodeOfItemLocation",
    StandardTransient,
    base
);
impl SListNodeOfItemLocation {
    /// Stores the item and shares the tail; `SListNodeOfItemLocation::new(item, &tail)`.
    pub fn new(item: ItemLocation, tail: &SListOfItemLocation) -> Self {
        Self {
            base: StandardTransient::default(),
            tail: Location::from_items(tail.clone()),
            value: item,
        }
    }
    /// Borrows the shared remainder of the list; `node.tail()`.
    pub fn tail(&self) -> &SListOfItemLocation {
        self.tail.items()
    }
    /// Borrows the stored item; `node.value()`.
    pub const fn value(&self) -> &ItemLocation {
        &self.value
    }
    pub(super) const fn tail_location(&self) -> &Location {
        &self.tail
    }
}
/// `TopLoc_SListOfItemLocation`: a persistent singly linked list with shared tails; `SListOfItemLocation::default()`.
#[derive(Clone, Default)]
pub struct SListOfItemLocation {
    node: Option<Handle<SListNodeOfItemLocation>>,
}
impl SListOfItemLocation {
    /// Prepends an item, caching `tail transformation * item transformation` in it;
    /// `SListOfItemLocation::new(item, &tail)`.
    pub fn new(mut item: ItemLocation, tail: &Self) -> Self {
        if let Some(head) = &tail.node {
            item.trsf.pre_multiply(head.value.trsf);
        }
        Self {
            node: Some(Handle::new(SListNodeOfItemLocation::new(item, tail))),
        }
    }
    /// Shares another list, as `Assign`; `list.assign(&other)`.
    pub fn assign(&mut self, other: &Self) {
        self.node.clone_from(&other.node);
    }
    /// Tests for the empty list; `list.is_empty()`.
    pub const fn is_empty(&self) -> bool {
        self.node.is_none()
    }
    /// Drops this reference to the list; `list.clear()`.
    pub fn clear(&mut self) {
        self.node = None;
    }
    /// Borrows the head item or fails with `Standard_NoSuchObject`; `list.value()?`.
    pub fn value(&self) -> Result<&ItemLocation, Failure> {
        self.node
            .as_deref()
            .map(SListNodeOfItemLocation::value)
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_NoSuchObject",
                    "TopLoc_SListOfItemLocation::Value: empty list; expected at least one item",
                    "",
                )
            })
    }
    /// Borrows the tail; the empty list is its own tail; `list.tail()`.
    pub fn tail(&self) -> &Self {
        self.node
            .as_deref()
            .map_or(self, SListNodeOfItemLocation::tail)
    }
    /// Replaces the list by one headed by `item`; `list.construct(item)`.
    pub fn construct(&mut self, item: ItemLocation) {
        *self = Self::new(item, self);
    }
    /// Replaces the list by its tail; `list.to_tail()`.
    pub fn to_tail(&mut self) {
        *self = self.tail().clone();
    }
    /// Iterator view: true while an item remains; `list.more()`.
    pub const fn more(&self) -> bool {
        !self.is_empty()
    }
    /// Iterator view: moves to the tail, as `ToTail`; `list.next()`.
    pub fn next(&mut self) {
        self.to_tail();
    }
    pub(super) fn node(&self) -> Option<&SListNodeOfItemLocation> {
        self.node.as_deref()
    }
    pub(super) fn same_node(&self, other: &Self) -> bool {
        match (&self.node, &other.node) {
            (Some(left), Some(right)) => left == right,
            (None, None) => true,
            _ => false,
        }
    }
}
