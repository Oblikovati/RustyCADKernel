//! Allocated specialization nodes own each native key/value once and release them on removal.
use super::AisColoredDrawer;
use rustycad_tkbrep::{topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkernel::{
    ncollection::HashPolicy,
    standard::{Failure, Handle, SharedState},
};
use std::{fmt, sync::Arc};
pub(super) struct ShapeDrawerNodeFields {
    pub key: Shape,
    pub drawer: Handle<AisColoredDrawer>,
}
pub(super) struct ShapeDrawerNode {
    pub fields: SharedState<Option<ShapeDrawerNodeFields>>,
}
impl ShapeDrawerNode {
    pub(super) fn new(key: &Shape, value: &Handle<AisColoredDrawer>) -> Self {
        // OCCT 3d097a0 DataMap constructs the handle-valued base before copying the original key.
        let drawer = value.clone();
        let key = key.clone();
        Self {
            fields: SharedState::new(Some(ShapeDrawerNodeFields { key, drawer })),
        }
    }
    pub(super) fn read_fields<R>(
        &self,
        inspect: impl FnOnce(&ShapeDrawerNodeFields) -> R,
    ) -> Result<R, Failure> {
        let fields = self.fields.read();
        let live = fields.as_ref().ok_or_else(removed_shape_drawer_node)?;
        Ok(inspect(live))
    }
    pub(super) fn release_fields(&self) {
        // OCCT 3d097a0 removal must not let storage leases delay key/value ownership release on Clear/UnBind.
        let removed = self.fields.write().take();
        drop(removed);
    }
    pub(super) fn replace_value(&self, value: &Handle<AisColoredDrawer>) -> Result<(), Failure> {
        let replacement = value.clone();
        let removed = self.exchange_value(replacement)?;
        drop(removed);
        Ok(())
    }
    fn exchange_value(
        &self,
        replacement: Handle<AisColoredDrawer>,
    ) -> Result<Handle<AisColoredDrawer>, Failure> {
        let mut fields = self.fields.write();
        let live = fields.as_mut().ok_or_else(removed_shape_drawer_node)?;
        Ok(std::mem::replace(&mut live.drawer, replacement))
    }
}
impl fmt::Debug for ShapeDrawerNode {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = self.fields.read();
        output
            .debug_struct("ShapeDrawerNode")
            .field("key", &fields.as_ref().map(|live| &live.key))
            .finish()
    }
}
#[derive(Clone, Copy, Default)]
pub(super) struct ShapeDrawerNodeHasher;
impl HashPolicy<Arc<ShapeDrawerNode>> for ShapeDrawerNodeHasher {
    fn hash(&self, node: &Arc<ShapeDrawerNode>) -> usize {
        let fields = node.fields.read();
        let live = fields
            .as_ref()
            .expect("ShapeDrawerNode fields=None; expected a bound live key");
        ShapeMapHasher.hash(&live.key)
    }
    fn equal(&self, first: &Arc<ShapeDrawerNode>, second: &Arc<ShapeDrawerNode>) -> bool {
        if Arc::ptr_eq(first, second) {
            return true;
        }
        let first = first.fields.read();
        let second = second.fields.read();
        let left = first
            .as_ref()
            .expect("ShapeDrawerNode fields=None; expected a bound live key");
        let right = second
            .as_ref()
            .expect("ShapeDrawerNode fields=None; expected a bound live key");
        ShapeMapHasher.equal(&left.key, &right.key)
    }
}
pub(super) fn removed_shape_drawer_node() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "ShapeDrawerNode fields=None; expected a live native key/value reference",
        "",
    )
}
