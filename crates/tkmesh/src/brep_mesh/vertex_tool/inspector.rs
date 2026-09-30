//! Native coincident-node selector and deleted-slot reuse order.
use super::*;
use rustycad_tkernel::ncollection::{CellAction, CellGeometry, DynamicArray, List};
/// Native shared vector of mesh vertices; `MeshVertexCollection::default()`.
pub type MeshVertexCollection = Shared<SharedState<DynamicArray<BRepMeshVertex>>>;
/// Shared mutable native handle slot; `*inspector.change_vertices().write() = replacement`.
pub type MeshVertexCollectionSlot = Handle<Shared<SharedState<Handle<MeshVertexCollection>>>>;
/// Live deletion queue; `inspector.get_list_of_del_points().read().first()`.
pub type DeletedVertexIndices = Shared<SharedState<List<i32>>>;
/// Native inspector query point.
pub type VertexInspectorPoint = Xy;
/// Native inspector candidate index.
pub type VertexInspectorTarget = i32;
/// Tolerance-based selector preserving FIFO deleted-slot reuse; `BRepMeshVertexInspector::new(&allocator)`.
pub struct BRepMeshVertexInspector {
    index: i32,
    minimum_squared: f64,
    tolerance_squared: [f64; 2],
    vertices: MeshVertexCollectionSlot,
    deleted: Handle<DeletedVertexIndices>,
    point: Xy,
}
impl BRepMeshVertexInspector {
    /// Native spatial dimension; `BRepMeshVertexInspector::DIMENSION == 2`.
    pub const DIMENSION: usize = 2;
    /// Allocate the vector separately from the deletion-list arena; `BRepMeshVertexInspector::new(&allocator)`.
    pub fn new(allocator: &MeshAllocator) -> Self {
        let vertices = Handle::new(Shared::new(SharedState::new(DynamicArray::default())));
        Self {
            index: 0,
            minimum_squared: f64::MAX,
            tolerance_squared: [rustycad_tkernel::precision::square_confusion(), 0.0],
            vertices: Handle::new(Shared::new(SharedState::new(vertices))),
            deleted: Handle::new(Shared::new(SharedState::new(List::with_allocator(
                allocator_resource(allocator),
            )))),
            point: Xy::default(),
        }
    }
    /// Read a zero-based coordinate using native bounds checks; `BRepMeshVertexInspector::coord(0,&point)?`.
    pub fn coord(axis: i32, point: &Xy) -> Result<f64, Failure> {
        point.coord(axis.wrapping_add(1))
    }
    /// Shift both coordinates by the tolerance; `BRepMeshVertexInspector::shift(&point,0.1)`.
    pub fn shift(point: &Xy, tolerance: f64) -> Xy {
        Xy::new(point.x() + tolerance, point.y() + tolerance)
    }
    /// Compare candidate indices; `BRepMeshVertexInspector::is_equal(1,1)`.
    pub fn is_equal(first: i32, second: i32) -> bool {
        first == second
    }
    /// Append or reuse the first deleted index; `inspector.add(&vertex)?`.
    pub fn add(&mut self, vertex: &BRepMeshVertex) -> Result<i32, Failure> {
        let deleted = self.deleted.read().first().copied();
        if let Some(index) = deleted {
            self.set_vertex(index, *vertex)?;
            self.deleted.write().remove_first()?;
            return Ok(index);
        }
        let vertices = self.vertices();
        vertices.write().append(*vertex);
        Ok(vertices.read().length())
    }
    /// Set radial squared tolerance and zero second-axis threshold; `inspector.set_tolerance(1e-7)`.
    pub fn set_tolerance(&mut self, tolerance: f64) {
        self.tolerance_squared = [tolerance * tolerance, 0.0];
    }
    /// Set squared axis thresholds; `inspector.set_tolerances(0.1,0.2)`.
    pub fn set_tolerances(&mut self, first: f64, second: f64) {
        self.tolerance_squared = [first * first, second * second];
    }
    /// Clear collections while retaining query state; `inspector.clear()`.
    pub fn clear(&mut self) {
        self.vertices().write().clear(false);
        self.deleted.write().clear();
    }
    /// Mark deleted and append even a repeated index; `inspector.delete(1)?`.
    pub fn delete(&mut self, index: i32) -> Result<(), Failure> {
        let mut vertex = self.get_vertex(index)?;
        vertex.set_movability(DegreeOfFreedom::Deleted);
        self.set_vertex(index, vertex)?;
        self.deleted.write().append(index);
        Ok(())
    }
    /// Count storage including deleted slots; `inspector.nb_vertices()`.
    pub fn nb_vertices(&self) -> i32 {
        self.vertices().read().length()
    }
    /// Read a one-based vertex slot; `inspector.get_vertex(1)?`.
    pub fn get_vertex(&self, index: i32) -> Result<BRepMeshVertex, Failure> {
        Ok(*self
            .vertices()
            .read()
            .value(index.wrapping_sub(1))
            .map_err(vertex_array_failure)?)
    }
    /// Write the native mutable vertex slot; `inspector.set_vertex(1,vertex)?`.
    pub fn set_vertex(&self, index: i32, vertex: BRepMeshVertex) -> Result<(), Failure> {
        *self
            .vertices()
            .write()
            .change_value(index.wrapping_sub(1))
            .map_err(vertex_array_failure)? = vertex;
        Ok(())
    }
    /// Reset query state before candidate inspection; `inspector.set_point(Xy::default())`.
    pub fn set_point(&mut self, point: Xy) {
        self.index = 0;
        self.minimum_squared = f64::MAX;
        self.point = point;
    }
    /// Read the closest strict-tolerance match; `inspector.get_coincident_point()`.
    pub fn get_coincident_point(&self) -> i32 {
        self.index
    }
    /// Retain a live handle to the deletion queue; `inspector.get_list_of_del_points()`.
    pub fn get_list_of_del_points(&self) -> Handle<DeletedVertexIndices> {
        self.deleted.clone()
    }
    /// Read the current native vertex handle; `inspector.vertices()`.
    pub fn vertices(&self) -> Handle<MeshVertexCollection> {
        self.vertices.read().clone()
    }
    /// Access the shared replaceable vertex handle slot; `*inspector.change_vertices().write() = replacement`.
    pub fn change_vertices(&self) -> MeshVertexCollectionSlot {
        self.vertices.clone()
    }
    /// Inspect a candidate, purging deleted slots and preserving strict ties; `inspector.inspect(1)?`.
    pub fn inspect(&mut self, index: i32) -> Result<CellAction, Failure> {
        let vertex = self.get_vertex(index)?;
        if vertex.movability() == DegreeOfFreedom::Deleted {
            self.deleted.write().append(index);
            return Ok(CellAction::Purge);
        }
        let difference = self.point - *vertex.coord();
        let squared = difference.square_modulus();
        if self.in_tolerance(difference) && squared < self.minimum_squared {
            self.minimum_squared = squared;
            self.index = index;
        }
        Ok(CellAction::Keep)
    }
    fn in_tolerance(&self, difference: Xy) -> bool {
        if self.tolerance_squared[1].abs() < rustycad_tkernel::precision::confusion() {
            return difference.square_modulus() < self.tolerance_squared[0];
        }
        difference.x() * difference.x() < self.tolerance_squared[0]
            && difference.y() * difference.y() < self.tolerance_squared[1]
    }
}
impl CellGeometry for BRepMeshVertexInspector {
    type CellTarget = i32;
    type CellPoint = Xy;
    const CELL_DIMENSION: usize = Self::DIMENSION;
    fn cell_coordinate(axis: usize, point: &Xy) -> f64 {
        if axis == 0 { point.x() } else { point.y() }
    }
    fn targets_equal(first: &i32, second: &i32) -> bool {
        Self::is_equal(*first, *second)
    }
}
fn vertex_array_failure(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    error.into_failure("BRepMesh_VertexInspector")
}
