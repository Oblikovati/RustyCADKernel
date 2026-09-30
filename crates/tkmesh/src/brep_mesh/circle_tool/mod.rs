//! Native circle bounds, cell registration, and circumcircle arithmetic.
//! Ported from OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871 BRepMesh_CircleTool.hxx/.cxx.
use crate::brep_mesh_data::MeshAllocator;
use rustycad_tkernel::{
    ncollection::{Array1, BaseAllocator, CellFilter, List},
    standard::Failure,
};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Circle2, Xy};
mod circle;
mod construction;
mod inspector;
pub use circle::BRepMeshCircle;
pub use inspector::{BRepMeshCircleInspector, CircleInspectorPoint, CircleInspectorTarget};
/// Spatial circle index used by Delaunay insertion; `BRepMeshCircleTool::new(&allocator)`.
pub struct BRepMeshCircleTool {
    tolerance: f64,
    allocator: MeshAllocator,
    filter: CellFilter<BRepMeshCircleInspector>,
    selector: BRepMeshCircleInspector,
    face_max: Xy,
    face_min: Xy,
}
impl BRepMeshCircleTool {
    /// Create native ten-unit cells and 64-slot blocks; `BRepMeshCircleTool::new(&allocator)`.
    pub fn new(allocator: &MeshAllocator) -> Self {
        Self::with_reserved(64, allocator)
    }
    /// Reserve at least 64 circle slots; `BRepMeshCircleTool::with_reserved(100,&allocator)`.
    pub fn with_reserved(reserved: i32, allocator: &MeshAllocator) -> Self {
        let tolerance = rustycad_tkernel::precision::p_confusion();
        Self {
            tolerance,
            allocator: allocator.clone(),
            filter: CellFilter::with_allocator(10., allocator.get().map(|a| a.resource().clone())),
            selector: BRepMeshCircleInspector::new(tolerance, reserved.max(64), allocator),
            face_max: Xy::default(),
            face_min: Xy::default(),
        }
    }
    /// Reset the unused native tolerance field without clearing circles; `tool.init(100)`.
    pub fn init(&mut self, _reserved: i32) {
        self.tolerance = rustycad_tkernel::precision::p_confusion();
    }
    /// Reset spatial cells while retaining registered circles; `tool.set_cell_size(0.5)?`.
    pub fn set_cell_size(&mut self, size: f64) -> Result<(), Failure> {
        self.filter
            .reset(size, self.allocator.get().map(|a| a.resource().clone()))
    }
    /// Retain native one-based axis-size bounds; `tool.set_cell_sizes(0.5,0.25)?`.
    pub fn set_cell_sizes(&mut self, x: f64, y: f64) -> Result<(), Failure> {
        let mut sizes =
            Array1::with_bounds(1, 2).map_err(|e| e.into_failure("BRepMesh_CircleTool"))?;
        sizes.as_mut_slice().copy_from_slice(&[x, y]);
        self.filter
            .reset_sizes(&sizes, self.allocator.get().map(|a| a.resource().clone()));
        Ok(())
    }
    /// Clip subsequent insertion boxes to these bounds; `tool.set_min_max_size(min,max)`.
    pub fn set_min_max_size(&mut self, min: Xy, max: Xy) {
        self.face_min = min;
        self.face_max = max;
    }
    /// Test circle storage rather than populated cells; `tool.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.selector.circles().is_empty()
    }
    /// Register a gp circle without removing old cell references; `tool.bind(1,&circle)?`.
    pub fn bind(&mut self, index: i32, circle: &Circle2) -> Result<(), Failure> {
        self.bind_parameters(index, circle.location().xy(), circle.radius())
    }
    /// Construct a circle, retaining previous storage if construction fails; `tool.bind_points(1,&points)?`.
    pub fn bind_points(&mut self, index: i32, points: &[Xy; 3]) -> Result<bool, Failure> {
        let mut center = Xy::default();
        let mut radius = 0.;
        if !Self::make_circle(points, &mut center, &mut radius) {
            return Ok(false);
        }
        self.bind_parameters(index, center, radius)?;
        Ok(true)
    }
    /// Register a deleted circle without spatial insertion; `tool.moc_bind(1)?`.
    pub fn moc_bind(&mut self, index: i32) -> Result<(), Failure> {
        self.selector
            .bind(index, BRepMeshCircle::new(Xy::default(), -1.))
    }
    /// Mark strictly positive radii deleted; zero, negative and NaN remain unchanged; `tool.delete(1)?`.
    pub fn delete(&mut self, index: i32) -> Result<(), Failure> {
        let circle = self.selector.circle(index)?;
        if circle.radius() > 0. {
            circle.set_radius(-1.);
        }
        Ok(())
    }
    /// Inspect the point's cell and return the live mutable hit list; `tool.select(&point)?`.
    pub fn select(&mut self, point: &Xy) -> Result<&mut List<i32>, Failure> {
        self.selector.set_point(*point);
        self.filter
            .try_inspect_with(point, |index| self.selector.inspect(*index))?;
        Ok(self.selector.get_shot_circles())
    }
    fn bind_parameters(&mut self, index: i32, location: Xy, radius: f64) -> Result<(), Failure> {
        let max = Xy::new(
            std_min(location.x() + radius, self.face_max.x()),
            std_min(location.y() + radius, self.face_max.y()),
        );
        let min = Xy::new(
            std_max(location.x() - radius, self.face_min.x()),
            std_max(location.y() - radius, self.face_min.y()),
        );
        self.filter.add_range(&index, &min, &max);
        self.selector
            .bind(index, BRepMeshCircle::new(location, radius))
    }
}
fn resource(allocator: &MeshAllocator) -> BaseAllocator {
    allocator
        .get()
        .map(|a| a.resource().clone())
        .unwrap_or_else(BaseAllocator::common)
}
