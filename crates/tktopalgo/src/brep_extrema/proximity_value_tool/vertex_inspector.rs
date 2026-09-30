// Rust translation of BRepExtrema_VertexInspector (BRepExtrema_ProximityValueTool.hxx/.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_VertexInspector`: the cell-filter inspector that tells whether a point
//! coincides with one already added.
use rustycad_tkernel::ncollection::{CellAction, CellInspector};
use rustycad_tkernel::precision;
use rustycad_tkmath::gp::Xyz;

/// `BRepExtrema_VertexInspector` (docs/topalgo-design.md section 13): the added points
/// (target `i` is the point added `i`-th, from 1) and the current point; an inspected
/// target within the tolerance (squared, per coordinate) of the current point means the
/// current point need not be added. Each inspected target sets the answer again, so
/// the last one decides (native quirk).
///
/// ```
/// use rustycad_tkmath::gp::Xyz;
/// use rustycad_tktopalgo::brep_extrema::BRepExtremaVertexInspector;
/// let mut inspector = BRepExtremaVertexInspector::new();
/// inspector.add(Xyz::new(0.0, 0.0, 0.0));
/// inspector.set_current(Xyz::new(0.0, 0.0, 0.0));
/// inspector.inspect_target(1);
/// assert!(!inspector.is_need_add());
/// ```
#[derive(Clone)]
pub struct BRepExtremaVertexInspector {
    my_tol: f64,
    my_is_need_add: bool,
    my_points: Vec<Xyz>,
    my_current: Xyz,
}

impl Default for BRepExtremaVertexInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl BRepExtremaVertexInspector {
    /// `Dimension`: the points have three coordinates.
    pub const DIMENSION: i32 = 3;

    /// `BRepExtrema_VertexInspector()`: tolerance Precision::SquareConfusion(), no
    /// point; `BRepExtremaVertexInspector::new()`.
    pub fn new() -> Self {
        Self {
            my_tol: precision::square_confusion(),
            my_is_need_add: true,
            my_points: Vec::new(),
            my_current: Xyz::default(),
        }
    }

    /// `Coord(i, thePnt)`: the coordinate `i` (from 0) of a point;
    /// `BRepExtremaVertexInspector::coord(2, &point)`.
    pub fn coord(i: i32, point: &Xyz) -> f64 {
        match i {
            0 => point.x(),
            1 => point.y(),
            _ => point.z(),
        }
    }

    /// `Shift(thePnt, theTol)`: the point moved by `tol` along the three axes;
    /// `BRepExtremaVertexInspector::shift(&point, 1e-7)`.
    pub fn shift(point: &Xyz, tol: f64) -> Xyz {
        Xyz::new(point.x() + tol, point.y() + tol, point.z() + tol)
    }

    /// `Add(thePnt)`: keeps a point, the target of the next index; `inspector.add(point)`.
    pub fn add(&mut self, point: Xyz) {
        self.my_points.push(point);
    }

    /// `SetTol(theTol)`: the (squared) tolerance; `inspector.set_tol(1e-14)`.
    pub fn set_tol(&mut self, tol: f64) {
        self.my_tol = tol;
    }

    /// `SetCurrent(theCurPnt)`: the point searched for; it needs adding until a target
    /// says otherwise; `inspector.set_current(point)`.
    pub fn set_current(&mut self, current: Xyz) {
        self.my_current = current;
        self.my_is_need_add = true;
    }

    /// `IsNeedAdd()`: whether the current point is new; `inspector.is_need_add()`.
    pub fn is_need_add(&self) -> bool {
        self.my_is_need_add
    }

    /// `Inspect(theTarget)`: compares the point of `target` with the current point;
    /// always keeps the target. A target without a point (an unchecked read natively)
    /// leaves the answer at "add"; `inspector.inspect_target(1)`.
    pub fn inspect_target(&mut self, target: i32) -> CellAction {
        self.my_is_need_add = true;
        let point = usize::try_from(target - 1)
            .ok()
            .and_then(|i| self.my_points.get(i));
        let Some(point) = point else {
            return CellAction::Keep;
        };
        let dx: f64 = self.my_current.x() - point.x();
        let dy: f64 = self.my_current.y() - point.y();
        let dz: f64 = self.my_current.z() - point.z();
        if dx * dx <= self.my_tol && dy * dy <= self.my_tol && dz * dz <= self.my_tol {
            self.my_is_need_add = false;
        }
        CellAction::Keep
    }
}

impl CellInspector for BRepExtremaVertexInspector {
    type Target = i32;
    type Point = Xyz;
    const DIMENSION: usize = 3;
    fn coord(axis: usize, point: &Xyz) -> f64 {
        Self::coord(axis as i32, point)
    }
    fn is_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
    fn inspect(&mut self, target: &i32) -> CellAction {
        self.inspect_target(*target)
    }
}
