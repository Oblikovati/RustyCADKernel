//! Native topology queries used to prepare and store discrete mesh representations.
use crate::imesh_data::{EdgeHandle, PCurveHandle};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::{geom::GeomCurve, top_abs::ShapeEnum};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Point2, Point3},
    toploc::Location,
};
use rustycad_tkshhealing::shape_analysis::{Curve3dRange, PCurveRange, ShapeAnalysisEdge};
mod flags;
mod polygons;
/// Native mesh shape utilities; `ShapeTool::box_max_dimension(&bounds, &mut size)?`.
#[derive(Default)]
pub struct ShapeTool {
    transient: StandardTransient,
}
impl_transient!(
    ShapeTool,
    "BRepMesh_ShapeTool",
    StandardTransient,
    transient
);
pub(super) fn native_max(left: f64, right: f64) -> f64 {
    if left < right { right } else { left }
}
impl ShapeTool {
    /// Maximum face, edge and vertex tolerance; `ShapeTool::max_face_tolerance(&face)?`.
    pub fn max_face_tolerance(face: &Face) -> Result<f64, Failure> {
        let edge_max = Self::element_tolerance(face, ShapeEnum::Edge)?;
        let vertex_max = Self::element_tolerance(face, ShapeEnum::Vertex)?;
        Ok(native_max(
            BRepTool::face_tolerance(face)?,
            native_max(edge_max, vertex_max),
        ))
    }
    fn element_tolerance(face: &Face, kind: ShapeEnum) -> Result<f64, Failure> {
        let mut maximum = -f64::MAX;
        let mut explorer = Explorer::new(face, kind, ShapeEnum::Shape)?;
        while explorer.more() {
            let tolerance = if kind == ShapeEnum::Edge {
                BRepTool::edge_tolerance(&topods::edge(explorer.current())?)?
            } else {
                BRepTool::vertex_tolerance(&topods::vertex(explorer.current())?)?
            };
            if tolerance > maximum {
                maximum = tolerance;
            }
            explorer.next()?;
        }
        Ok(maximum)
    }
    /// Largest axis extent, preserving output for void boxes; `ShapeTool::box_max_dimension(&bounds, &mut size)?`.
    pub fn box_max_dimension(bounds: &BoundingBox3, maximum: &mut f64) -> Result<(), Failure> {
        if bounds.is_void() {
            return Ok(());
        }
        let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
        *maximum = native_max(xmax - xmin, native_max(ymax - ymin, zmax - zmin));
        Ok(())
    }
    /// Apply a topology placement; `ShapeTool::use_location(point, &location)`.
    pub fn use_location(point: Point3, location: &Location) -> Point3 {
        if location.is_identity() {
            return point;
        }
        point.transformed(location.transformation())
    }
    /// Edge 3D curve and native oriented range; `ShapeTool::range_3d(&edge, false)?`.
    pub fn range_3d(edge: &Edge, consider_orientation: bool) -> Result<Curve3dRange, Failure> {
        ShapeAnalysisEdge::new().curve_3d(edge, consider_orientation)
    }
    /// Pcurve and native oriented range; `ShapeTool::range_on_face(&edge, &face, false)?`.
    pub fn range_on_face(
        edge: &Edge,
        face: &Face,
        consider_orientation: bool,
    ) -> Result<PCurveRange, Failure> {
        ShapeAnalysisEdge::new().pcurve_on_face(edge, face, consider_orientation)
    }
    /// Evaluate pcurve endpoints without changing outputs when no curve exists.
    /// Example: `ShapeTool::uv_points(&edge, &face, &mut first, &mut last, true)?`.
    pub fn uv_points(
        edge: &Edge,
        face: &Face,
        first: &mut Point2,
        last: &mut Point2,
        consider_orientation: bool,
    ) -> Result<bool, Failure> {
        let (pcurve, start, end) = Self::range_on_face(edge, face, consider_orientation)?;
        let Some(curve) = pcurve.get() else {
            return Ok(false);
        };
        *first = curve.value(start)?;
        *last = curve.value(end)?;
        Ok(true)
    }
}
