// Copyright (c) 2002-2024 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/NCollection/NCollection_HArray1.hxx for
// `double` (the deprecated alias TColStd_HArray1OfReal) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `NCollection_HArray1<double>`, a bounded real array shared through handles.
//!
//! OCCT hands such an array out as an `occ::handle` whose edits every holder sees (the
//! Poly polygons share their parameters this way), so this transient keeps one
//! [`Array1`] in a [`SharedState`] and every member takes `&self`. Only the `double`
//! instantiation is ported: each transient needs its own runtime type descriptor.
use super::Array1;
use crate::standard::{Failure, Handle, SharedState, StandardTransient};
/// A shared, bounded array of reals; `HArray1OfReal::with_bounds(1, 3)?`.
pub struct HArray1OfReal {
    base: StandardTransient,
    values: SharedState<Array1<'static, f64>>,
}
crate::impl_transient!(
    HArray1OfReal,
    "NCollection_HArray1<double>",
    StandardTransient,
    base
);
impl HArray1OfReal {
    /// Allocates zeros at `lower..=upper`; `HArray1OfReal::with_bounds(1, 3)?`.
    pub fn with_bounds(lower: i32, upper: i32) -> Result<Self, Failure> {
        let values =
            Array1::with_bounds(lower, upper).map_err(|e| e.into_failure("NCollection_HArray1"))?;
        Ok(Self::from_array(values))
    }
    /// Takes ownership of an array and its bounds; `HArray1OfReal::from_array(values)`.
    pub fn from_array(values: Array1<'static, f64>) -> Self {
        Self {
            base: StandardTransient::default(),
            values: SharedState::new(values),
        }
    }
    /// Wraps a new shared array in a handle; `HArray1OfReal::shared(values)`.
    pub fn shared(values: Array1<'static, f64>) -> Handle<Self> {
        Handle::new(Self::from_array(values))
    }
    /// Lower bound; `array.lower()`.
    pub fn lower(&self) -> i32 {
        self.values.read().lower()
    }
    /// Upper bound; `array.upper()`.
    pub fn upper(&self) -> i32 {
        self.values.read().upper()
    }
    /// Number of values; `array.len()`.
    pub fn len(&self) -> usize {
        self.values.read().len()
    }
    /// True without values; `array.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.values.read().is_empty()
    }
    /// Reads one value; `array.value(1)?`.
    pub fn value(&self, index: i32) -> Result<f64, Failure> {
        self.values
            .read()
            .value(index)
            .copied()
            .map_err(|e| e.into_failure("NCollection_HArray1::Value"))
    }
    /// Writes one value; `array.set_value(1, 0.5)?`.
    pub fn set_value(&self, index: i32, value: f64) -> Result<(), Failure> {
        self.values
            .write()
            .set_value(index, value)
            .map_err(|e| e.into_failure("NCollection_HArray1::SetValue"))
    }
    /// Copy of the wrapped array, as `Array1()`; `array.array1()`.
    pub fn array1(&self) -> Array1<'static, f64> {
        self.values.read().clone()
    }
    /// Lends the wrapped array for editing, as `ChangeArray1()`; `array.change_array1(|a| a.init(0.0))`.
    pub fn change_array1<R>(&self, edit: impl FnOnce(&mut Array1<'static, f64>) -> R) -> R {
        edit(&mut self.values.write())
    }
}
