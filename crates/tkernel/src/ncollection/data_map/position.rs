//! Native iteration follows one link chain, then scans actual bucket pointers.
use crate::ncollection::{BaseMapStorage, MapLinkNode};
#[derive(Clone, Copy, Default)]
pub(super) struct DataMapPosition {
    pub(super) node: *mut MapLinkNode,
    bucket: usize,
}
impl DataMapPosition {
    pub(super) fn new(base: &BaseMapStorage) -> Self {
        let mut result = Self::default();
        result.seek(base);
        result
    }
    fn seek(&mut self, base: &BaseMapStorage) {
        for (bucket, &node) in base.bucket_slots().iter().enumerate().skip(self.bucket) {
            if !node.is_null() {
                self.bucket = bucket;
                self.node = node;
                return;
            }
        }
        self.node = std::ptr::null_mut();
    }
    pub(super) fn advance(&mut self, base: &BaseMapStorage) {
        if self.node.is_null() {
            return;
        }
        self.node = unsafe { (*self.node).next() };
        if !self.node.is_null() {
            return;
        }
        self.bucket += 1;
        self.seek(base);
    }
}
