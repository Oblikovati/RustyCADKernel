//! Martensite owned shape/drawer specialization backed by the existing native-order DataMap.
use super::{
    AisColoredDrawer,
    shape_drawer_node::{ShapeDrawerNode, ShapeDrawerNodeHasher},
};
use rustycad_tkbrep::{topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkernel::{
    ncollection::{DataMap, HashPolicy},
    standard::{Failure, Handle, HandleReference, SharedState},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
pub(super) type ShapeDrawerNodeTable = DataMap<Arc<ShapeDrawerNode>, (), ShapeDrawerNodeHasher>;
/// Owned native `NCollection_DataMap<TopoDS_Shape,Handle<AIS_ColoredDrawer>,TopTools_ShapeMapHasher>` semantics.
/// Example: `let map=AisShapeDrawerMap::default(); map.bind(&shape, &drawer)?;`.
#[derive(Default)]
pub struct AisShapeDrawerMap {
    pub(super) nodes: SharedState<ShapeDrawerNodeTable>,
    // OCCT 3d097a0 BaseMap::destroy72–97 resets mySize after destructors.
    // ChainedSlots::clear26–30 takes payload before destruction; its borrowed
    // length cannot represent mySize after unlocking for callbacks. This actual
    // scalar lets valid destructor queries run without holding the node-table guard.
    extent: AtomicUsize,
}
impl AisShapeDrawerMap {
    /// Observes the actual binding count; `map.len()`.
    pub fn len(&self) -> usize {
        // Native scalar reads remain valid while unlocked child destructors run.
        self.extent.load(Ordering::Relaxed)
    }
    /// Checks the owned container, including later Clear operations; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        let extent = self.len();
        extent == 0
    }
    /// Checks native IsSame equality, independently of orientation; `map.is_bound(&shape)`.
    pub fn is_bound(&self, key: &Shape) -> bool {
        let nodes = self.nodes.read();
        find_shape_drawer_node(&nodes, key).is_some()
    }
    /// Binds or replaces a value while retaining the first stored key and node.
    /// Example: `map.bind(&shape.reversed(), &replacement)?`.
    pub fn bind(&self, key: &Shape, value: &Handle<AisColoredDrawer>) -> Result<bool, Failure> {
        // The borrowed-slot path also serves native same-map expansion without owning input copies.
        self.bind_reference(key, value)
    }
    pub(super) fn bind_reference(
        &self,
        key: &Shape,
        value: &impl HandleReference<AisColoredDrawer>,
    ) -> Result<bool, Failure> {
        let mut nodes = self.nodes.write();
        prepare_shape_drawer_binding(&mut nodes);
        if let Some(node) = find_shape_drawer_node(&nodes, key) {
            let node = Arc::clone(node);
            drop(nodes);
            node.replace_value(value)?;
            return Ok(false);
        }
        let inserted = nodes.bind(Arc::new(ShapeDrawerNode::new(key, value)), ());
        self.set_native_extent(nodes.len());
        Ok(inserted)
    }
    /// Removes the actual node and releases its key/value despite retained aliases.
    /// Example: `map.unbind(&shape)`.
    pub fn unbind(&self, key: &Shape) -> bool {
        let mut nodes = self.nodes.write();
        let Some(node) = find_shape_drawer_node(&nodes, key).cloned() else {
            return false;
        };
        let removed = nodes.unbind(&node);
        self.set_native_extent(nodes.len());
        drop(nodes);
        node.release_fields();
        removed
    }
    /// Clears bindings in native bucket order, optionally releasing bucket storage.
    /// Example: `map.clear(true)`.
    pub fn clear(&self, release: bool) {
        let mut nodes = self.nodes.write();
        let removed: Vec<_> = nodes.items().map(|(node, ())| Arc::clone(node)).collect();
        nodes.clear(release);
        drop(nodes);
        for node in removed {
            node.release_fields();
        }
        self.set_native_extent(0);
    }
    fn set_native_extent(&self, extent: usize) {
        // All visible count/empty/container views read this same native scalar.
        self.extent.store(extent, Ordering::Relaxed);
    }
    /// Rehashes existing nodes without changing their stored key/value locations.
    /// Example: `map.resize(1000)`.
    pub fn resize(&self, buckets: usize) {
        let mut nodes = self.nodes.write();
        nodes.resize(buckets);
    }
    /// Observes the real native bucket count; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        let nodes = self.nodes.read();
        nodes.bucket_count()
    }
}
impl Drop for AisShapeDrawerMap {
    fn drop(&mut self) {
        // Owning-map destruction must release fields even if a temporary storage lease survives.
        self.clear(true);
    }
}
pub(super) fn find_shape_drawer_node<'a>(
    nodes: &'a ShapeDrawerNodeTable,
    key: &Shape,
) -> Option<&'a Arc<ShapeDrawerNode>> {
    let hash = ShapeMapHasher.hash(key);
    nodes
        .find_hashed(hash, |node| {
            node.read_fields(|live| ShapeMapHasher.equal(live.key, key))
                .expect("ShapeDrawerNode fields=None; expected a live mapped key")
        })
        .map(|(node, ())| node)
}

fn prepare_shape_drawer_binding(nodes: &mut ShapeDrawerNodeTable) {
    // OCCT 3d097a0 emplaceImpl rehashes before lookup, including replacement of an existing key.
    if nodes.is_empty() || nodes.len() > nodes.bucket_count() {
        let extent = nodes.len();
        nodes.resize(extent);
    }
}
