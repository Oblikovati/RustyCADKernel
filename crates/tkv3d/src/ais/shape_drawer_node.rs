//! Allocated specialization nodes own each native key/value once and release them on removal.
use super::AisColoredDrawer;
use rustycad_tkbrep::{topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkernel::{
    ncollection::HashPolicy,
    standard::{Failure, Handle, HandleFieldRef, HandleReference, SharedState},
};
use std::{fmt, sync::Arc};
pub(super) struct ShapeDrawerNodeFields {
    pub key: Shape,
}
pub(super) struct ShapeDrawerBorrowedFields<'a> {
    pub key: &'a Shape,
    pub drawer: &'a Handle<AisColoredDrawer>,
}
pub(super) struct ShapeDrawerNode {
    pub fields: SharedState<Option<ShapeDrawerNodeFields>>,
    // Native immutable Key references may coexist with assignment to the actual Value slot.
    drawer: SharedState<Option<Handle<AisColoredDrawer>>>,
}
impl ShapeDrawerNode {
    pub(super) fn new(key: &Shape, value: &impl HandleReference<AisColoredDrawer>) -> Self {
        // OCCT 3d097a0 DataMap constructs the handle-valued base before copying the original key.
        let drawer = value.clone_handle();
        let key = key.clone();
        Self {
            fields: SharedState::new(Some(ShapeDrawerNodeFields { key })),
            drawer: SharedState::new(Some(drawer)),
        }
    }
    pub(super) fn read_fields<R>(
        &self,
        inspect: impl FnOnce(&ShapeDrawerBorrowedFields<'_>) -> R,
    ) -> Result<R, Failure> {
        let fields = self.fields.read();
        let live = fields.as_ref().ok_or_else(removed_shape_drawer_node)?;
        let drawer = self.drawer.read();
        let value = drawer.as_ref().ok_or_else(removed_shape_drawer_node)?;
        Ok(inspect(&ShapeDrawerBorrowedFields {
            key: &live.key,
            drawer: value,
        }))
    }
    pub(super) fn with_entry<R>(
        &self,
        inspect: impl FnOnce(
            &Shape,
            &HandleFieldRef<'_, Option<Handle<AisColoredDrawer>>, AisColoredDrawer>,
        ) -> R,
    ) -> Result<R, Failure> {
        let fields = self.fields.read();
        let live = fields.as_ref().ok_or_else(removed_shape_drawer_node)?;
        let value = HandleFieldRef::new(&self.drawer, live_shape_drawer_value);
        Ok(inspect(&live.key, &value))
    }
    pub(super) fn release_fields(&self) {
        // OCCT 3d097a0 removal must not let storage leases delay key/value ownership release on Clear/UnBind.
        let removed_key = self.fields.write().take();
        let removed_value = self.drawer.write().take();
        drop(removed_key);
        drop(removed_value);
    }
    pub(super) fn replace_value(
        &self,
        value: &impl HandleReference<AisColoredDrawer>,
    ) -> Result<(), Failure> {
        let replacement = value.clone_handle();
        let removed = self.exchange_value(replacement)?;
        drop(removed);
        Ok(())
    }
    fn exchange_value(
        &self,
        replacement: Handle<AisColoredDrawer>,
    ) -> Result<Handle<AisColoredDrawer>, Failure> {
        let mut value = self.drawer.write();
        let live = value.as_mut().ok_or_else(removed_shape_drawer_node)?;
        Ok(std::mem::replace(live, replacement))
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

fn live_shape_drawer_value(slot: &Option<Handle<AisColoredDrawer>>) -> &Handle<AisColoredDrawer> {
    // This internal borrow spans a live iterator node; erasing that node invalidates native references.
    slot.as_ref()
        .expect("ShapeDrawerNode value=None; expected a live iterator value slot")
}
