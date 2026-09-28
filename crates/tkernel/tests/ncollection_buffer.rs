// Copyright (c) 2026 OPEN CASCADE SAS
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

//! Empty buffer iterators must delimit an empty range even with an allocator.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
#[test]
fn NCollection_BufferTest__EmptyRange() {
    let mut buffer = Buffer::new(Some(BaseAllocator::common()), 0);
    let bounds = buffer.as_mut_slice().as_mut_ptr_range();
    assert_eq!(bounds.start, bounds.end);
    let constant = buffer.as_slice().as_ptr_range();
    assert_eq!(constant.start, constant.end);
    assert_eq!(buffer.as_slice().iter().count(), 0);
}
