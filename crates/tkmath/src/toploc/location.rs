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
// Rust translation of FoundationClasses/TKMath/TopLoc/TopLoc_Location.cxx and
// TopLoc_Location.lxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::{Datum3, ItemLocation, SListOfItemLocation};
use crate::gp::Transform3;
use rustycad_tkernel::{
    ncollection::HashKey,
    standard::{
        Failure, Handle,
        dump::{self, DumpOutput, DumpStream},
        hash_bytes,
    },
};
use std::io::{self, Write};
/// `TopLoc_Location`: an immutable chain of powered datums, safe for concurrent reads; `Location::default()`.
#[derive(Clone, Default)]
pub struct Location {
    items: SListOfItemLocation,
}
impl Location {
    /// Allocates a new datum even for an identity matrix; `Location::new(transform)`.
    pub fn new(transform: Transform3) -> Self {
        Self::from_datum(Handle::new(Datum3::new(transform)))
    }
    /// Shares datum identity across locations; `Location::from_datum(datum.clone())`.
    pub fn from_datum(datum: Handle<Datum3>) -> Self {
        let mut location = Self::default();
        // Power one never inverts the datum, so gp_Trsf::Powered cannot fail here.
        let item = ItemLocation::new(datum, 1).expect("power one never inverts a datum");
        location.items.construct(item);
        location
    }
    pub(super) const fn from_items(items: SListOfItemLocation) -> Self {
        Self { items }
    }
    pub(super) const fn items(&self) -> &SListOfItemLocation {
        &self.items
    }
    pub(super) const fn items_mut(&mut self) -> &mut SListOfItemLocation {
        &mut self.items
    }
    /// Empty chains alone are identity locations; `Location::default().is_identity()`.
    pub const fn is_identity(&self) -> bool {
        self.items.is_empty()
    }
    /// Resets to the identity, as `Identity`; `location.identity()`.
    pub fn identity(&mut self) {
        self.items.clear();
    }
    /// Clears the item list, as `Clear`; `location.clear()`.
    pub fn clear(&mut self) {
        self.items.clear();
    }
    /// Borrows the most recently prepended datum, or fails with `Standard_NoSuchObject`; `location.first_datum()?`.
    pub fn first_datum(&self) -> Result<&Handle<Datum3>, Failure> {
        Ok(&self.items.value()?.datum)
    }
    /// Returns the first datum's exponent, or fails with `Standard_NoSuchObject`; `location.first_power()?`.
    pub fn first_power(&self) -> Result<i32, Failure> {
        Ok(self.items.value()?.power)
    }
    /// Borrows the chain without its first item; the identity is its own tail; `location.next_location()`.
    pub fn next_location(&self) -> &Self {
        self.items.node().map_or(self, |node| node.tail_location())
    }
    /// Returns the cached product of every datum power; `location.transformation()`.
    pub fn transformation(&self) -> Transform3 {
        self.items
            .value()
            .map_or_else(|_| Transform3::default(), |item| item.trsf)
    }
    /// Hashes datum addresses and powers with OCCT's Murmur chain; the identity hashes to zero.
    /// Addresses make the value specific to one process, as natively; `location.hash_code()`.
    pub fn hash_code(&self) -> usize {
        let mut items = Vec::new();
        let mut cursor = &self.items;
        while let Ok(item) = cursor.value() {
            items.push((item.datum.as_ptr() as usize, item.power));
            cursor = cursor.tail();
        }
        Self::hash_items(&items)
    }
    /// The HashCode recurrence over `(datum address, power)` pairs, head first;
    /// `Location::hash_items(&[(0x1000, 1)])`.
    pub fn hash_items(items: &[(usize, i32)]) -> usize {
        if items.is_empty() {
            return 0;
        }
        let mut hash: usize = if cfg!(target_pointer_width = "64") {
            0xA329_F1D3_A586_u64 as usize
        } else {
            0xc70f_6907
        };
        for (address, power) in items {
            let power_hash = hash_bytes(&power.to_ne_bytes());
            let combined = [*address, power_hash, hash];
            let bytes: Vec<u8> = combined
                .iter()
                .flat_map(|word| word.to_ne_bytes())
                .collect();
            hash = hash_bytes(&bytes);
        }
        hash
    }
    /// Same datums (by identity) with the same powers, in order; `left.is_equal(&right)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        if self.items.same_node(&other.items) {
            return true;
        }
        let (mut left, mut right) = (&self.items, &other.items);
        while let (Ok(a), Ok(b)) = (left.value(), right.value()) {
            if a.datum != b.datum || a.power != b.power {
                return false;
            }
            (left, right) = (left.tail(), right.tail());
        }
        left.is_empty() && right.is_empty()
    }
    /// Negation of [`Location::is_equal`]; `left.is_different(&right)`.
    pub fn is_different(&self, other: &Self) -> bool {
        !self.is_equal(other)
    }
    /// OCCT's scale comparison precision; `Location::scale_prec() == 1e-14`.
    pub const fn scale_prec() -> f64 {
        1.0e-14
    }
    /// Writes the transformation and the identity flag; `location.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> io::Result<()> {
        dump::dump_class_begin(output, "TopLoc_Location")?;
        if depth != 0 {
            let mut field = DumpStream::default();
            self.transformation().dump_json(&mut field)?;
            dump::dump_key_to_class(output, "Transformation", field.text())?;
        }
        dump::dump_integer(output, "IsIdentity", i32::from(self.is_identity()))
    }
    /// Prints each exponent and datum, head first; `location.shallow_dump(&mut out)?`.
    pub fn shallow_dump(&self, output: &mut impl Write) -> io::Result<()> {
        write!(output, "TopLoc_Location : ")?;
        if self.is_identity() {
            writeln!(output, "Identity")?;
        }
        let mut cursor = &self.items;
        while let Ok(item) = cursor.value() {
            writeln!(output)?;
            writeln!(output, "       Exponent : {}", item.power)?;
            item.datum.shallow_dump(output)?;
            cursor = cursor.tail();
        }
        writeln!(output)
    }
}
impl From<&Location> for Transform3 {
    fn from(location: &Location) -> Self {
        location.transformation()
    }
}
impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl Eq for Location {}
impl std::hash::Hash for Location {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}
impl HashKey for Location {
    fn occt_hash(&self) -> usize {
        self.hash_code()
    }
}
impl std::fmt::Debug for Location {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = formatter.debug_list();
        let mut cursor = &self.items;
        while let Ok(item) = cursor.value() {
            list.entry(&(item.datum.as_ptr(), item.power));
            cursor = cursor.tail();
        }
        list.finish()
    }
}
