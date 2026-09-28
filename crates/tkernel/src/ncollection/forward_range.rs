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

//! Adapts More/Next hosts with compile-checked Value > Current > CurrentId selection.
//! Rust hosts declare accessor descriptors explicitly because stable Rust has no method-name SFINAE.
mod accessors;
pub use accessors::{
    AccessorMetadata, AccessorPresence, AccessorSet, Missing, Present, ReadAccessor, SelectAccessor,
};
/// A cursor with stable borrowed or owned accessor output; implement `more` and `advance`.
pub trait ForwardHost: Sized {
    /// Declares its accessor methods in Value, Current, CurrentId order; `AccessorSet<Present<A>,Missing,Missing>`.
    type Accessors: SelectAccessor<Self>;
    /// Checks whether the current position exists; `host.more()`.
    fn more(&self) -> bool;
    /// Advances the host; `host.advance()`.
    fn advance(&mut self);
}
/// Selected reference or value output; `ForwardItem<MyHost>`.
pub type ForwardItem<I> = <<I as ForwardHost>::Accessors as SelectAccessor<I>>::Item;
/// Selected unqualified value type for snapshots; `ForwardValue<MyHost>`.
pub type ForwardValue<I> = <<I as ForwardHost>::Accessors as SelectAccessor<I>>::Value;
/// OCCT's input-iterator category, with no random-access guarantee; `InputIterator`.
pub struct InputIterator;
/// Legacy iterator type traits; `ForwardIteratorTraits::Difference` is `isize`.
pub trait ForwardIteratorTraits {
    /// Unqualified accessor value type; e.g. `i32` for `&i32` output.
    type Value;
    /// Native signed pointer difference type; `isize`.
    type Difference;
    /// Traversal category; `InputIterator`.
    type Category;
}
/// Exhaustion marker supporting comparisons in both directions; `iter==ForwardRangeSentinel`.
#[derive(Clone, Copy, Default)]
pub struct ForwardRangeSentinel;
/// Owned postfix snapshot that does not copy the host; `*iter.post_increment()?`.
pub struct PostfixProxy<T> {
    value: T,
}
impl<T> std::ops::Deref for PostfixProxy<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
/// A borrowed cursor adapter; `ForwardRangeIter::new(&mut host)`.
pub struct ForwardRangeIter<'a, I: ForwardHost> {
    host: &'a mut I,
}
/// A movable owning range; `for item in ForwardRange::new(host) {}`.
pub struct ForwardRange<I: ForwardHost> {
    host: I,
}
impl<I: ForwardHost> ForwardRange<I> {
    /// Moves a host into the range, including non-copyable hosts; `ForwardRange::new(host)`.
    pub fn new(host: I) -> Self {
        Self { host }
    }
    /// Borrows the current position without resetting it; `range.iter()`.
    pub fn iter(&mut self) -> ForwardRangeIter<'_, I> {
        ForwardRangeIter::new(&mut self.host)
    }
    /// Returns the end marker; `range.end()`.
    pub fn end(&self) -> ForwardRangeSentinel {
        ForwardRangeSentinel
    }
}
impl<I: ForwardHost + Clone> ForwardRange<I> {
    /// Copies a copyable host into an independent cursor; `ForwardRange::from_copy(&host)`.
    pub fn from_copy(host: &I) -> Self {
        Self::new(host.clone())
    }
}
impl<I: ForwardHost> Iterator for ForwardRange<I> {
    type Item = ForwardItem<I>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter().next()
    }
}
impl<'a, I: ForwardHost> ForwardRangeIter<'a, I> {
    /// Borrows a host at its current position; `ForwardRangeIter::new(&mut host)`.
    pub fn new(host: &'a mut I) -> Self {
        Self { host }
    }
    /// Reads without advancing; empty hosts yield None; `iter.current()`.
    pub fn current(&self) -> Option<ForwardItem<I>> {
        self.host.more().then(|| I::Accessors::read(self.host))
    }
    /// Exposes reference/value field access using Rust's field projection; `iter.arrow()?.x`.
    pub fn arrow(&self) -> Option<ForwardItem<I>> {
        self.current()
    }
    /// Advances a valid host; exhaustion remains safe; `iter.advance()`.
    pub fn advance(&mut self) {
        if self.host.more() {
            self.host.advance();
        }
    }
    /// Copies the current value and advances without copying the host; `iter.post_increment()`.
    pub fn post_increment(&mut self) -> Option<PostfixProxy<ForwardValue<I>>> {
        let value = I::Accessors::snapshot(self.current()?);
        self.advance();
        Some(PostfixProxy { value })
    }
}
impl<I: ForwardHost> Iterator for ForwardRangeIter<'_, I> {
    type Item = ForwardItem<I>;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.current()?;
        self.advance();
        Some(item)
    }
}
impl<I: ForwardHost> ForwardIteratorTraits for ForwardRangeIter<'_, I> {
    type Value = ForwardValue<I>;
    type Difference = isize;
    type Category = InputIterator;
}
impl<I: ForwardHost> PartialEq<ForwardRangeSentinel> for ForwardRangeIter<'_, I> {
    fn eq(&self, _other: &ForwardRangeSentinel) -> bool {
        !self.host.more()
    }
}
impl<I: ForwardHost> PartialEq<ForwardRangeIter<'_, I>> for ForwardRangeSentinel {
    fn eq(&self, other: &ForwardRangeIter<'_, I>) -> bool {
        !other.host.more()
    }
}
