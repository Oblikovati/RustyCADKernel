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

//! Accessor descriptors replace C++ SFINAE; priority and reference/value types remain compile checked.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::forward_range::*;
use std::any::TypeId;
use std::marker::PhantomData;
#[derive(Clone)]
struct MockHost<'a, T, P> {
    values: &'a [T],
    size: usize,
    index: usize,
    policy: PhantomData<P>,
}
impl<'a, T, P> MockHost<'a, T, P> {
    fn new(values: &'a [T], size: usize) -> Self {
        Self {
            values,
            size,
            index: 0,
            policy: PhantomData,
        }
    }
}
trait MockPolicy<'a, T: 'a>: Sized {
    type Accessors: SelectAccessor<MockHost<'a, T, Self>>;
}
impl<'a, T, P: MockPolicy<'a, T>> ForwardHost for MockHost<'a, T, P> {
    type Accessors = P::Accessors;
    fn more(&self) -> bool {
        self.index < self.size
    }
    fn advance(&mut self) {
        self.index += 1;
    }
}
struct ReferenceAccessor;
impl<'a, T: Clone, P> ReadAccessor<MockHost<'a, T, P>> for ReferenceAccessor {
    type Item = &'a T;
    type Value = T;
    fn read(host: &MockHost<'a, T, P>) -> Self::Item {
        &host.values[host.index]
    }
    fn snapshot(item: Self::Item) -> T {
        item.clone()
    }
}
struct ValueAccessor;
impl<'a, T: Clone, P> ReadAccessor<MockHost<'a, T, P>> for ValueAccessor {
    type Item = T;
    type Value = T;
    fn read(host: &MockHost<'a, T, P>) -> T {
        host.values[host.index].clone()
    }
    fn snapshot(item: T) -> T {
        item
    }
}
struct IndexAccessor;
impl<T, P> ReadAccessor<MockHost<'_, T, P>> for IndexAccessor {
    type Item = i32;
    type Value = i32;
    fn read(host: &MockHost<'_, T, P>) -> i32 {
        host.index as i32
    }
    fn snapshot(item: i32) -> i32 {
        item
    }
}
macro_rules! policy {
    ($name:ident,$value:ty,$current:ty,$id:ty) => {
        #[derive(Clone)]
        struct $name;
        impl<'a, T: Clone + 'a> MockPolicy<'a, T> for $name {
            type Accessors = AccessorSet<$value, $current, $id>;
        }
    };
}
policy!(ValueOnly, Present<ReferenceAccessor>, Missing, Missing);
policy!(CurrentOnly, Missing, Present<ReferenceAccessor>, Missing);
policy!(IdOnly, Missing, Missing, Present<ValueAccessor>);
policy!(
    ValueAndCurrent,
    Present<ReferenceAccessor>,
    Present<ReferenceAccessor>,
    Missing
);
policy!(
    CurrentAndId,
    Missing,
    Present<ReferenceAccessor>,
    Present<IndexAccessor>
);
policy!(Integrated, Missing, Present<ReferenceAccessor>, Missing);
policy!(PointValue, Missing, Present<ValueAccessor>, Missing);
struct NonCopyable;
impl<'a, T: Clone + 'a> MockPolicy<'a, T> for NonCopyable {
    type Accessors = AccessorSet<Missing, Present<ValueAccessor>, Missing>;
}
type MockValueIterator<'a> = MockHost<'a, i32, ValueOnly>;
type MockCurrentIterator<'a> = MockHost<'a, i32, CurrentOnly>;
type MockCurrentIdIterator<'a> = MockHost<'a, i32, IdOnly>;
type MockBothValueAndCurrent<'a> = MockHost<'a, i32, ValueAndCurrent>;
type MockBothCurrentAndId<'a> = MockHost<'a, i32, CurrentAndId>;
type MockNonCopyableIterator<'a> = MockHost<'a, i32, NonCopyable>;
type MockWithBeginEnd<'a> = MockHost<'a, i32, Integrated>;
impl<'a> IntoIterator for MockWithBeginEnd<'a> {
    type Item = &'a i32;
    type IntoIter = ForwardRange<Self>;
    fn into_iter(self) -> Self::IntoIter {
        ForwardRange::new(self)
    }
}
impl<'a, 'b> IntoIterator for &'b mut MockWithBeginEnd<'a> {
    type Item = &'a i32;
    type IntoIter = ForwardRangeIter<'b, MockWithBeginEnd<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        ForwardRangeIter::new(self)
    }
}
#[derive(Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}
type MockPointIterator<'a> = MockHost<'a, Point, ValueOnly>;
type MockPointByValueIterator<'a> = MockHost<'a, Point, PointValue>;
fn flags<I: ForwardHost>() -> (bool, bool, bool) {
    (
        <I::Accessors as AccessorMetadata>::HAS_VALUE,
        <I::Accessors as AccessorMetadata>::HAS_CURRENT,
        <I::Accessors as AccessorMetadata>::HAS_CURRENT_ID,
    )
}
include!("ncollection_forward_range/cases.rs");
