use super::{Tools, ranges::first_second_range};
use crate::int_tools::{CommonPrt, FClass2d};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_adaptor::BRepAdaptorCurve,
    topexp::Explorer,
    topods::{self, Edge, Face, Vertex, Wire},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    top_abs::{Orientation, ShapeEnum, State},
};
use rustycad_tkmath::gp::{Point2, Point3};
impl Tools {
    /// Tests strict squared distance against summed vertex tolerances; `Tools::compute_vv(&v1, &v2)?`.
    pub fn compute_vv(first: &Vertex, last: &Vertex) -> Result<i32, Failure> {
        let sum = BRepTool::vertex_tolerance(first)? + BRepTool::vertex_tolerance(last)?;
        let distance = BRepTool::pnt(first)?.square_distance(&BRepTool::pnt(last)?);
        Ok(if distance < sum * sum { 0 } else { -1 })
    }
    /// Searches nested edges using composed orientation; `Tools::has_internal_edge(&wire)?`.
    pub fn has_internal_edge(wire: &Wire) -> Result<bool, Failure> {
        for edge in Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if edge.orientation() == Orientation::Internal {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Copies the forward face without children then adds the wire; `Tools::make_face_from_wire_and_face(&wire, &face, &mut out)?`.
    pub fn make_face_from_wire_and_face(
        wire: &Wire,
        face: &Face,
        output: &mut Face,
    ) -> Result<(), Failure> {
        let forward = face.oriented(Orientation::Forward);
        *output = topods::face(&forward.empty_copied()?)?;
        BRepBuilder::default().add(output, wire)
    }
    /// Uses a fresh native IntTools classifier at face tolerance; `Tools::classify_point_by_face(&face, uv)?`.
    pub fn classify_point_by_face(face: &Face, point: Point2) -> Result<State, Failure> {
        FClass2d::new(face, BRepTool::face_tolerance(face)?)?.perform(point, true)
    }
    /// Tests edge vertices using the native hard-coded squared tolerance; `Tools::is_vertex_on_edge(&edge, 0.)?`.
    pub fn is_vertex_on_edge(edge: &Edge, parameter: f64) -> Result<bool, Failure> {
        let point = BRepAdaptorCurve::with_edge(edge)?.value(parameter)?;
        for shape in Explorer::new(edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            let vertex = topods::vertex(&shape)?;
            let _tolerance = BRepTool::vertex_tolerance(&vertex)?;
            // 3d097a0 overwrites the computed tolerance squared with this constant.
            if BRepTool::pnt(&vertex)?.square_distance(&point) < 1e-12 {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Uses the supplied vertex's actual squared tolerance; `Tools::is_edge_vertex(&edge, &vertex, 0.)?`.
    pub fn is_edge_vertex(edge: &Edge, vertex: &Vertex, parameter: f64) -> Result<bool, Failure> {
        let point = BRepAdaptorCurve::with_edge(edge)?.value(parameter)?;
        let tolerance = BRepTool::vertex_tolerance(vertex)?;
        Ok(BRepTool::pnt(vertex)?.square_distance(&point) < tolerance * tolerance)
    }
    /// Accepts either edge's midpoint vertex, as the native implementation does; `Tools::is_common_vertex(&part)?`.
    pub fn is_common_vertex(part: &CommonPrt) -> Result<bool, Failure> {
        let range = part.range1();
        if Self::is_vertex_on_edge(part.edge1(), 0.5 * (range.first() + range.last()))? {
            return Ok(true);
        }
        let range = first_second_range(part)?;
        Self::is_vertex_on_edge(part.edge2(), 0.5 * (range.first() + range.last()))
    }
    /// Adds Confusion to both supplied and stored tolerances; `Tools::is_point_vertex(p, 1e-7, &vertex)?`.
    pub fn is_point_vertex(
        point: Point3,
        tolerance: f64,
        vertex: &Vertex,
    ) -> Result<bool, Failure> {
        let sum = BRepTool::vertex_tolerance(vertex)?
            + tolerance
            + rustycad_tkernel::precision::confusion();
        Ok(BRepTool::pnt(vertex)?.square_distance(&point) <= sum * sum)
    }
    /// Compares midpoint positions using summed edge tolerances; `Tools::is_middle_points_equal(&e1, &e2)?`.
    pub fn is_middle_points_equal(first: &Edge, last: &Edge) -> Result<bool, Failure> {
        let (a, at) = edge_midpoint(first)?;
        let (b, bt) = edge_midpoint(last)?;
        let sum = at + bt;
        Ok(a.square_distance(&b) < sum * sum)
    }
}
fn edge_midpoint(edge: &Edge) -> Result<(Point3, f64), Failure> {
    let tolerance = BRepTool::edge_tolerance(edge)?;
    let (curve, first, last) = BRepTool::curve(edge)?;
    Ok((curve.eval_d0(0.5 * (first + last))?, tolerance))
}
