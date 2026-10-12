//! Native identity-keyed compound grouping: allocated nodes plus the actual dense index pointer table.
use super::{
    AisColoredDrawer,
    colored_compound_node::{ColoredCompoundHasher, ColoredCompoundNode},
};
use rustycad_tkbrep::topods::Compound;
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle, SharedState},
};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicUsize, Ordering},
};
pub(super) type ColoredCompoundTable = DataMap<Arc<ColoredCompoundNode>, (), ColoredCompoundHasher>;
/// Owned native drawer-to-compound indexed-map specialization; `let map=AisColoredDrawerCompounds::default()`.
#[derive(Default)]
pub struct AisColoredDrawerCompounds {
    pub(super) nodes: SharedState<ColoredCompoundTable>,
    pub(super) indexes: SharedState<Vec<Weak<ColoredCompoundNode>>>,
    // Native BaseMap resets Size after unlocked key/value destruction, not when payload is detached.
    extent: AtomicUsize,
}
/// Native protected `DataMapOfDrawerCompd` specialization; `let groups = AisColoredShapeDrawerCompounds::default();`.
pub type AisColoredShapeDrawerCompounds = AisColoredDrawerCompounds;
impl AisColoredDrawerCompounds {
    /// Read native one-based index extent; `map.len()`.
    pub fn len(&self) -> usize {
        // Scalar inspection remains valid inside native child destructor callbacks.
        self.extent.load(Ordering::Relaxed)
    }
    /// Read current emptiness; `map.is_empty()`.
    pub fn is_empty(&self) -> bool {
        let extent = self.len();
        extent == 0
    }
    /// Add without replacing an existing value; `map.add(&drawer,&compound)?`.
    pub fn add(
        &self,
        drawer: &Handle<AisColoredDrawer>,
        compound: &Compound,
    ) -> Result<usize, Failure> {
        let mut nodes = self.nodes.write();
        prepare_colored_compound_insert(&mut nodes);
        if let Some(node) = find_colored_compound(&nodes, drawer) {
            return self.index_of(node);
        }
        let node = Arc::new(ColoredCompoundNode::new(drawer, compound));
        self.indexes.write().push(Arc::downgrade(&node));
        nodes.bind(node, ());
        self.extent.store(nodes.len(), Ordering::Relaxed);
        Ok(nodes.len())
    }
    fn index_of(&self, node: &Arc<ColoredCompoundNode>) -> Result<usize, Failure> {
        self.indexes
            .read()
            .iter()
            .position(|slot| Weak::ptr_eq(slot, &Arc::downgrade(node)))
            .map(|position| position + 1)
            .ok_or_else(super::colored_compound_node::colored_compound_removed)
    }
    /// Clear actual owned fields before resetting native Size; `map.clear(false)`.
    pub fn clear(&self, release: bool) {
        let nodes = self.nodes.read();
        let removed: Vec<_> = nodes.items().map(|(node, ())| Arc::clone(node)).collect();
        drop(nodes);
        for node in removed {
            node.release();
        }
        self.nodes.write().clear(release);
        self.indexes.write().clear();
        self.extent.store(0, Ordering::Relaxed);
    }
    /// Rehash without relocating native Key/Value fields; `map.resize(1000)`.
    pub fn resize(&self, buckets: usize) {
        let mut nodes = self.nodes.write();
        nodes.resize(buckets);
    }
    /// Read native bucket count; `map.bucket_count()`.
    pub fn bucket_count(&self) -> usize {
        let nodes = self.nodes.read();
        nodes.bucket_count()
    }
}
impl Drop for AisColoredDrawerCompounds {
    fn drop(&mut self) {
        // Safe node metadata aliases must not retain fields beyond the owning map.
        self.clear(true);
    }
}
pub(super) fn find_colored_compound<'a>(
    nodes: &'a ColoredCompoundTable,
    drawer: &Handle<AisColoredDrawer>,
) -> Option<&'a Arc<ColoredCompoundNode>> {
    let address = drawer.object_address() as usize;
    nodes
        .find_hashed(address, |node| {
            node.drawer.object_address() == drawer.object_address()
        })
        .map(|(node, ())| node)
}
fn prepare_colored_compound_insert(nodes: &mut ColoredCompoundTable) {
    // Native Add rehashes before checking duplicates, even on a duplicate-key hit.
    if nodes.is_empty() || nodes.len() > nodes.bucket_count() {
        nodes.resize(nodes.len());
    }
}
