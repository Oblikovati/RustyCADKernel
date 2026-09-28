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

//! Typed allocator traits, rebinding and actual allocator-backed containers.
#![allow(non_snake_case, clippy::approx_constant)]
use rustycad_tkernel::ncollection::{AllocatorTypes, BaseAllocator, OccAllocator};
use std::{mem::size_of, sync::Arc};
#[test]
fn NCollection_OccAllocatorTest__AllocatorTypeTraits() {
    // The source's nullable object handle only exercises the generic value type here.
    type Element = Option<Arc<()>>;
    type Alloc = OccAllocator<Element>;
    assert_eq!(
        size_of::<<Alloc as AllocatorTypes>::Value>(),
        size_of::<Element>()
    );
    assert_eq!(
        size_of::<<Alloc as AllocatorTypes>::Pointer>(),
        size_of::<*mut ()>()
    );
    assert_eq!(
        size_of::<<Alloc as AllocatorTypes>::ConstPointer>(),
        size_of::<*const ()>()
    );
    assert_eq!(
        size_of::<<Alloc as AllocatorTypes>::Size>(),
        size_of::<usize>()
    );
    assert_eq!(
        size_of::<<Alloc as AllocatorTypes>::Difference>(),
        size_of::<isize>()
    );
    let mut dummy: Element = None;
    let address = &dummy as *const Element;
    let reference: <Alloc as AllocatorTypes>::Reference<'_> = &mut dummy;
    assert_eq!(reference as *const Element, address);
    let constant: <Alloc as AllocatorTypes>::ConstReference<'_> = &dummy;
    assert_eq!(constant as *const Element, address);
    let rebound: OccAllocator<i32> = Alloc::default().rebind();
    assert_eq!(rebound, OccAllocator::<i32>::default());
    assert_eq!(
        size_of::<<OccAllocator<i32> as AllocatorTypes>::Value>(),
        size_of::<i32>()
    );
}
#[test]
fn NCollection_OccAllocatorTest__STLContainerIntegration() {
    let arena = BaseAllocator::incremental(12 * 1024);
    let typed = OccAllocator::<i32>::new(arena.clone());
    let mut list = typed.list();
    list.push_back(2);
    assert_eq!(list.len(), 1);
    assert_eq!(list.front(), Some(&2));
    let chars = OccAllocator::<u8>::default();
    let mut vector = chars.rebind::<i32>().vector();
    vector.push(1);
    assert_eq!(vector.len(), 1);
    assert_eq!(vector[0], 1);
    let pointers = OccAllocator::<*mut ()>::default();
    let mut second = pointers.rebind::<i32>().vector();
    second.resize(10, 0);
    second.push(-1);
    assert_eq!(second.len(), 11);
    assert_eq!(second.last(), Some(&-1));
    assert_ne!(typed, chars);
    assert_eq!(typed, OccAllocator::<i32>::new(arena));
}
#[test]
fn NCollection_OccAllocatorTest__DefaultAllocator() {
    let mut vector = OccAllocator::<i32>::default().vector();
    for i in 0..100 {
        vector.push(i);
    }
    assert_eq!(vector.len(), 100);
    for (i, value) in vector.iter().enumerate() {
        assert_eq!(*value, i as i32);
    }
}
#[test]
fn NCollection_OccAllocatorTest__AllocatorComparison() {
    let first = BaseAllocator::incremental(12 * 1024);
    let second = BaseAllocator::incremental(12 * 1024);
    let a = OccAllocator::<i32>::new(first.clone());
    let b = OccAllocator::<i32>::new(first);
    let c = OccAllocator::<i32>::new(second);
    let d = OccAllocator::<i32>::default();
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, d);
    assert!(!(a != b));
    assert!(a != c);
}
#[test]
fn NCollection_OccAllocatorTest__CrossTypeCompatibility() {
    let arena = BaseAllocator::incremental(12 * 1024);
    let ints = OccAllocator::<i32>::new(arena.clone());
    let doubles = OccAllocator::<f64>::new(arena.clone());
    let _chars = OccAllocator::<u8>::new(arena.clone());
    assert_eq!(ints, OccAllocator::<i32>::new(arena));
    let mut integers = ints.vector();
    let mut reals = doubles.vector();
    integers.push(42);
    reals.push(3.14);
    assert_eq!(integers.len(), 1);
    assert_eq!(reals.len(), 1);
    assert_eq!(integers[0], 42);
    assert_eq!(reals[0], 3.14);
}
