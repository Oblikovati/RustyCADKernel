//! Untyped stored order pointers keep payload lifetimes covariant without changing native layout.
#![allow(unsafe_code)]
use super::MapLinkNode;
#[derive(Default)]
#[repr(C)]
pub(super) struct OrderedNodeLinks {
    previous: *mut MapLinkNode,
    next: *mut MapLinkNode,
}
impl OrderedNodeLinks {
    pub(super) fn previous<N>(&self) -> *mut N {
        self.previous.cast()
    }
    pub(super) fn next<N>(&self) -> *mut N {
        self.next.cast()
    }
    pub(super) fn previous_mut<N>(&mut self) -> &mut *mut N {
        // SAFETY: all sized raw pointers have identical representation; this borrows only the
        // pointer field, never its target. Typed node access still requires caller validity.
        unsafe { &mut *std::ptr::from_mut(&mut self.previous).cast::<*mut N>() }
    }
    pub(super) fn next_mut<N>(&mut self) -> &mut *mut N {
        // SAFETY: same representation argument as previous_mut, with disjoint exclusive storage.
        unsafe { &mut *std::ptr::from_mut(&mut self.next).cast::<*mut N>() }
    }
}
macro_rules! ordered_node_links {
    ($node:ident<$($parameter:ident),+>) => {
        impl<$($parameter),+> $node<$($parameter),+> {
            /// Reads the borrowed insertion-order successor; `node.order_next().is_null()`.
            pub fn order_next(&self) -> *mut Self { self.order.next() }
            /// Reads the native const OrderNext overload; `node.order_next_const().is_null()`.
            pub fn order_next_const(&self) -> *const Self { self.order.next::<Self>().cast_const() }
            /// Reads native myOrderPrev without following it; `node.order_previous().is_null()`.
            pub fn order_previous(&self) -> *mut Self { self.order.previous() }
            /// Edits native myOrderPrev; `*node.order_previous_mut() = previous`.
            pub fn order_previous_mut(&mut self) -> &mut *mut Self { self.order.previous_mut() }
            /// Edits native myOrderNext; `*node.order_next_mut() = next`.
            pub fn order_next_mut(&mut self) -> &mut *mut Self { self.order.next_mut() }
        }
    };
}
pub(super) use ordered_node_links;
