// Rust translation of ChFi2d_Builder::BuildChamferEdge (both), ComputePoint (both) and
// OrientChamfer (ChFi2d_Builder_0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfer edge at a vertex: a segment between the points at the distances from the
//! vertex on both edges (or at the angle from the first edge), with its vertices.
use super::ChFi2dBuilder;
use crate::chfi2d::ConstructionError;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Builder, Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Direction3, Point2, Point3, Vector3};

impl ChFi2dBuilder {
    /// `BuildChamferEdge(V, AdjEdge1, AdjEdge2, D1, D2, NewExtr1, NewExtr2)`: the
    /// chamfer between the points at the distances from the vertex; ParametersError for
    /// a distance not positive.
    pub(super) fn build_chamfer_edge(
        &mut self,
        v: &Vertex,
        (adj1, adj2): (&Edge, &Edge),
        (d1, d2): (f64, f64),
    ) -> Result<(Edge, Vertex, Vertex), Failure> {
        if d1 <= 0.0 || d2 <= 0.0 {
            self.status = ConstructionError::ParametersError;
            return Ok((Edge::default(), Vertex::default(), Vertex::default()));
        }
        let (p1, param1) = compute_point(v, adj1, d1)?;
        let (p2, param2) = compute_point(v, adj2, d2)?;
        let dir = Direction3::from_vector(Vector3::between(p1, p2))?;
        let line = GeomLine::from_point(p1, dir);
        self.finish_chamfer(v, (adj1, adj2), (p1, p2), line, (param1, param2))
    }

    /// `BuildChamferEdge(V, AdjEdge1, D, Ang, AdjEdge2, NewExtr1, NewExtr2)`: the chamfer
    /// from the point at the distance on the first edge, at the angle from it, to its
    /// intersection with the second edge; ParametersError for a distance or an angle not
    /// positive.
    pub(super) fn build_chamfer_edge_angle(
        &mut self,
        v: &Vertex,
        (adj1, adj2): (&Edge, &Edge),
        (d, ang): (f64, f64),
    ) -> Result<(Edge, Vertex, Vertex), Failure> {
        if d <= 0.0 || ang <= 0.0 {
            self.status = ConstructionError::ParametersError;
            return Ok((Edge::default(), Vertex::default(), Vertex::default()));
        }
        let (p1, param1) = compute_point(v, adj1, d)?;
        let my_vec = Vector3::between(p1, BRepTool::pnt(v)?);
        let tan = self.tangent_at_vertex(v, adj2)?;
        // compute the chamfer geometric support
        let rot_axe = Axis1::new(p1, Direction3::from_vector(tan.crossed(my_vec))?);
        let dir = Direction3::from_vector(my_vec.rotated(rot_axe, -ang)?)?;
        let line = GeomLine::from_point(p1, dir);
        let handle: Handle<dyn GeomCurve> = Handle::new(line.clone()).cast();
        let (p2, param2) = self.compute_point_on_line(&handle, adj2)?;
        self.finish_chamfer(
            v,
            (adj1, adj2),
            (p1, p2),
            line,
            (param1, param2.unwrap_or(0.0)),
        )
    }

    /// The tangent of the second edge at the vertex, oriented as the vertex on the edge.
    fn tangent_at_vertex(&self, v: &Vertex, adj2: &Edge) -> Result<Vector3, Failure> {
        let p = BRepTool::pnt(v)?;
        let c = BRepAdaptorCurve::with_edge_on_face(adj2, &self.ref_face)?;
        let (point, mut tan) = c.d1(c.first_parameter()?)?;
        if point.distance(&p) > precision::confusion() {
            tan = c.d1(c.last_parameter()?)?.1;
        }
        let (v1, v2) = topexp::vertices(adj2, false);
        let orient = if v1.is_same(v) {
            v1.orientation()
        } else {
            v2.orientation()
        };
        Ok(if orient == Orientation::Reversed {
            tan * -1.0
        } else {
            tan
        })
    }

    /// `ComputePoint(F, L, E, Param)`: the intersection of the line with the curve of the
    /// edge on the face nearest to the start of the line, and its parameter on the edge
    /// (`None` without intersection, an uninitialized double natively; the point is then
    /// the surface point at the origin).
    fn compute_point_on_line(
        &self,
        line: &Handle<dyn GeomCurve>,
        e: &Edge,
    ) -> Result<(Point3, Option<f64>), Failure> {
        let surface = BRepAdaptorSurface::with_face(&self.ref_face, true)?;
        let lin2d = geom_api::to_2d(line, &surface.plane()?)?;
        let c2d = BRepTool::curve_on_face(e, &self.ref_face)?.pcurve;
        let (adaptor_l, adaptor_c) = (
            Geom2dAdaptorCurve::with_curve(&lin2d)?,
            Geom2dAdaptorCurve::with_curve(&c2d)?,
        );
        let tol = precision::p_intersection();
        let intersection = GInter::between(&adaptor_l, &adaptor_c, tol, tol)?;
        let (mut param_on_line, mut p2d, mut param) = (1e300, Point2::default(), None);
        if intersection.is_done() {
            for i in 1..=intersection.nb_points()? {
                let ip = intersection.point(i)?;
                if ip.param_on_first() < param_on_line {
                    (p2d, param_on_line, param) =
                        (ip.value(), ip.param_on_first(), Some(ip.param_on_second()));
                }
            }
        }
        Ok((surface.value(p2d.x(), p2d.y())?, param))
    }

    /// The chamfer edge on its line from the first point to the second, with its new
    /// vertices on the edges; the edge oriented as the first edge leaves the vertex.
    fn finish_chamfer(
        &mut self,
        v: &Vertex,
        (adj1, adj2): (&Edge, &Edge),
        (p1, p2): (Point3, Point3),
        line: GeomLine,
        (param1, param2): (f64, f64),
    ) -> Result<(Edge, Vertex, Vertex), Failure> {
        let tol = precision::confusion();
        let b = BRepBuilder::default();
        let (mut x1, mut x2) = (Vertex::default(), Vertex::default());
        b.make_vertex_at(&mut x1, p1, tol)?;
        b.make_vertex_at(&mut x2, p2, tol)?;
        x1.set_orientation(Orientation::Forward);
        x2.set_orientation(Orientation::Reversed);
        // chamfer edge construction
        let param = elclib::line_parameter(line.lin().position(), p2);
        let new_line: Handle<dyn GeomCurve> = Handle::new(line).cast();
        let mut chamfer = Edge::default();
        b.make_edge_with_curve(&mut chamfer, &new_line, tol)?;
        b.range(&chamfer, 0.0, param, false)?;
        Builder.add(&mut chamfer, &x1)?;
        b.update_vertex_on_edge(&x1, 0.0, &chamfer, tol)?;
        Builder.add(&mut chamfer, &x2)?;
        b.update_vertex_on_edge(&x2, param, &chamfer, tol)?;
        orient_chamfer(&mut chamfer, adj1, v);
        // set the orientation of NewExtr1 and NewExtr2 for the adjacent edges
        x1.set_orientation(vertex_orientation_at(adj1, v));
        x2.set_orientation(vertex_orientation_at(adj2, v));
        b.update_vertex_on_edge(&x1, param1, adj1, tol)?;
        b.update_vertex_on_edge(&x2, param2, adj2, tol)?;
        self.status = ConstructionError::IsDone;
        Ok((chamfer, x1, x2))
    }
}

/// The orientation of the vertex `v` on the edge (its first vertex or else its last).
fn vertex_orientation_at(e: &Edge, v: &Vertex) -> Orientation {
    let v1 = topexp::first_vertex(e, false);
    if v1.is_same(v) {
        v1.orientation()
    } else {
        topexp::last_vertex(e, false).orientation()
    }
}

/// `ComputePoint(V, E, D, Param)`: the point of the edge at the distance `d` from the
/// vertex and its parameter (along a line, an angle on a circle, the arc length
/// otherwise).
fn compute_point(v: &Vertex, e: &Edge, d: f64) -> Result<(Point3, f64), Failure> {
    let c = BRepAdaptorCurve::with_edge(e)?;
    match c.get_type()? {
        CurveType::Line => point_on_line(&c, v, e, d),
        CurveType::Circle => point_on_circle(&c, v, e, d),
        // in all other case than lines and circles
        _ => point_by_abscissa(&c, v, e, d),
    }
}

/// The line case of `ComputePoint`: the point at the distance `d` from the vertex
/// toward the other one, projected back on the line.
fn point_on_line(
    c: &BRepAdaptorCurve,
    v: &Vertex,
    e: &Edge,
    d: f64,
) -> Result<(Point3, f64), Failure> {
    let (v1, v2) = topexp::vertices(e, false);
    let (p1, p2) = (BRepTool::pnt(&v1)?, BRepTool::pnt(&v2)?);
    let my_vec = Vector3::between(p1, p2).normalized()? * d;
    let point = if v2.is_same(v) {
        p2.translated(my_vec * -1.0)
    } else {
        p1.translated(my_vec)
    };
    let param = elclib::line_parameter(c.line()?.position(), point);
    // szv:OCC20823
    Ok((c.value(param)?, param))
}

/// The circle case of `ComputePoint`: the angle `d / radius` from the parameter of the
/// vertex toward the parameter of the other one.
fn point_on_circle(
    c: &BRepAdaptorCurve,
    v: &Vertex,
    e: &Edge,
    d: f64,
) -> Result<(Point3, f64), Failure> {
    let (v1, v2) = topexp::vertices(e, false);
    let radius = c.circle()?.radius();
    let (near, far) = if v.is_same(&v1) {
        (&v1, &v2)
    } else {
        (&v2, &v1)
    };
    let (param1, param2) = (BRepTool::parameter(near, e)?, BRepTool::parameter(far, e)?);
    let delta_alpha = d / radius;
    let param = if param1 > param2 {
        param1 - delta_alpha
    } else {
        param1 + delta_alpha
    };
    Ok((c.value(param)?, param))
}

/// The other curves in `ComputePoint`: the arc length `d` from the end of the range at
/// the vertex.
fn point_by_abscissa(
    c: &BRepAdaptorCurve,
    v: &Vertex,
    e: &Edge,
    d: f64,
) -> Result<(Point3, f64), Failure> {
    let (v1, v2) = topexp::vertices(e, false);
    let p = BRepTool::pnt(if v.is_same(&v1) { &v1 } else { &v2 })?;
    let cc = c.transformed_curve().curve();
    let curve = cc.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi2d_Builder::ComputePoint: null curve",
        )
    })?;
    let (first, last) = (c.first_parameter()?, c.last_parameter()?);
    let u0 = if p.distance(&c.value(first)?) <= precision::confusion() {
        first
    } else {
        last
    };
    let param = AbscissaPoint::from_abscissa(curve, d, u0)?.parameter()?;
    Ok((curve.value(param)?, param))
}

/// `OrientChamfer(chamfer, E, V)`: FORWARD when the edge and its vertex other than `v`
/// have the same orientation, else REVERSED.
fn orient_chamfer(chamfer: &mut Edge, e: &Edge, v: &Vertex) {
    let orient = e.orientation();
    let (v1, v2) = topexp::vertices(e, false);
    let v_orient = if v1.is_same(v) {
        v2.orientation()
    } else {
        v1.orientation()
    };
    let same = (orient == Orientation::Forward && v_orient == Orientation::Forward)
        || (orient == Orientation::Reversed && v_orient == Orientation::Reversed);
    chamfer.set_orientation(if same {
        Orientation::Forward
    } else {
        Orientation::Reversed
    });
}
