// Created on: 2007-09-04
// Created by: Andrey BETENEV
// Copyright (c) 2007-2014 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Atomic.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_Atomic: the deprecated full-barrier integer operations.
//!
//! The GCC branch uses `__sync_*` builtins, which are sequentially consistent and wrap
//! on overflow; `AtomicI32` with `SeqCst` gives the same values and ordering on every
//! platform, so the per-platform branches of the header collapse into one.

use std::sync::atomic::{AtomicI32, Ordering};

/// `Standard_Atomic_Increment(volatile int*)`: adds one and returns the new value (wrapping).
///
/// ```
/// use std::sync::atomic::AtomicI32;
/// let counter = AtomicI32::new(1);
/// assert_eq!(rustycad_tkernel::standard::atomic::atomic_increment(&counter), 2);
/// ```
pub fn atomic_increment(value: &AtomicI32) -> i32 {
    value.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
}

/// `Standard_Atomic_Decrement(volatile int*)`: subtracts one and returns the new value (wrapping).
///
/// ```
/// use std::sync::atomic::AtomicI32;
/// let counter = AtomicI32::new(1);
/// assert_eq!(rustycad_tkernel::standard::atomic::atomic_decrement(&counter), 0);
/// ```
pub fn atomic_decrement(value: &AtomicI32) -> i32 {
    value.fetch_sub(1, Ordering::SeqCst).wrapping_sub(1)
}

/// `Standard_Atomic_CompareAndSwap(volatile int*, int, int)`: stores `new_value` when the
/// current value is `old_value`, and returns whether it did.
///
/// ```
/// use std::sync::atomic::AtomicI32;
/// let counter = AtomicI32::new(5);
/// assert!(rustycad_tkernel::standard::atomic::atomic_compare_and_swap(&counter, 5, 9));
/// ```
pub fn atomic_compare_and_swap(value: &AtomicI32, old_value: i32, new_value: i32) -> bool {
    value
        .compare_exchange(old_value, new_value, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}
