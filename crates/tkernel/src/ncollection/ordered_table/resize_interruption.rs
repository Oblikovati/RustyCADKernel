//! A consumed map can retain native partial rehash without exposing invalid safe traversal.
use super::*;
use crate::ncollection::BaseMapResize;
use std::{
    any::Any,
    collections::HashSet,
    panic::{AssertUnwindSafe, catch_unwind},
};

type ResizeCompletion<T> = (T, Box<dyn Any + Send>);
type ResizeClearResult<T, E> = Result<ResizeCompletion<T>, Box<E>>;
pub(in crate::ncollection) struct ResizeInterruption<K, V, H, N: StorageNode<K, V>> {
    table: Option<OrderedTable<K, V, H, N>>,
    replacement: Option<BaseMapResize>,
    original_links: Vec<(*mut MapLinkNode, *mut MapLinkNode)>,
    panic: Option<Box<dyn Any + Send>>,
}
impl<K, V, H: HashPolicy<K>, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    pub fn resize_with_progress(
        self,
        requested: usize,
    ) -> Result<Self, Box<ResizeInterruption<K, V, H, N>>> {
        let mut replacement = self
            .base
            .begin_resize(requested)
            .unwrap_or_else(|error| panic!("{error}"));
        if !replacement.is_resize() {
            return Ok(self);
        }
        let original_links = self.original_links();
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.relink(&original_links, &mut replacement)
        }));
        self.finish_captured_resize(replacement, original_links, result)
    }
    fn finish_captured_resize(
        mut self,
        replacement: BaseMapResize,
        original_links: Vec<(*mut MapLinkNode, *mut MapLinkNode)>,
        result: Result<(), Box<dyn Any + Send>>,
    ) -> Result<Self, Box<ResizeInterruption<K, V, H, N>>> {
        if let Err(panic) = result {
            return Err(Box::new(ResizeInterruption {
                table: Some(self),
                replacement: Some(replacement),
                original_links,
                panic: Some(panic),
            }));
        }
        self.bucket_count = replacement.bucket_count();
        self.base.end_resize(replacement);
        Ok(self)
    }
}
impl<K, V, H, N: StorageNode<K, V>> ResizeInterruption<K, V, H, N> {
    pub fn table(&self) -> &OrderedTable<K, V, H, N> {
        self.table
            .as_ref()
            .expect("interruption table: expected retained owner")
    }
    pub fn replacement(&self) -> &BaseMapResize {
        self.replacement
            .as_ref()
            .expect("interruption buckets: expected retained replacement")
    }
    pub fn panic_payload(&self) -> &(dyn Any + Send) {
        &**self
            .panic
            .as_ref()
            .expect("interruption panic: expected original payload")
    }
    pub fn node_pointers(&self) -> Vec<*mut N> {
        self.original_links
            .iter()
            .map(|(node, _)| node.cast())
            .collect()
    }
    pub fn recover(mut self) -> ResizeCompletion<OrderedTable<K, V, H, N>> {
        OrderedTable::<K, V, H, N>::restore_bucket_links(&self.original_links);
        drop(self.replacement.take());
        (self.table.take().unwrap(), self.panic.take().unwrap())
    }
    pub fn native_clear(
        mut self: Box<Self>,
        release: bool,
    ) -> ResizeClearResult<OrderedTable<K, V, H, N>, Self> {
        let Some(reached) = self.certified_reachable() else {
            return Err(self);
        };
        let mut table = self.table.take().unwrap();
        self.abandon_replacement();
        table.abandon_unreachable(&reached);
        table.clear(release);
        Ok((table, self.panic.take().unwrap()))
    }
    fn certified_reachable(&self) -> Option<HashSet<usize>> {
        let table = self.table();
        let mut reached = HashSet::new();
        for &head in table.base.bucket_slots() {
            let mut node = head;
            while !node.is_null() {
                if !table.identities.contains_key(&node.addr()) || !reached.insert(node.addr()) {
                    return None;
                }
                // SAFETY: registry membership proves this node is a still-live allocation we own.
                node = unsafe { (*node).next() };
            }
        }
        Some(reached)
    }
    fn abandon_replacement(&mut self) {
        std::mem::forget(self.replacement.take());
    }
}
impl<K, V, H, N: StorageNode<K, V>> Drop for ResizeInterruption<K, V, H, N> {
    fn drop(&mut self) {
        self.abandon_replacement();
        let Some(mut table) = self.table.take() else {
            return;
        };
        table.contain_resize_nodes();
        // Only detached base/policy teardown remains; H::drop may unwind without touching K/V.
        drop(table);
    }
}
impl<K, V, H, N: StorageNode<K, V>> OrderedTable<K, V, H, N> {
    fn abandon_unreachable(&self, reached: &HashSet<usize>) {
        for node in self.slots.iter().flatten() {
            if !reached.contains(&node.addr()) {
                // SAFETY: these exact owned nodes become native raw orphans without payload destruction.
                unsafe {
                    self.resource().relinquish_typed_to_raw();
                }
            }
        }
    }
    fn contain_resize_nodes(&mut self) {
        for &node in self.slots.iter().flatten() {
            // SAFETY: neutralize owned order links before releasing any lease that permits arena reset.
            unsafe {
                *N::previous(node) = std::ptr::null_mut();
                *N::next(node) = std::ptr::null_mut();
            }
        }
        self.abandon_incomplete_copy();
        self.finish_clear(false);
    }
}
