// Copyright (c) 2025 OPEN CASCADE SAS
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

//! OCC22980 retained at its original 2^24 operation count.
#![allow(non_snake_case)]
use rustycad_tkernel::osd::Parallel;
use std::sync::atomic::{AtomicI32, Ordering};
#[test]
fn Standard_AtomicTest__OCC22980_AtomicOperations() {
    let sum = AtomicI32::new(0);
    assert_eq!(sum.fetch_sub(1, Ordering::SeqCst) - 1, -1);
    assert_eq!(sum.fetch_add(1, Ordering::SeqCst) + 1, 0);
    assert_eq!(sum.fetch_add(1, Ordering::SeqCst) + 1, 1);
    assert_eq!(sum.fetch_add(1, Ordering::SeqCst) + 1, 2);
    sum.store(0, Ordering::SeqCst);
    const COUNT: usize = 1 << 24;
    Parallel::for_range(
        0,
        COUNT as i32,
        |_| {
            sum.fetch_add(1, Ordering::SeqCst);
        },
        false,
    )
    .unwrap();
    assert_eq!(sum.load(Ordering::SeqCst), COUNT as i32);
    Parallel::for_range(
        0,
        COUNT as i32,
        |_| {
            sum.fetch_sub(1, Ordering::SeqCst);
        },
        false,
    )
    .unwrap();
    assert_eq!(sum.load(Ordering::SeqCst), 0);
}
