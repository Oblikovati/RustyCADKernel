//! Native access delegates without consulting More, even for custom exhausted hosts.
use super::{
    ForwardHost, ForwardItem, ForwardRangeIter, ForwardValue, PostfixProxy, SelectAccessor,
};

/// Owns native ArrowProxy's by-value result; `ForwardArrowProxy::new(value).arrow()`.
pub struct ForwardArrowProxy<T> {
    /// Public native myValue storage; `proxy.value`.
    pub value: T,
}

impl<T> ForwardArrowProxy<T> {
    /// Moves a value into proxy storage; `ForwardArrowProxy::new(value)`.
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Returns native const operator-> identity; `proxy.arrow() == &proxy.value`.
    pub fn arrow(&self) -> *const T {
        std::ptr::from_ref(&self.value)
    }
}

impl<T> std::ops::Deref for ForwardArrowProxy<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<I: ForwardHost> ForwardRangeIter<'_, I> {
    /// Calls the selected native accessor unconditionally; `iter.dereference()`.
    /// Hosts define whether exhausted access returns a value or raises an error.
    pub fn dereference(&self) -> ForwardItem<I> {
        self.host.read(I::Accessors::read)
    }

    /// Captures the native value then calls Next, without consulting More.
    /// Example: `let old = iter.postfix_increment(); assert_eq!(*old, expected)`.
    /// The existing post_increment method remains the checked Rust adapter.
    pub fn postfix_increment(&mut self) -> PostfixProxy<ForwardValue<I>> {
        let value = I::Accessors::snapshot(self.dereference());
        self.pre_increment();
        PostfixProxy { value }
    }
}
