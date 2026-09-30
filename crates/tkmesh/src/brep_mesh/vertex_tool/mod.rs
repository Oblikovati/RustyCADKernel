//! Native spatial uniqueness tool using the NCollection cell filter.
use super::{BRepMeshVertex, DegreeOfFreedom};
use crate::brep_mesh_data::MeshAllocator;
use rustycad_tkernel::{
    impl_transient,
    ncollection::{BaseAllocator, CellFilter, Shared},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::gp::Xy;
mod inspector;
mod operations;
pub use inspector::{
    BRepMeshVertexInspector, DeletedVertexIndices, MeshVertexCollection, MeshVertexCollectionSlot,
    VertexInspectorPoint, VertexInspectorTarget,
};
struct VertexToolState {
    filter: CellFilter<BRepMeshVertexInspector>,
    selector: BRepMeshVertexInspector,
    tolerance: [f64; 2],
}
/// Native indexed UV uniqueness tool; `BRepMeshVertexTool::new(&allocator)`.
pub struct BRepMeshVertexTool {
    transient: StandardTransient,
    allocator: MeshAllocator,
    state: SharedState<VertexToolState>,
}
impl_transient!(
    BRepMeshVertexTool,
    "BRepMesh_VertexTool",
    StandardTransient,
    transient
);
impl BRepMeshVertexTool {
    /// Initialize native confusion-sized cells and two-axis tolerance; `BRepMeshVertexTool::new(&allocator)`.
    pub fn new(allocator: &MeshAllocator) -> Self {
        let tolerance = rustycad_tkernel::precision::confusion();
        let mut selector = BRepMeshVertexInspector::new(allocator);
        selector.set_tolerances(tolerance, tolerance);
        Self {
            transient: StandardTransient::default(),
            allocator: allocator.clone(),
            state: SharedState::new(VertexToolState {
                filter: CellFilter::with_allocator(
                    tolerance + 0.05 * tolerance,
                    allocator.get().map(|a| a.resource().clone()),
                ),
                selector,
                tolerance: [tolerance; 2],
            }),
        }
    }
    /// Reset uniform cells before clearing vertices; `tool.set_cell_size(0.1)?`.
    pub fn set_cell_size(&self, size: f64) -> Result<(), Failure> {
        let mut state = self.state.write();
        state
            .filter
            .reset(size, self.allocator.get().map(|a| a.resource().clone()))?;
        state.selector.clear();
        Ok(())
    }
    /// Retain native one-based size-array bounds when resetting cells; `tool.set_cell_sizes(0.1,0.2)?`.
    pub fn set_cell_sizes(&self, first: f64, second: f64) -> Result<(), Failure> {
        let mut sizes = rustycad_tkernel::ncollection::Array1::with_bounds(1, 2)
            .map_err(|e| e.into_failure("BRepMesh_VertexTool"))?;
        sizes.as_mut_slice().copy_from_slice(&[first, second]);
        let mut state = self.state.write();
        state
            .filter
            .reset_sizes(&sizes, self.allocator.get().map(|a| a.resource().clone()));
        state.selector.clear();
        Ok(())
    }
    /// Set radial coincidence tolerance and symmetric insertion bounds; `tool.set_tolerance(1e-7)`.
    pub fn set_tolerance(&self, tolerance: f64) {
        let mut state = self.state.write();
        state.selector.set_tolerance(tolerance);
        state.tolerance = [tolerance; 2];
    }
    /// Set independent coincidence thresholds and insertion bounds; `tool.set_tolerances(0.1,0.2)`.
    pub fn set_tolerances(&self, first: f64, second: f64) {
        let mut state = self.state.write();
        state.selector.set_tolerances(first, second);
        state.tolerance = [first, second];
    }
    /// Read insertion tolerances without squaring; `tool.get_tolerance()`.
    pub fn get_tolerance(&self) -> (f64, f64) {
        let tolerance = self.state.read().tolerance;
        (tolerance[0], tolerance[1])
    }
    /// Retain the current vector handle; `tool.vertices()`.
    pub fn vertices(&self) -> Handle<MeshVertexCollection> {
        self.state.read().selector.vertices()
    }
    /// Access the native mutable handle through shared ownership; `*tool.change_vertices().write() = replacement`.
    pub fn change_vertices(&self) -> MeshVertexCollectionSlot {
        self.state.read().selector.change_vertices()
    }
    /// Read a one-based stored vertex; `tool.find_key(1)?`.
    pub fn find_key(&self, index: i32) -> Result<BRepMeshVertex, Failure> {
        self.state.read().selector.get_vertex(index)
    }
    /// Return allocated slots, including deleted vertices; `tool.extent()`.
    pub fn extent(&self) -> i32 {
        self.state.read().selector.nb_vertices()
    }
    /// Test the native storage extent; `tool.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.extent() == 0
    }
    /// Delete the final allocated index without shrinking storage; `tool.remove_last()?`.
    pub fn remove_last(&self) -> Result<(), Failure> {
        self.delete_vertex(self.extent())
    }
    /// Retain a live deletion queue; `tool.get_list_of_del_nodes()`.
    pub fn get_list_of_del_nodes(&self) -> Handle<DeletedVertexIndices> {
        self.state.read().selector.get_list_of_del_points()
    }
}
fn allocator_resource(allocator: &MeshAllocator) -> BaseAllocator {
    allocator
        .get()
        .map(|a| a.resource().clone())
        .unwrap_or_else(BaseAllocator::common)
}
