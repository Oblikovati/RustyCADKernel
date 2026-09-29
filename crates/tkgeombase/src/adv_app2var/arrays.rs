// Copyright (c) 1996-1999 Matra Datavision
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

//! The `NCollection_HArray1` allocations of the AdvApp2Var classes: 1-based arrays
//! whose failures name the member that allocated them.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// A zero-filled array `1..=size` (`new NCollection_HArray1<T>(1, size)`, which is
/// uninitialised natively for doubles); `one_based(3, "AdvApp2Var_Patch")?`.
pub(super) fn one_based<T: Default>(
    size: i32,
    member: &str,
) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, size).map_err(|error| error.into_failure(member))
}

/// The array `1..=values.len()` holding `values`; `one_based_from(&[-1.0, 1.0])`.
pub(super) fn one_based_from<T: Default + Copy>(values: &[T]) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_size(values.len());
    array.update_lower_bound(1);
    array.as_mut_slice().copy_from_slice(values);
    array
}

/// The copy of an optional table (`HArray->Array1()`), or a Standard_NullObject naming
/// what is missing; `required(iso.som_tab(), "AdvApp2Var_Patch: iso sums")?`.
pub(super) fn required<T>(value: Option<T>, what: &str) -> Result<T, Failure> {
    value.ok_or_else(|| {
        Failure::with_message("Standard_NullObject", &format!("{what}: null handle"))
    })
}
