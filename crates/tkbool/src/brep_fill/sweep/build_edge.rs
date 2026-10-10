// Rust translation of the first static BuildEdge (C3d, C2d, S, VF, VL, f, l, Tol3d)
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! An edge of a filling face from its 3d curve and its p-curve.
use rustycad_tkbo::bop_tools::is_micro_edge;
use rustycad_tkbo::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// The point of the surface at the p-curve parameter `t`.
fn surface_point(c2d: &dyn Geom2dCurve, s: &dyn GeomSurface, t: f64) -> Result<Point3, Failure> {
    let uv = c2d.d0(t)?;
    s.d0(uv.x(), uv.y())
}

/// Standard_NullObject for a missing geometry (a crash natively).
fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Sweep BuildEdge: null {what}; expected a {what}"),
        "BRepFill_Sweep",
    )
}

/// The geometry of the edge to build.
pub(super) struct EdgeGeometry<'a> {
    pub(super) c3d: &'a Handle<dyn GeomCurve>,
    pub(super) c2d: &'a Handle<dyn Geom2dCurve>,
    pub(super) s: &'a Handle<dyn GeomSurface>,
}

/// `BuildEdge(C3d, C2d, S, VF, VL, f, l, Tol3d)`: a degenerated edge on the p-curve when
/// the vertices are the same or within their tolerance (the vertices enlarged to the
/// surface points), else the edge of the 3d curve between the vertices (enlarged to
/// its ends) with its p-curve, replaced by a degenerated edge when it is a micro edge.
pub(super) fn build_edge(
    g: &EdgeGeometry,
    vf: &Vertex,
    vl: &Vertex,
    (f, l): (f64, f64),
    tol3d: f64,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let (p1, tol1) = (BRepTool::pnt(vf)?, BRepTool::vertex_tolerance(vf)?);
    let (p2, tol2) = (BRepTool::pnt(vl)?, BRepTool::vertex_tolerance(vl)?);
    let mut tol: f64 = tol1.max(tol2);
    let c2d = g.c2d.get().ok_or_else(|| null("pcurve"))?;
    let s = g.s.get().ok_or_else(|| null("surface"))?;
    if vf.is_same(vl) || p1.distance(&p2) < tol {
        // The degenerated case.
        tol = tol.max(p1.distance(&surface_point(c2d, s, f)?));
        tol = tol.max(p2.distance(&surface_point(c2d, s, l)?));
        builder.update_vertex_tolerance(vf, tol)?;
        builder.update_vertex_tolerance(vl, tol)?;
        return degenerated_edge(g, (vf, vl), (f, l), tol);
    }
    let c3d = g.c3d.get().ok_or_else(|| null("curve"))?;
    let d: f64 = p1.distance(&c3d.d0(f)?);
    if d > tol1 {
        builder.update_vertex_tolerance(vf, d)?;
    }
    let d: f64 = p2.distance(&c3d.d0(l)?);
    if d > tol2 {
        builder.update_vertex_tolerance(vl, d)?;
    }
    let e: Edge = BRepLibMakeEdge::from_curve_vertices_params(g.c3d, vf, vl, f, l)?.edge()?;
    builder.update_edge_pcurve(&e, g.c2d, g.s, &Location::default(), tol3d, None)?;
    if is_micro_edge(&e, &Handle::<Context>::default(), true)? {
        builder.update_vertex_tolerance(vf, p1.distance(&p2))?;
        let ends = (
            &vf.oriented_vertex(Orientation::Forward),
            &vf.oriented_vertex(Orientation::Reversed),
        );
        return degenerated_edge(g, ends, (f, l), tol);
    }
    Ok(e)
}

/// A degenerated edge on the p-curve between two (oriented) vertices.
fn degenerated_edge(
    g: &EdgeGeometry,
    (v1, v2): (&Vertex, &Vertex),
    (f, l): (f64, f64),
    tol: f64,
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut e = Edge::default();
    builder.make_edge(&mut e)?;
    builder.update_edge_pcurve(&e, g.c2d, g.s, &Location::default(), tol, None)?;
    builder.add(&mut e, v1)?;
    builder.add(&mut e, v2)?;
    builder.range(&e, f, l, false)?;
    builder.degenerated(&e, true)?;
    Ok(e)
}

/// The oriented copy of a vertex as a vertex.
trait OrientedVertex {
    fn oriented_vertex(&self, orientation: Orientation) -> Vertex;
}

impl OrientedVertex for Vertex {
    fn oriented_vertex(&self, orientation: Orientation) -> Vertex {
        Vertex::unchecked(self.oriented(orientation))
    }
}
