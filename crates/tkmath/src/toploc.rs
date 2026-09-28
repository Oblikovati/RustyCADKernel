// Created on: 1991-01-21
// Created by: Christophe MARION
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of FoundationClasses/TKMath/TopLoc/TopLoc_Location.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Shared datum chains with eagerly computed transformations (OCCT OCC25545).
use crate::gp::Transform3;
use rustycad_tkernel::standard::Failure;
use std::sync::Arc;
/// A datum's identity is distinct from its numeric transform; `Datum3::new(transform)`.
#[derive(Debug)]
pub struct Datum3 {
    transform: Transform3,
}
impl Datum3 {
    /// Creates an independently identifiable coordinate system; `Datum3::new(transform)`.
    pub const fn new(transform: Transform3) -> Self {
        Self { transform }
    }
    /// Borrows the original transform; `datum.transformation()`.
    pub const fn transformation(&self) -> &Transform3 {
        &self.transform
    }
}
#[derive(Debug)]
struct LocationItem {
    datum: Arc<Datum3>,
    power: i32,
    tail: Location,
    cached: Transform3,
}
/// Immutable TopLoc_Location chain, safe for concurrent reads; `Location::default()`.
#[derive(Clone, Debug, Default)]
pub struct Location {
    head: Option<Arc<LocationItem>>,
}
impl Location {
    /// Allocates a new datum even for an identity matrix; `Location::new(transform)`.
    pub fn new(transform: Transform3) -> Self {
        Self::from_datum(Arc::new(Datum3::new(transform)))
    }
    /// Shares datum identity across locations; `Location::from_datum(datum.clone())`.
    pub fn from_datum(datum: Arc<Datum3>) -> Self {
        Self::default()
            .prepend(datum, 1)
            .expect("power one never inverts a datum")
    }
    /// Empty chains alone are identity locations; `Location::default().is_identity()`.
    pub const fn is_identity(&self) -> bool {
        self.head.is_none()
    }
    /// Returns the construction-time cached transformation; `location.transformation()`.
    pub fn transformation(&self) -> Transform3 {
        self.head
            .as_ref()
            .map_or_else(Transform3::default, |item| item.cached)
    }
    fn prepend(self, datum: Arc<Datum3>, power: i32) -> Result<Self, Failure> {
        let cached = self
            .transformation()
            .multiplied(datum.transform.powered(power)?);
        Ok(Self {
            head: Some(Arc::new(LocationItem {
                datum,
                power,
                tail: self,
                cached,
            })),
        })
    }
    /// Borrows the most recently prepended datum; `location.first_datum()`.
    pub fn first_datum(&self) -> Option<&Arc<Datum3>> {
        self.head.as_ref().map(|item| &item.datum)
    }
    /// Returns the first datum's exponent; `location.first_power()`.
    pub fn first_power(&self) -> Option<i32> {
        self.head.as_ref().map(|item| item.power)
    }
    /// Borrows the remaining chain; `location.next_location()`.
    pub fn next_location(&self) -> Option<&Self> {
        self.head.as_ref().map(|item| &item.tail)
    }
    /// Composes datum chains and cancels adjacent inverse powers; `left.multiplied(&right)`.
    pub fn multiplied(&self, other: &Self) -> Result<Self, Failure> {
        if self.is_identity() {
            return Ok(other.clone());
        }
        let mut items = Vec::new();
        let mut cursor = other;
        while let Some(item) = &cursor.head {
            items.push(item);
            cursor = &item.tail;
        }
        let mut result = self.clone();
        for item in items.into_iter().rev() {
            result = result.append_item(item)?;
        }
        Ok(result)
    }
    fn append_item(mut self, item: &LocationItem) -> Result<Self, Failure> {
        let mut power = item.power;
        if let Some(head) = &self.head
            && Arc::ptr_eq(&head.datum, &item.datum)
        {
            power += head.power;
            self = head.tail.clone();
        }
        if power == 0 {
            Ok(self)
        } else {
            self.prepend(item.datum.clone(), power)
        }
    }
    /// Reverses datum order and negates exponents; `location.inverted()`.
    pub fn inverted(&self) -> Result<Self, Failure> {
        let mut result = Self::default();
        let mut cursor = self;
        while let Some(item) = &cursor.head {
            result = result.prepend(item.datum.clone(), -item.power)?;
            cursor = &item.tail;
        }
        Ok(result)
    }
}
impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        let (mut left, mut right) = (self, other);
        while let (Some(a), Some(b)) = (&left.head, &right.head) {
            if !Arc::ptr_eq(&a.datum, &b.datum) || a.power != b.power {
                return false;
            }
            (left, right) = (&a.tail, &b.tail);
        }
        left.is_identity() && right.is_identity()
    }
}
impl Eq for Location {}
