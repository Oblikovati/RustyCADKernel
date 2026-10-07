// Rust translation of ChFi2d_Builder::BuildFilletEdge (ChFi2d_Builder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillet edge at a vertex: the circle tangent to the 2d curves of both edges
//! (Geom2dGcc_Circ2d2TanRad with the qualifiers of the corner), the solution nearest to
//! the vertex inside both edges, and the arc between the tangency points.
use super::ChFi2dBuilder;
use super::basis_pcurve;
use crate::chfi2d::ConstructionError;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::gcc_ent::Position;
use rustycad_tkgeomalgo::geom2d_gcc::{Circ2d2TanRad, QualifiedCurve};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Point2, Vector2};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, build_curves3d_default};
use std::f64::consts::PI;

/// The corner of the fillet: the edges (FORWARD), their vertices, the parameters of the
/// vertex and of the other end on each one, the orientations, the basis 2d curves with
/// their ranges, and the tangents at the vertex.
struct Corner {
    v: [Vertex; 4],
    params: [f64; 4],
    o1: Orientation,
    oe1: Orientation,
    basis: [Handle<dyn Geom2dCurve>; 2],
    ranges: [(f64, f64); 2],
    p: Point2,
    ve: [Vector2; 2],
    sens: [bool; 2],
}

/// The 2d tangent of a line or circle at `param`, its point, and whether it runs
/// directly (true for a line).
fn tangent(
    basis: &Handle<dyn Geom2dCurve>,
    param: f64,
) -> Result<(Point2, Vector2, bool), Failure> {
    if let Some(circle) = Handle::<Geom2dCircle>::down_cast(basis).get() {
        let circ = circle.circ2d()?;
        let (p, v) = elclib::d1(param, &circ);
        return Ok((p, v, circ.is_direct()));
    }
    let line = Handle::<Geom2dLine>::down_cast(basis);
    let line = line.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFi2d_Builder::BuildFilletEdge: the edge is not a line or a circle",
        )
    })?;
    let (p, v) = elclib::d1(param, &line.lin2d());
    Ok((p, v, true))
}

/// The vertex of an edge (`first`, `last`) at `v` and the other one: `first` when it is
/// the same as `v`.
fn near_and_far<'a>(v: &Vertex, first: &'a Vertex, last: &'a Vertex) -> (&'a Vertex, &'a Vertex) {
    if first.is_same(v) {
        (first, last)
    } else {
        (last, first)
    }
}

fn is_circle(c: &Handle<dyn Geom2dCurve>) -> bool {
    c.is_instance_name("Geom2d_Circle")
}

fn is_line(c: &Handle<dyn Geom2dCurve>) -> bool {
    c.is_instance_name("Geom2d_Line")
}

impl Corner {
    /// Save the non-modified parts of the edges, restore their geometric supports.
    fn new(builder: &ChFi2dBuilder, v: &Vertex, adj1: &Edge, adj2: &Edge) -> Result<Self, Failure> {
        let (mut e1, mut e2) = (adj1.clone(), adj2.clone());
        let (v1, v2) = (
            topexp::first_vertex(&e1, false),
            topexp::last_vertex(&e1, false),
        );
        let (v3, v4) = (
            topexp::first_vertex(&e2, false),
            topexp::last_vertex(&e2, false),
        );
        let oe1 = e1.orientation();
        e1.set_orientation(Orientation::Forward);
        e2.set_orientation(Orientation::Forward);
        let (near1, far1) = near_and_far(v, &v1, &v2);
        let (near2, far2) = near_and_far(v, &v3, &v4);
        let o1 = far1.orientation();
        let params = [
            BRepTool::parameter(near1, &e1)?,
            BRepTool::parameter(far1, &e1)?,
            BRepTool::parameter(near2, &e2)?,
            BRepTool::parameter(far2, &e2)?,
        ];
        let c1 = BRepTool::curve_on_face(&e1, &builder.new_face)?;
        let c2 = BRepTool::curve_on_face(&e2, &builder.new_face)?;
        let basis = [
            basis_pcurve(&e1, &builder.new_face)?,
            basis_pcurve(&e2, &builder.new_face)?,
        ];
        let (_, ve1, sens1) = tangent(&basis[0], params[0])?;
        let (p, ve2, sens2) = tangent(&basis[1], params[2])?;
        Ok(Self {
            v: [v1, v2, v3, v4],
            params,
            o1,
            oe1,
            basis,
            ranges: [(c1.first, c1.last), (c2.first, c2.last)],
            p,
            ve: [ve1, ve2],
            sens: [sens1, sens2],
        })
    }

    /// The qualifiers of both curves by the side of the corner (the sign of the cross
    /// product of the tangents, 0 at a downcast point) and their sense.
    fn qualifiers(&self, cross: f64) -> (Position, Position) {
        let [param1, param2, param3, param4] = self.params;
        let pick = |direct: bool, a: Position, b: Position| if direct { a } else { b };
        let (enclosed, outside) = (Position::Enclosed, Position::Outside);
        if cross < 0.0 {
            let q1 = if param3 > param4 {
                pick(self.sens[0], enclosed, outside)
            } else {
                pick(self.sens[0], outside, enclosed)
            };
            let q2 = if param1 > param2 {
                pick(self.sens[1], outside, enclosed)
            } else {
                pick(self.sens[1], enclosed, outside)
            };
            return (q1, q2);
        }
        let q1 = if param3 > param4 {
            pick(self.sens[0], outside, enclosed)
        } else {
            pick(self.sens[0], enclosed, outside)
        };
        let q2 = if param1 > param2 {
            pick(self.sens[1], enclosed, outside)
        } else {
            pick(self.sens[1], outside, enclosed)
        };
        (q1, q2)
    }
}

/// Whether `u` is strictly between `a` and `b` (either order).
fn strictly_inside(u: f64, a: f64, b: f64) -> bool {
    (u < a && u > b) || (u < b && u > a)
}

/// The case of an arc of circle containing the circle origin (2 pi strictly between
/// the parameters): `u` on the arc across the origin.
fn across_origin(u: f64, a: f64, b: f64) -> Option<bool> {
    let two_pi = 2.0 * PI;
    if !strictly_inside(two_pi, a, b) {
        return None;
    }
    let inside = (a < u && u < two_pi) || (0.0 <= u && u < b - two_pi);
    Some(inside || (b < u && u < two_pi) || (0.0 <= u && u < a - two_pi))
}

impl ChFi2dBuilder {
    /// `BuildFilletEdge(V, AdjEdge1, AdjEdge2, Radius, NewExtr1, NewExtr2)`: the fillet
    /// edge and its new extremities on both edges; the status tells a tangency or a
    /// computation error (null edge).
    pub(super) fn build_fillet_edge(
        &mut self,
        v: &Vertex,
        (adj1, adj2): (&Edge, &Edge),
        radius: f64,
    ) -> Result<(Edge, Vertex, Vertex), Failure> {
        let none = (Edge::default(), Vertex::default(), Vertex::default());
        let corner = Corner::new(self, v, adj1, adj2)?;
        let Some(cross) = self.corner_cross(&corner)? else {
            return Ok(none);
        };
        let (q1, q2) = corner.qualifiers(cross);
        let fillet = solve_fillet(&corner, (q1, q2), radius)?;
        if !fillet.is_done() || fillet.nb_solutions() == 0 {
            self.status = ConstructionError::ComputationError;
            return Ok(none);
        }
        self.status = ConstructionError::IsDone;
        let numsol = choose_solution(&corner, &fillet)?;
        let (u1, u2, ptg1) = fillet.tangency1(numsol)?;
        let (vv1, vv2, ptg2) = fillet.tangency2(numsol)?;
        if !valid_tangency(&corner, u2, 0) || !valid_tangency(&corner, vv2, 1) {
            self.status = ConstructionError::ComputationError;
            return Ok(none);
        }
        let (x1, x2) = self.new_extremities(&corner, (u2, ptg1), (vv2, ptg2))?;
        let cir = fillet.this_solution(numsol)?;
        let edge = self.fillet_arc(v, &corner, cir, (u1, vv1), (&x1, &x2))?;
        Ok((edge, x1, x2))
    }

    /// The cross product of the tangents at the vertex, 0 at a downcast point; `None` at
    /// a true tangency point (the status is set).
    fn corner_cross(&mut self, c: &Corner) -> Result<Option<f64>, Failure> {
        let [ve1, ve2] = c.ve;
        let cross = ve1.crossed(ve2);
        if !ve1.is_parallel(ve2, precision::angular())? {
            return Ok(Some(cross));
        }
        // Ve1 and Ve2 are parallel: cross at 0
        let ve3 = if c.params[0] < c.params[1] { -ve1 } else { ve1 };
        let ve4 = if c.params[2] > c.params[3] { -ve2 } else { ve2 };
        if !ve4.is_opposite(ve3, precision::angular())? {
            // There is a true tangency point and the calculation is stopped
            self.status = ConstructionError::TangencyError;
            return Ok(None);
        }
        // Otherwise this is a downcast point, and the calculation is continued
        Ok(Some(0.0))
    }

    /// The vertices at the tangency points (the vertices of the edges when the points
    /// are at their ends).
    fn new_extremities(
        &self,
        c: &Corner,
        (u2, ptg1): (f64, Point2),
        (vv2, ptg2): (f64, Point2),
    ) -> Result<(Vertex, Vertex), Failure> {
        let surface = BRepAdaptorSurface::with_face(&self.ref_face, true)?;
        let b = BRepBuilder::default();
        let tol = precision::confusion();
        let pick = |u: f64,
                    (first, last): (f64, f64),
                    at_first: &Vertex,
                    at_last: &Vertex,
                    made: Vertex| {
            let mut out = made;
            if (u - first).abs() <= precision::p_confusion() {
                out = at_first.clone();
            }
            if (u - last).abs() <= precision::p_confusion() {
                out = at_last.clone();
            }
            out
        };
        let mut vertex1 = Vertex::default();
        b.make_vertex_at(&mut vertex1, surface.value(ptg1.x(), ptg1.y())?, tol)?;
        let x1 = pick(u2, c.ranges[0], &c.v[0], &c.v[1], vertex1);
        let mut vertex2 = Vertex::default();
        b.make_vertex_at(&mut vertex2, surface.value(ptg2.x(), ptg2.y())?, tol)?;
        let x2 = pick(vv2, c.ranges[1], &c.v[2], &c.v[3], vertex2);
        Ok((x1, x2))
    }

    /// The fillet arc on the plane of the reference face between the new extremities,
    /// oriented by the sense of the first edge at the vertex; its 3d curve built.
    fn fillet_arc(
        &self,
        v: &Vertex,
        c: &Corner,
        cir: Circle2,
        (mut u1, mut vv1): (f64, f64),
        (x1, x2): (&Vertex, &Vertex),
    ) -> Result<Edge, Failure> {
        let sense = self.fillet_sense(v, c, cir, u1)?;
        if u1 > vv1 && u1 > 2.0 * PI {
            (u1, vv1) = elclib::adjust_periodic(0.0, 2.0 * PI, precision::confusion(), u1, vv1);
        }
        let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_circ2d(cir)).cast();
        let plane = BRepAdaptorSurface::with_face(&self.ref_face, true)?.plane()?;
        let ref_surf: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(plane)).cast();
        let same = (c.o1 == Orientation::Forward && c.oe1 == Orientation::Forward)
            || (c.o1 == Orientation::Reversed && c.oe1 == Orientation::Reversed);
        let make = |forward: bool| -> Result<Edge, Failure> {
            if forward {
                BRepLibMakeEdge::from_pcurve_vertices_params(&circle, &ref_surf, x1, x2, u1, vv1)?
                    .edge()
            } else {
                BRepLibMakeEdge::from_pcurve_vertices_params(&circle, &ref_surf, x2, x1, vv1, u1)?
                    .edge()
            }
        };
        let mut edge = make(same)?;
        if !sense {
            let s1 = edge.orientation();
            edge = make(!same)?;
            edge.set_orientation(if s1 == Orientation::Forward {
                Orientation::Reversed
            } else {
                Orientation::Forward
            });
        }
        build_curves3d_default(&edge)?;
        Ok(edge)
    }

    /// Update tops of the fillet: whether the fillet circle at its first tangency runs
    /// as the first edge leaving the vertex.
    fn fillet_sense(&self, v: &Vertex, c: &Corner, cir: Circle2, u1: f64) -> Result<bool, Failure> {
        let pntbid = BRepTool::pnt(v)?;
        let sommet = Point2::new(pntbid.x(), pntbid.y());
        let (ptg1, vec) = elclib::d1(u1, &cir);
        let mut vec1 = Vector2::default();
        if let Some(cc1) = Handle::<Geom2dCircle>::down_cast(&c.basis[0]).get() {
            let cir2d = cc1.circ2d()?;
            let par = elclib::circle_parameter_2d(cir2d.position(), ptg1)?;
            vec1 = elclib::d1(par, &cir2d).1;
        } else if let Some(cc1) = Handle::<Geom2dLine>::down_cast(&c.basis[0]).get() {
            let lin2d = cc1.lin2d();
            let par = elclib::line_parameter_2d(lin2d.position(), sommet);
            vec1 = elclib::d1(par, &lin2d).1;
        }
        if c.oe1 == Orientation::Reversed {
            vec1 = -vec1;
        }
        Ok(vec1.dot(vec) > 0.0)
    }
}

/// `Geom2dGcc_Circ2d2TanRad` on the basis curves with their qualifiers.
fn solve_fillet(
    c: &Corner,
    (q1, q2): (Position, Position),
    radius: f64,
) -> Result<Circ2d2TanRad, Failure> {
    let a1 = Geom2dAdaptorCurve::with_curve(&c.basis[0])?;
    let a2 = Geom2dAdaptorCurve::with_curve(&c.basis[1])?;
    Circ2d2TanRad::new(
        &QualifiedCurve::new(&a1, q1)?,
        &QualifiedCurve::new(&a2, q2)?,
        radius,
        precision::confusion(),
    )
}

/// The solution whose tangency point on the first curve (on a line) or the second one is
/// nearest to the vertex point among those inside the edge; 1 when none.
fn choose_solution(c: &Corner, fillet: &Circ2d2TanRad) -> Result<i32, Failure> {
    let [param1, param2, param3, param4] = c.params;
    let (mut numsol, mut dist1) = (1, 1.0e40);
    for nsol in 1..=fillet.nb_solutions() {
        let (_, pu2, ptg1) = fillet.tangency1(nsol)?;
        let (inside, dist) = if is_line(&c.basis[0]) {
            (strictly_inside(pu2, param1, param2), ptg1.distance(c.p))
        } else {
            let (_, ppu2, ptg2) = fillet.tangency2(nsol)?;
            let mut inside = strictly_inside(ppu2, param3, param4);
            if is_circle(&c.basis[1]) {
                inside = across_origin(ppu2, param3, param4).unwrap_or(inside);
            }
            (inside, ptg2.distance(c.p))
        };
        if inside && dist < dist1 {
            (numsol, dist1) = (nsol, dist);
        }
    }
    Ok(numsol)
}

/// Check the validity of a tangency parameter on the edge `k` (bug 0022695: one end
/// included), across the origin of a circle.
fn valid_tangency(c: &Corner, u: f64, k: usize) -> bool {
    let (a, b) = (c.params[2 * k], c.params[2 * k + 1]);
    let inside = (u < a && u >= b) || (u <= b && u > a);
    if is_circle(&c.basis[k]) {
        return across_origin(u, a, b).unwrap_or(inside);
    }
    inside
}
