//! Link-only writes retain payload provenance; unlink tokens permit rollback after user hash panic.
use super::{DoubleMapNode, DoubleStorage, MapLinkNode};
pub(in crate::ncollection::double_map) struct UnlinkedDoubleNode {
    bucket: usize,
    previous: *mut MapLinkNode,
    target: *mut MapLinkNode,
    secondary: bool,
}
impl<A, B> DoubleStorage<A, B> {
    pub(in crate::ncollection::double_map) unsafe fn next(
        node: *mut MapLinkNode,
        secondary: bool,
    ) -> *mut MapLinkNode {
        if secondary {
            unsafe { (*node.cast::<DoubleMapNode<A, B>>()).next2_value().cast() }
        } else {
            unsafe { (*node).next() }
        }
    }
    pub(in crate::ncollection::double_map) unsafe fn set_next(
        node: *mut MapLinkNode,
        next: *mut MapLinkNode,
        secondary: bool,
    ) {
        if secondary {
            unsafe { DoubleMapNode::<A, B>::set_second_link(node.cast(), next.cast()) };
        } else {
            unsafe { *(*node).next_mut() = next };
        }
    }
    pub(in crate::ncollection::double_map) fn unlink(
        &mut self,
        bucket: usize,
        target: *mut MapLinkNode,
        secondary: bool,
    ) -> Option<UnlinkedDoubleNode> {
        let previous = self.previous_link(bucket, target, secondary)?;
        let token = UnlinkedDoubleNode {
            bucket,
            previous,
            target,
            secondary,
        };
        self.publish_link(&token, unsafe { Self::next(target, secondary) });
        Some(token)
    }
    fn previous_link(
        &self,
        bucket: usize,
        target: *mut MapLinkNode,
        secondary: bool,
    ) -> Option<*mut MapLinkNode> {
        let (mut previous, mut node) = (std::ptr::null_mut(), self.head(bucket, secondary));
        while !node.is_null() && node != target {
            previous = node;
            node = unsafe { Self::next(node, secondary) };
        }
        if node.is_null() {
            return None;
        }
        Some(previous)
    }
    pub(in crate::ncollection::double_map) fn restore(&mut self, token: UnlinkedDoubleNode) {
        self.publish_link(&token, token.target);
    }
    fn publish_link(&mut self, token: &UnlinkedDoubleNode, next: *mut MapLinkNode) {
        self.topology.edit(|base| {
            if !token.previous.is_null() {
                unsafe { Self::set_next(token.previous, next, token.secondary) };
                return;
            }
            if token.secondary {
                base.second_slots_mut()[token.bucket] = next;
            } else {
                base.bucket_slots_mut()[token.bucket] = next;
            }
        });
    }
}
