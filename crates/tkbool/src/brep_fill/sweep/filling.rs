// Rust translation of the static Filling (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Filling`: the face of revolution that fills a round corner between two faces of a
//! sweep.
use super::build_edge::{EdgeGeometry, build_edge};
use super::build_face::{EdgeEdgeMap, FaceBorders, build_face};
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfRevolution,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{
    self, Axis1, Axis2, Axis3, Direction2, Point2, Point3, Transform3, Vector2, Vector3,
};
use rustycad_tkmath::toploc::Location;
use std::f64::consts::PI;

/// The frame of a round corner: the axis of the revolution and the tangent of the first
/// part at the corner.
pub(super) struct CornerFrame {
    pub(super) axe: Axis2,
    pub(super) tangent_on_part1: Vector3,
}

/// The two faces and their edges at the corner, with the auxiliary edges of the
/// previous filling (`Aux1`, `Aux2`, null for none).
pub(super) struct FillingInput<'a> {
    pub(super) ef: &'a Shape,
    pub(super) f1: &'a Shape,
    pub(super) el: &'a Shape,
    pub(super) f2: &'a Shape,
    pub(super) tol: f64,
}

/// Standard_NullObject for a missing geometry (a crash natively).
fn null(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("BRepFill_Sweep Filling: null {what}; expected a {what}"),
        "BRepFill_Sweep",
    )
}

/// A trimmed 2d line (`new Geom2d_TrimmedCurve(new Geom2d_Line(P, D), U1, U2)`).
fn trimmed_line(
    p: Point2,
    d: Direction2,
    u1: f64,
    u2: f64,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p, d)).cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(&line, u1, u2, true, true)?).cast())
}

/// The revolution of the first profile about the corner axis, trimmed to the angle
/// between the profiles, the parameters of the farthest profile point, the angle and
/// whether the result must be reversed; `None` when a profile point is on the axis.
struct Revolution {
    surf: Handle<dyn GeomSurface>,
    angle: f64,
    a_prm: [f64; 3],
    to_reverse: bool,
    f1: f64,
    l1: f64,
    f2: f64,
    l2: f64,
}

/// The revolution surface of the corner.
fn revolution(
    input: &FillingInput,
    frame: &CornerFrame,
    e1: &Edge,
    e2: &Edge,
) -> Result<Option<Revolution>, Failure> {
    let (prof1, f1, l1) = BRepTool::curve(e1)?;
    let (prof2, f2, l2) = BRepTool::curve(e2)?;
    let (c1, c2) = (
        prof1.get().ok_or_else(|| null("curve"))?,
        prof2.get().ok_or_else(|| null("curve"))?,
    );
    // Both profiles are the same curve in different positions; with the same
    // orientation the parameter domain of one is the opposite of the other's.
    let same_curve_domain: bool = input.ef.orientation() != input.el.orientation();
    let mut a_tf = Transform3::default();
    a_tf.set_transformation_to(Axis3::from_axis2(frame.axe));
    let a_prm: [f64; 3] = [f1, 0.5 * (f1 + l1), l1];
    let mut a_p1: [Point3; 3] = [Point3::default(); 3];
    for (p, prm) in a_p1.iter_mut().zip(a_prm) {
        *p = c1.value(prm)?.transformed(a_tf);
    }
    // The farthest point from the center of revolution gives a correct angle.
    let mut max_idx: usize = 0;
    let mut max_dist: f64 = f64::MIN;
    for (i, p) in a_p1.iter().enumerate() {
        let dist: f64 = p.x() * p.x() + p.z() * p.z();
        if dist > max_dist {
            (max_dist, max_idx) = (dist, i);
        }
    }
    let a_prm2: [f64; 3] = [f2, 0.5 * (f2 + l2), l2];
    let a_p2: Point3 = c2
        .value(
            a_prm2[if same_curve_domain {
                max_idx
            } else {
                2 - max_idx
            }],
        )?
        .transformed(a_tf);
    let a_v1 = Vector2::new(a_p1[max_idx].z(), a_p1[max_idx].x());
    let a_v2 = Vector2::new(a_p2.z(), a_p2.x());
    if a_v1.square_magnitude() <= gp::resolution() || a_v2.square_magnitude() <= gp::resolution() {
        return Ok(None);
    }
    let mut angle: f64 = a_v1.angle(a_v2)?;
    let mut axe = Axis1::new(frame.axe.location(), frame.axe.y_direction());
    if angle < 0. {
        angle = -angle;
        axe.reverse();
    }
    let rev: Handle<dyn GeomSurface> =
        Handle::new(GeomSurfaceOfRevolution::new(&prof1, axe)?).cast();
    let surf: Handle<dyn GeomSurface> = Handle::new(GeomRectangularTrimmedSurface::new(
        &rev,
        (0., angle),
        (f1, l1),
        (true, true),
    )?)
    .cast();
    // Control the direction of the rotation.
    let d1u: Vector3 = surf
        .get()
        .ok_or_else(|| null("surface"))?
        .dn(0., a_prm[max_idx], 1, 0)?;
    let to_reverse: bool = d1u.angle(frame.tangent_on_part1)? > PI / 2.;
    Ok(Some(Revolution {
        surf,
        angle,
        a_prm,
        to_reverse,
        f1,
        l1,
        f2,
        l2,
    }))
}

/// The p-curve of the second profile on the revolution (`C2`), reversed with the
/// vertices when it runs against the first, then brought to the range of the profile.
fn second_pcurve(
    rev: &Revolution,
    tol: f64,
    v1: &mut Vertex,
    v2: &mut Vertex,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let c2 = trimmed_line(Point2::new(rev.angle, 0.), Direction2::Y, rev.f1, rev.l1)?;
    let curve = c2.get().ok_or_else(|| null("pcurve"))?;
    let surf = rev.surf.get().ok_or_else(|| null("surface"))?;
    let uv = curve.d0(rev.f1)?;
    let p1: Point3 = surf.d0(uv.x(), uv.y())?;
    let uv = curve.d0(rev.l1)?;
    let p2: Point3 = surf.d0(uv.x(), uv.y())?;
    let p: Point3 = BRepTool::pnt(v1)?;
    if p.distance(&p2) + tol < p.distance(&p1) {
        // E2 runs opposite to E1.
        curve.reverse()?;
        std::mem::swap(v1, v2);
    }
    let mut c3: Handle<dyn Geom2dCurve> = Handle::default();
    geom_lib::same_range(
        precision::p_confusion(),
        &c2,
        (curve.first_parameter(), curve.last_parameter()),
        (rev.f2, rev.l2),
        &mut c3,
    )?;
    Ok(c3)
}

/// The vertices of the corner: of `E1` FORWARD, of `E2` REVERSED.
struct CornerVertices {
    vf: Vertex,
    vl: Vertex,
    v1: Vertex,
    v2: Vertex,
}

/// The vertices of both edges, oriented as natively.
fn corner_vertices(e1: &Edge, e2: &Edge) -> CornerVertices {
    let (vf, vl) = topexp::vertices(e1, false);
    let (v1, v2) = topexp::vertices(e2, false);
    let forward = |v: Vertex| Vertex::unchecked(v.oriented(Orientation::Forward));
    let reversed = |v: Vertex| Vertex::unchecked(v.oriented(Orientation::Reversed));
    CornerVertices {
        vf: forward(vf),
        vl: forward(vl),
        v1: reversed(v1),
        v2: reversed(v2),
    }
}

/// `Filling(EF, F1, EL, F2, EEmap, Tol, Axe, TangentOnPart1, Aux1, Aux2, Result)`: false
/// when a profile point is on the axis or no normal can orient the result.
pub(super) fn filling(
    input: &FillingInput,
    eemap: &mut EdgeEdgeMap,
    frame: &CornerFrame,
    aux: (&mut Edge, &mut Edge),
    result: &mut Face,
) -> Result<bool, Failure> {
    let builder = BRepBuilder::default();
    let (aux1, aux2) = aux;
    let e1: Edge = topods::edge(input.ef)?;
    let e2: Edge = topods::edge(input.el)?;
    let mut cv = corner_vertices(&e1, &e2);
    let (mut e3, mut e4) = (Edge::default(), Edge::default());
    builder.make_edge(&mut e3)?;
    builder.make_edge(&mut e4)?;
    let (mut with_e3, mut with_e4) = (false, false);
    if !aux1.is_null() && !cv.vf.is_same(&cv.v1) {
        e3 = aux1.clone();
        with_e3 = true;
    }
    if cv.vf.is_same(&cv.vl) {
        e4 = e3.clone();
        e4.reverse();
        with_e4 = with_e3;
    } else if !aux2.is_null() && !cv.vl.is_same(&cv.v2) {
        e4 = aux2.clone();
        with_e4 = true;
    }
    let Some(rev) = revolution(input, frame, &e1, &e2)? else {
        return Ok(false);
    };
    let c1 = trimmed_line(Point2::new(0., 0.), Direction2::Y, rev.f1, rev.l1)?;
    let c2 = second_pcurve(&rev, input.tol, &mut cv.v1, &mut cv.v2)?;
    for v in [&cv.vf, &cv.v1, &cv.vl, &cv.v2] {
        BRepTool::pnt(v)?;
    }
    let c3 = trimmed_line(Point2::new(0., rev.f1), Direction2::X, 0., rev.angle)?;
    let c4 = trimmed_line(Point2::new(0., rev.l1), Direction2::X, 0., rev.angle)?;
    let conf: f64 = precision::confusion();
    builder.update_edge_pcurve(&e1, &c1, &rev.surf, &Location::default(), conf, None)?;
    builder.update_edge_pcurve(&e2, &c2, &rev.surf, &Location::default(), conf, None)?;
    let laterals = Laterals {
        rev: &rev,
        c3: &c3,
        c4: &c4,
    };
    if e3.is_same(&e4) {
        e3 = laterals.side(e3, with_e3, (&cv.vf, &cv.v1), rev.f1, false)?;
        builder.update_edge_pcurves(
            &e3,
            (&c3, &c4),
            &rev.surf,
            &Location::default(),
            conf,
            None,
        )?;
        e4 = e3.clone();
        e4.reverse();
    } else {
        e3 = laterals.side(e3, with_e3, (&cv.vf, &cv.v1), rev.f1, true)?;
        e4 = laterals.side4(e4, with_e4, (&cv.vl, &cv.v2), rev.l1)?;
    }
    let borders = FaceBorders {
        e1: &e1,
        e2: &e3,
        e3: &e2,
        e4: &e4,
    };
    build_face(&rev.surf, &borders, eemap, false, false, result)?;
    builder.continuity_on_faces(&e1, &topods::face(input.f1)?, result, Continuity::G1)?;
    builder.continuity_on_faces(&e2, &topods::face(input.f2)?, result, Continuity::G1)?;
    *aux1 = rendered_border(&e3)?;
    *aux2 = rendered_border(&e4)?;
    orient_result(input, &rev, c1, result)
}

/// A calculated border kept as an auxiliary edge, or a new empty edge when it is
/// degenerated.
fn rendered_border(e: &Edge) -> Result<Edge, Failure> {
    if !BRepTool::degenerated(e)? {
        return Ok(e.clone());
    }
    let mut empty = Edge::default();
    BRepBuilder::default().make_edge(&mut empty)?;
    Ok(empty)
}

/// The lateral borders of the filling face.
struct Laterals<'a> {
    rev: &'a Revolution,
    c3: &'a Handle<dyn Geom2dCurve>,
    c4: &'a Handle<dyn Geom2dCurve>,
}

impl Laterals<'_> {
    /// The border at the first ends (`E3`): a new edge on the iso V = `v` of the
    /// revolution, or the auxiliary edge extended to the angle; with `with_pcurve` the
    /// auxiliary edge gets the p-curve C3.
    fn side(
        &self,
        e: Edge,
        with: bool,
        ends: (&Vertex, &Vertex),
        v: f64,
        with_pcurve: bool,
    ) -> Result<Edge, Failure> {
        self.lateral(e, with, ends, v, self.c3, with_pcurve)
    }

    /// The border at the last ends (`E4`) with the p-curve C4.
    fn side4(
        &self,
        e: Edge,
        with: bool,
        ends: (&Vertex, &Vertex),
        v: f64,
    ) -> Result<Edge, Failure> {
        self.lateral(e, with, ends, v, self.c4, true)
    }

    fn lateral(
        &self,
        e: Edge,
        with: bool,
        ends: (&Vertex, &Vertex),
        v: f64,
        pcurve: &Handle<dyn Geom2dCurve>,
        with_pcurve: bool,
    ) -> Result<Edge, Failure> {
        let surf = self.rev.surf.get().ok_or_else(|| null("surface"))?;
        if !with {
            let c3d: Handle<dyn GeomCurve> = surf.v_iso(v)?;
            let g = EdgeGeometry {
                c3d: &c3d,
                c2d: pcurve,
                s: &self.rev.surf,
            };
            return build_edge(
                &g,
                ends.0,
                ends.1,
                (0., self.rev.angle),
                precision::confusion(),
            );
        }
        extend_to_angle(&e, self.rev.angle)?;
        if with_pcurve {
            BRepBuilder::default().update_edge_pcurve(
                &e,
                pcurve,
                &self.rev.surf,
                &Location::default(),
                precision::confusion(),
                None,
            )?;
        }
        Ok(e)
    }
}

/// An auxiliary edge shorter than the angle extended to it, its last vertex set to
/// Precision::Confusion().
fn extend_to_angle(e: &Edge, angle: f64) -> Result<(), Failure> {
    let angle_old: f64 = BRepAdaptorCurve::with_edge(e)?.last_parameter()?;
    if angle > angle_old {
        BRepBuilder::default().range(e, 0., angle, false)?;
        let v: Vertex = topexp::last_vertex(e, false);
        vertex_tshape(&v, "BRepFill_Sweep Filling")?.set_tolerance(precision::confusion());
    }
    Ok(())
}

/// The orientation of the result from the angle between its normal and the normal of
/// the first face at the first sample point where both are defined (the p-curve read
/// there is the one on F1 after the first try, as natively).
fn orient_result(
    input: &FillingInput,
    rev: &Revolution,
    mut c1: Handle<dyn Geom2dCurve>,
    result: &mut Face,
) -> Result<bool, Failure> {
    let f1: Face = topods::face(input.f1)?;
    let e1: Edge = topods::edge(input.ef)?;
    let surf = rev.surf.get().ok_or_else(|| null("surface"))?;
    let mut angle: f64 = f64::MIN;
    for prm in rev.a_prm {
        let uv = c1.get().ok_or_else(|| null("pcurve"))?.d0(prm)?;
        let d = surf.d1(uv.x(), uv.y())?;
        let n1: Vector3 = d.d1u.crossed(d.d1v);
        if n1.square_magnitude() < precision::square_confusion() {
            continue;
        }
        c1 = BRepTool::curve_on_face(&e1, &f1)?.pcurve;
        let uv = c1.get().ok_or_else(|| null("pcurve"))?.d0(prm)?;
        let adaptor = BRepAdaptorSurface::with_face(&f1, true)?;
        let d2 = adaptor.d1(uv.x(), uv.y())?;
        let n2: Vector3 = d2.d1u.crossed(d2.d1v);
        if n2.square_magnitude() < precision::square_confusion() {
            continue;
        }
        angle = n1.angle(n2)?;
        break;
    }
    if angle == f64::MIN {
        return Ok(false);
    }
    let reversed: bool = (input.f1.orientation() == Orientation::Reversed) ^ (angle > PI / 2.);
    result.set_orientation(if reversed {
        Orientation::Reversed
    } else {
        Orientation::Forward
    });
    if rev.to_reverse {
        result.reverse();
    }
    Ok(true)
}
