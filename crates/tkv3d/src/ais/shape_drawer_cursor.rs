//! Guard-free live bucket cursor: same-map expansion sees later inserted heads without key/value copies.
use super::{AisShapeDrawerMap, shape_drawer_node::ShapeDrawerNode};
use rustycad_tkernel::standard::{Failure, exceptions::no_such_object};
use std::sync::{Arc, Weak};
pub(super) struct ShapeDrawerNativeCursor<'a> {
    owner: &'a AisShapeDrawerMap,
    bucket: usize,
    table_identity: *const (),
    table_length: usize,
    current: Option<Weak<ShapeDrawerNode>>,
}
impl<'a> ShapeDrawerNativeCursor<'a> {
    pub(super) fn new(owner: &'a AisShapeDrawerMap) -> Result<Self, Failure> {
        let mut cursor = Self {
            owner,
            bucket: 0,
            table_identity: std::ptr::null(),
            table_length: 0,
            current: None,
        };
        let nodes = owner.nodes.read();
        if let Some(table) = nodes.buckets() {
            cursor.table_identity = table.identity();
            cursor.table_length = table.len();
        }
        drop(nodes);
        cursor.find_next_head()?;
        Ok(cursor)
    }
    pub(super) fn current(&self) -> Result<Option<Arc<ShapeDrawerNode>>, Failure> {
        // Owning metadata does not retain erased key/value fields; existing lifecycle proofs cover it.
        self.current
            .as_ref()
            .map(|node| node.upgrade().ok_or_else(|| cursor_changed(self.bucket)))
            .transpose()
    }
    pub(super) fn next(&mut self) -> Result<(), Failure> {
        if self.current.is_none() {
            return Ok(());
        }
        self.current()?;
        if self.follow_current_chain()? {
            return Ok(());
        }
        self.bucket += 1;
        self.find_next_head()
    }
    fn follow_current_chain(&mut self) -> Result<bool, Failure> {
        let nodes = self.owner.nodes.read();
        let table = nodes.buckets().ok_or_else(|| cursor_changed(self.bucket))?;
        self.check_table(table.identity(), table.len())?;
        let mut chain = table.chain(self.bucket)?;
        let found = chain
            .by_ref()
            .any(|(node, ())| Weak::ptr_eq(&Arc::downgrade(node), self.current.as_ref().unwrap()));
        if !found {
            return Err(cursor_changed(self.bucket));
        }
        self.current = chain.next().map(|(node, ())| Arc::downgrade(node));
        Ok(self.current.is_some())
    }
    fn find_next_head(&mut self) -> Result<(), Failure> {
        let nodes = self.owner.nodes.read();
        let Some(table) = nodes.buckets() else {
            return Ok(());
        };
        self.check_table(table.identity(), table.len())?;
        while self.bucket < self.table_length {
            if let Some((node, ())) = table.chain(self.bucket)?.next() {
                self.current = Some(Arc::downgrade(node));
                return Ok(());
            }
            self.bucket += 1;
        }
        self.current = None;
        Ok(())
    }
    fn check_table(&self, identity: *const (), length: usize) -> Result<(), Failure> {
        // OCCT 3d097a0 BasicIterator caches bucket/end pointers; rehash invalidates native traversal.
        if identity != self.table_identity || length != self.table_length {
            return Err(cursor_changed(self.bucket));
        }
        Ok(())
    }
}
fn cursor_changed(bucket: usize) -> Failure {
    no_such_object(&format!(
        "ShapeDrawer cursor bucket={bucket}; expected its original live node and bucket table"
    ))
}
