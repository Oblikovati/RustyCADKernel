//! Adapts More/Next hosts with compile-checked Value > Current > CurrentId selection.
//! Rust hosts declare accessor descriptors explicitly because stable Rust has no method-name SFINAE.
mod accessors;
mod host_binding;
pub use accessors::{
    AccessorMetadata, AccessorPresence, AccessorSet, Missing, Present, ReadAccessor, SelectAccessor,
};
use host_binding::HostBinding;
use std::cell::RefCell;
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
    host: HostBinding<'a, I>,
}
/// A movable owning range; `for item in ForwardRange::new(host) {}`.
pub struct ForwardRange<I: ForwardHost> {
    host: RefCell<I>,
}
impl<I: ForwardHost> ForwardRange<I> {
    /// Moves a host into the range, including non-copyable hosts; `ForwardRange::new(host)`.
    pub fn new(host: I) -> Self {
        Self {
            host: RefCell::new(host),
        }
    }
    /// Borrows the current position without resetting it; `range.iter()`.
    pub fn iter(&mut self) -> ForwardRangeIter<'_, I> {
        ForwardRangeIter::new(self.host.get_mut())
    }
    /// Borrows the shared cursor at its current position; `range.begin().next()`.
    /// Multiple adapters advance the same host, matching OCCT's const `begin()`.
    pub fn begin(&self) -> ForwardRangeIter<'_, I> {
        ForwardRangeIter {
            host: HostBinding::Shared(&self.host),
        }
    }
    /// Borrows the same advancing cursor as `begin`; `range.cbegin().next()`.
    pub fn cbegin(&self) -> ForwardRangeIter<'_, I> {
        self.begin()
    }
    /// Returns the end marker; `range.end()`.
    pub fn end(&self) -> ForwardRangeSentinel {
        ForwardRangeSentinel
    }
    /// Returns the const-range exhaustion marker; `range.cbegin() == range.cend()`.
    pub fn cend(&self) -> ForwardRangeSentinel {
        self.end()
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
        Self {
            host: HostBinding::Exclusive(host),
        }
    }
    /// Reads without advancing; empty hosts yield None; `iter.current()`.
    pub fn current(&self) -> Option<ForwardItem<I>> {
        self.host
            .read(|host| host.more().then(|| I::Accessors::read(host)))
    }
    /// Exposes reference/value field access using Rust's field projection; `iter.arrow()?.x`.
    pub fn arrow(&self) -> Option<ForwardItem<I>> {
        self.current()
    }
    /// Advances a valid host; exhaustion remains safe; `iter.advance()`.
    pub fn advance(&mut self) {
        self.host.change(|host| {
            if host.more() {
                host.advance();
            }
        });
    }
    /// Calls the host's Next even at exhaustion and returns this adapter; `iter.pre_increment()`.
    /// OCCT delegates prefix increment unconditionally; `advance` is the checked alternative.
    pub fn pre_increment(&mut self) -> &mut Self {
        self.host.change(ForwardHost::advance);
        self
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
        self.host.read(|host| !host.more())
    }
}
impl<I: ForwardHost> PartialEq<ForwardRangeIter<'_, I>> for ForwardRangeSentinel {
    fn eq(&self, other: &ForwardRangeIter<'_, I>) -> bool {
        other == self
    }
}
