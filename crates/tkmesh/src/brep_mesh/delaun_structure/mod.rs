//! Indexed planar mesh connectivity, retaining OCCT's deletion and compaction order.
use super::{
    BRepMeshEdge, BRepMeshTriangle, BRepMeshVertex, BRepMeshVertexTool, DegreeOfFreedom,
    PairOfIndex,
};
use crate::brep_mesh_data::MeshAllocator;
use rustycad_tkernel::{
    impl_transient,
    ncollection::{
        BaseAllocator, DataMap, DefaultHashPolicy, DynamicArray, HashKey, IndexedDataMap, List, Map,
    },
    standard::{Failure, Handle, SharedState, StandardTransient},
};

mod compact;
mod dump;
mod elements;
mod links;

// Native RemoveLink mutates a const map key without rehashing it. Movability
// changes equality but not the endpoint hash (OCCT 3d097a0328e71).
#[derive(Debug)]
struct MeshLinkKey(SharedState<BRepMeshEdge>);
impl PartialEq for MeshLinkKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.snapshot() == other.0.snapshot()
    }
}
impl HashKey for MeshLinkKey {
    fn occt_hash(&self) -> usize {
        self.0.snapshot().occt_hash()
    }
}
struct DelaunConnectivity {
    node_links: DataMap<i32, List<i32>>,
    links: IndexedDataMap<MeshLinkKey, PairOfIndex>,
    deleted_links: List<i32>,
    elements: DynamicArray<BRepMeshTriangle>,
    element_domain: Map<i32>,
    link_domain: Map<i32>,
}
/// Shared mesh topology with native one-based indices; `BRepMeshDataStructureOfDelaun::new(&allocator,100)`.
pub struct BRepMeshDataStructureOfDelaun {
    transient: StandardTransient,
    allocator: MeshAllocator,
    nodes: Handle<BRepMeshVertexTool>,
    connectivity: SharedState<DelaunConnectivity>,
}
impl_transient!(
    BRepMeshDataStructureOfDelaun,
    "BRepMesh_DataStructureOfDelaun",
    StandardTransient,
    transient
);
impl BRepMeshDataStructureOfDelaun {
    /// Reserve native maps and element blocks; `BRepMeshDataStructureOfDelaun::new(&allocator,100)`.
    pub fn new(allocator: &MeshAllocator, reserved_nodes: i32) -> Self {
        Self {
            transient: StandardTransient::default(),
            allocator: allocator.clone(),
            nodes: Handle::new(BRepMeshVertexTool::new(allocator)),
            connectivity: SharedState::new(DelaunConnectivity::new(allocator, reserved_nodes)),
        }
    }
    /// Return allocated node slots, including deletion marks; `mesh.nb_nodes()`.
    pub fn nb_nodes(&self) -> i32 {
        self.nodes.extent()
    }
    /// Retain the allocator identity; `mesh.allocator()`.
    pub fn allocator(&self) -> &MeshAllocator {
        &self.allocator
    }
    /// Retain the live spatial node tool; `mesh.vertex_tool().set_tolerance(1e-7)`.
    pub fn vertex_tool(&self) -> &Handle<BRepMeshVertexTool> {
        &self.nodes
    }
    /// Find a coincident vertex using the spatial inspector; `mesh.index_of_node(&vertex)?`.
    pub fn index_of_node(&self, vertex: &BRepMeshVertex) -> Result<i32, Failure> {
        self.nodes.find_index(vertex)
    }
    /// Read a one-based node slot; `mesh.get_node(1)?`.
    pub fn get_node(&self, index: i32) -> Result<BRepMeshVertex, Failure> {
        self.nodes.find_key(index)
    }
    /// Add a node, initializing connectivity only on its first use; `mesh.add_node(&vertex,false)?`.
    pub fn add_node(&self, vertex: &BRepMeshVertex, force: bool) -> Result<i32, Failure> {
        let index = self.nodes.add(vertex, force)?;
        let mut state = self.connectivity.write();
        if !state.node_links.is_bound(&index) {
            state
                .node_links
                .bind(index, List::with_allocator(resource(&self.allocator)));
        }
        Ok(index)
    }
    /// Reject any already indexed replacement, including the current slot; `mesh.substitute_node(1,&vertex)?`.
    pub fn substitute_node(&self, index: i32, vertex: &BRepMeshVertex) -> Result<bool, Failure> {
        if self.nodes.find_index(vertex)? != 0 {
            return Ok(false);
        }
        self.nodes.substitute(index, vertex)?;
        Ok(true)
    }
    /// Mark an unattached free or forced node deleted; `mesh.remove_node(1,false)?`.
    pub fn remove_node(&self, index: i32, force: bool) -> Result<(), Failure> {
        if (force || self.get_node(index)?.movability() == DegreeOfFreedom::Free)
            && self.connectivity.read().node_links.find(&index)?.is_empty()
        {
            self.nodes.delete_vertex(index)?;
        }
        Ok(())
    }
    /// Copy attached links in insertion order; `mesh.links_connected_to(1)?`.
    pub fn links_connected_to(&self, index: i32) -> Result<List<i32>, Failure> {
        Ok(self.connectivity.read().node_links.find(&index)?.clone())
    }
}
impl DelaunConnectivity {
    fn new(allocator: &MeshAllocator, reserved: i32) -> Self {
        let arena = resource(allocator);
        let buckets = reserved.wrapping_mul(3).max(1) as usize;
        Self {
            node_links: DataMap::with_allocator(DefaultHashPolicy, buckets, arena.clone()),
            links: IndexedDataMap::with_allocator(DefaultHashPolicy, buckets, arena.clone()),
            deleted_links: List::with_allocator(arena.clone()),
            elements: DynamicArray::with_allocator(
                reserved.wrapping_mul(2).max(1) as usize,
                Some(arena),
            ),
            element_domain: Map::default(),
            link_domain: Map::default(),
        }
    }
}
fn resource(allocator: &MeshAllocator) -> BaseAllocator {
    allocator
        .get()
        .map(|a| a.resource().clone())
        .unwrap_or_else(BaseAllocator::common)
}
