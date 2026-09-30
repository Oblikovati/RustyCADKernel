// Rust translation of BRepBuilderAPI_VertexInspector.hxx and
// BRepBuilderAPI_VertexInspector::Inspect (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_VertexInspector`: the cell-filter inspector that finds the points
//! within a tolerance of a current point, coordinate by coordinate.
use rustycad_tkernel::ncollection::{CellAction, CellInspector, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Xyz;

/// `BRepBuilderAPI_VertexInspector` (docs/topalgo-design.md section 11): it keeps the
/// points added to it (target `i` is the point added `i`-th, from 1) and, for each
/// inspected target, records its index when each coordinate differs from the current
/// point by at most the tolerance.
///
/// ```
/// use rustycad_tkmath::gp::Xyz;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIVertexInspector;
/// let mut inspector = BRepBuilderAPIVertexInspector::new(0.1);
/// inspector.add(Xyz::new(0.0, 0.0, 0.0));
/// inspector.set_current(Xyz::new(0.05, 0.0, 0.0));
/// assert!(inspector.res_ind().is_empty());
/// ```
#[derive(Clone, Default)]
pub struct BRepBuilderAPIVertexInspector {
    my_tol: f64,
    my_res_ind: List<i32>,
    my_points: Vec<Xyz>,
    my_current: Xyz,
}

impl BRepBuilderAPIVertexInspector {
    /// `Coord(i, thePnt)`: coordinate `i` (from 0) of a point; the filter of the sewing
    /// has three axes, so `i` is 0, 1 or 2 (any larger axis reads Z);
    /// `BRepBuilderAPIVertexInspector::coord(1, &point)`.
    pub fn coord(axis: usize, point: &Xyz) -> f64 {
        match axis {
            0 => point.x(),
            1 => point.y(),
            _ => point.z(),
        }
    }

    /// `Shift(thePnt, theTol)`: the point moved by `tol` along the three axes;
    /// `BRepBuilderAPIVertexInspector::shift(&point, -0.1)`.
    pub fn shift(point: &Xyz, tol: f64) -> Xyz {
        Xyz::new(point.x() + tol, point.y() + tol, point.z() + tol)
    }

    /// `BRepBuilderAPI_VertexInspector(theTol)`: keeps the square of the tolerance;
    /// `BRepBuilderAPIVertexInspector::new(1e-6)`.
    pub fn new(tol: f64) -> Self {
        Self {
            my_tol: tol * tol,
            ..Self::default()
        }
    }

    /// `Add(thePnt)`: keeps a point, the target of the next index;
    /// `inspector.add(point)`.
    pub fn add(&mut self, point: Xyz) {
        self.my_points.push(point);
    }

    /// `ClearResList()`: forgets the found indices; `inspector.clear_res_list()`.
    pub fn clear_res_list(&mut self) {
        self.my_res_ind.clear();
    }

    /// `SetCurrent(theCurPnt)`: the point whose neighbours are searched;
    /// `inspector.set_current(point)`.
    pub fn set_current(&mut self, point: Xyz) {
        self.my_current = point;
    }

    /// `ResInd()`: the indices found since the last clear, in inspection order;
    /// `inspector.res_ind().extent()`.
    pub fn res_ind(&self) -> &List<i32> {
        &self.my_res_ind
    }

    /// `Inspect(theTarget)`: records `target` when each squared coordinate difference
    /// to the current point is at most the squared tolerance; always keeps the target.
    /// Standard_OutOfRange for a target without a point (the check of the native point
    /// vector); `inspector.inspect_target(1)?`.
    pub fn inspect_target(&mut self, target: i32) -> Result<CellAction, Failure> {
        let count = self.my_points.len() as i32;
        let Some(point) = usize::try_from(target - 1)
            .ok()
            .and_then(|i| self.my_points.get(i))
        else {
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!(
                    "BRepBuilderAPI_VertexInspector::Inspect: target {target} is outside [1, {count}]"
                ),
                "",
            ));
        };
        let dx: f64 = self.my_current.x() - point.x();
        let dy: f64 = self.my_current.y() - point.y();
        let dz: f64 = self.my_current.z() - point.z();
        if dx * dx <= self.my_tol && dy * dy <= self.my_tol && dz * dz <= self.my_tol {
            self.my_res_ind.append(target);
        }
        Ok(CellAction::Keep)
    }
}

impl CellInspector for BRepBuilderAPIVertexInspector {
    type Target = i32;
    type Point = Xyz;
    const DIMENSION: usize = 3;
    fn coord(axis: usize, point: &Xyz) -> f64 {
        Self::coord(axis, point)
    }
    fn is_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
    // The cell filter only holds the targets added with their points, so the native
    // range check never raises here; a failure would keep the target unrecorded.
    fn inspect(&mut self, target: &i32) -> CellAction {
        self.inspect_target(*target).unwrap_or(CellAction::Keep)
    }
}
