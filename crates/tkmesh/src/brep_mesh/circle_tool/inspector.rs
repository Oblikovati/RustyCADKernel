use super::{BRepMeshCircle, resource};
use crate::brep_mesh_data::MeshAllocator;
use rustycad_tkernel::{
    ncollection::{CellAction, CellGeometry, DynamicArray, List},
    standard::Failure,
};
use rustycad_tkmath::gp::Xy;
/// Native inspector point type.
pub type CircleInspectorPoint = Xy;
/// Native inspector candidate index.
pub type CircleInspectorTarget = i32;
/// Native squared-distance circle selection, including duplicate hits; `BRepMeshCircleInspector::new(1e-9,64,&allocator)`.
pub struct BRepMeshCircleInspector {
    squared_tolerance: f64,
    hits: List<i32>,
    circles: DynamicArray<BRepMeshCircle>,
    point: Xy,
}
impl BRepMeshCircleInspector {
    /// Native spatial dimension; `BRepMeshCircleInspector::DIMENSION == 2`.
    pub const DIMENSION: usize = 2;
    /// Allocate circle storage and result lists in the caller's arena; `BRepMeshCircleInspector::new(1e-9,64,&allocator)`.
    pub fn new(tolerance: f64, reserved: i32, allocator: &MeshAllocator) -> Self {
        Self {
            squared_tolerance: tolerance * tolerance,
            hits: List::with_allocator(resource(allocator)),
            circles: DynamicArray::with_allocator(
                reserved.max(1) as usize,
                Some(resource(allocator)),
            ),
            point: Xy::default(),
        }
    }
    /// Translate zero-based axes to checked native coordinates; `BRepMeshCircleInspector::coord(0,&point)?`.
    pub fn coord(axis: i32, point: &Xy) -> Result<f64, Failure> {
        point.coord(axis.wrapping_add(1))
    }
    /// Offset both coordinates; `BRepMeshCircleInspector::shift(&point,0.1)`.
    pub fn shift(point: &Xy, tolerance: f64) -> Xy {
        Xy::new(point.x() + tolerance, point.y() + tolerance)
    }
    /// Compare candidate identifiers; `BRepMeshCircleInspector::is_equal(1,1)`.
    pub fn is_equal(first: i32, second: i32) -> bool {
        first == second
    }
    /// Extend zero-based storage, default-initializing gaps; `inspector.bind(4,circle)?`.
    pub fn bind(&mut self, index: i32, circle: BRepMeshCircle) -> Result<(), Failure> {
        self.circles
            .set_value(index, circle)
            .map_err(|e| e.into_failure("BRepMesh_CircleInspector::Bind"))?;
        Ok(())
    }
    /// Borrow registered circle slots; `inspector.circles().length()`.
    pub fn circles(&self) -> &DynamicArray<BRepMeshCircle> {
        &self.circles
    }
    /// Mutate a stored circle without updating spatial cells; `inspector.circle(1)?.set_radius(-1.)`.
    pub fn circle(&mut self, index: i32) -> Result<&mut BRepMeshCircle, Failure> {
        self.circles
            .change_value(index)
            .map_err(|e| e.into_failure("BRepMesh_CircleInspector::Circle"))
    }
    /// Clear prior hits before replacing the query; `inspector.set_point(Xy::default())`.
    pub fn set_point(&mut self, point: Xy) {
        self.hits.clear();
        self.point = point;
    }
    /// Borrow the live mutable result list; `inspector.get_shot_circles().append(7)`.
    pub fn get_shot_circles(&mut self) -> &mut List<i32> {
        &mut self.hits
    }
    /// Retain OCCT's squared-tolerance comparison (bug27119); `inspector.inspect(1)?`.
    pub fn inspect(&mut self, index: i32) -> Result<CellAction, Failure> {
        let circle = *self.circle(index)?;
        if circle.radius() < 0. {
            return Ok(CellAction::Purge);
        }
        let dx = self.point.x() - circle.location().x();
        let dy = self.point.y() - circle.location().y();
        // OCCT deliberately retains this test: linear radius expansion hangs perf mesh bug27119.
        if (dx * dx + dy * dy) - (circle.radius() * circle.radius()) <= self.squared_tolerance {
            self.hits.append(index);
        }
        Ok(CellAction::Keep)
    }
}
impl CellGeometry for BRepMeshCircleInspector {
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
