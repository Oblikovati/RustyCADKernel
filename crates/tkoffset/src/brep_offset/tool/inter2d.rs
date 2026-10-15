// Rust translation of BRepOffset_Tool::Inter2d with the static ProjectVertexOnEdge and
// TryParameter of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection vertices of two edges on a face, and the parameters of vertices on
//! extended edges.
use super::pcurves::build_pcurves;
use super::{BRepOffsetTool, live};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeombase::extrema::{ExtPC, ExtPC2d};
use rustycad_tkmath::gp::Point2;
use rustycad_tktopalgo::brep_lib::BRepLibMakeVertex;

/// The default function tolerance of `Extrema_ExtPC(P, C)` and `Extrema_ExtPC2d(P, C)`.
const EXTREMA_TOL_F: f64 = 1.0e-10;

/// The candidate intersection of `Inter2d`: the parameters on both curves and the 2D
/// point.
struct Candidate {
    u1: f64,
    u2: f64,
    p2d: Point2,
}

/// The p-curves of one pair of `Inter2d` with their ranges.
struct PCurvePair {
    c1: Handle<dyn Geom2dCurve>,
    c2: Handle<dyn Geom2dCurve>,
    fl1: [f64; 2],
    fl2: [f64; 2],
}

impl BRepOffsetTool {
    /// `Inter2d(F, E1, E2, LV, TolConf)`: the intersection vertices of two edges on the
    /// face (common ends first, then end projections, then the 2D intersection, on the
    /// stored ranges then on the whole p-curves), each with its parameters on both edges;
    /// of several, only those nearest the start and the end of `E1`;
    /// `BRepOffsetTool::inter2d(&face, &e1, &e2, &mut lv, 1e-7)?`.
    pub fn inter2d(
        f: &Face,
        e1: &Edge,
        e2: &Edge,
        lv: &mut List<Shape>,
        tol_conf: f64,
    ) -> Result<(), Failure> {
        lv.clear();
        build_pcurves(e1, f)?;
        build_pcurves(e2, f)?;
        let nb_pc1 = if BRepTool::is_closed_on_face(e1, f)? {
            2
        } else {
            1
        };
        let nb_pc2 = if BRepTool::is_closed_on_face(e2, f)? {
            2
        } else {
            1
        };
        let s = BRepTool::surface(f)?;
        let mut ya_sol = false;
        let mut itry = 0;
        while !ya_sol && itry < 2 {
            for i in 1..=nb_pc1 {
                let p1 = pcurve_of(e1, f, i == 1)?;
                for j in 1..=nb_pc2 {
                    let p2 = pcurve_of(e2, f, j == 1)?;
                    let mut pair = PCurvePair {
                        c1: p1.0.clone(),
                        c2: p2.0.clone(),
                        fl1: p1.1,
                        fl2: p2.1,
                    };
                    let found = pair_intersection(&mut pair, itry, tol_conf, &mut ya_sol)?;
                    if let Some(candidate) = found {
                        lv.append(
                            intersection_vertex(&s, &candidate, (e1, e2), tol_conf)?.into_shape(),
                        );
                    }
                }
            }
            itry += 1;
        }
        keep_extreme_vertices(lv, e1)
    }
}

/// The p-curve of the edge on the face (of the reversed edge for the second one of a
/// seam) and its range.
fn pcurve_of(
    e: &Edge,
    f: &Face,
    forward: bool,
) -> Result<(Handle<dyn Geom2dCurve>, [f64; 2]), Failure> {
    let edge = if forward {
        e.clone()
    } else {
        topods::edge(&e.reversed())?
    };
    let pc = BRepTool::curve_on_face(&edge, f)?;
    Ok((pc.pcurve, [pc.first, pc.last]))
}

/// One pair of `Inter2d`: on the second try the whole curve ranges; on the first try
/// the common ends, then the projections of the ends; then the 2D intersection
/// unless a solution is known; the candidate of this pair.
fn pair_intersection(
    pair: &mut PCurvePair,
    itry: i32,
    tol_conf: f64,
    ya_sol: &mut bool,
) -> Result<Option<Candidate>, Failure> {
    let c1 = live(&pair.c1, "pcurve")?;
    let c2 = live(&pair.c2, "pcurve")?;
    if itry == 1 {
        pair.fl1 = [c1.first_parameter(), c1.last_parameter()];
        pair.fl2 = [c2.first_parameter(), c2.last_parameter()];
    }
    let ac1 = Geom2dAdaptorCurve::with_range(&pair.c1, pair.fl1[0], pair.fl1[1])?;
    let ac2 = Geom2dAdaptorCurve::with_range(&pair.c2, pair.fl2[0], pair.fl2[1])?;
    let mut found = None;
    if itry == 0 {
        found = end_intersection(pair, (&ac1, &ac2), tol_conf, ya_sol)?;
    }
    if !*ya_sol {
        found = curve_intersection(pair, (&ac1, &ac2), tol_conf, ya_sol)?.or(found);
    }
    Ok(found)
}

/// The first-try branch of `Inter2d`: the last pair of coincident ends, else the first
/// end of one curve on the other.
fn end_intersection(
    pair: &PCurvePair,
    (ac1, ac2): (&Geom2dAdaptorCurve, &Geom2dAdaptorCurve),
    tol_conf: f64,
    ya_sol: &mut bool,
) -> Result<Option<Candidate>, Failure> {
    let c1 = live(&pair.c1, "pcurve")?;
    let c2 = live(&pair.c2, "pcurve")?;
    let p1 = [c1.value(pair.fl1[0])?, c1.value(pair.fl1[1])?];
    let p2 = [c2.value(pair.fl2[0])?, c2.value(pair.fl2[1])?];
    let mut found = None;
    for (&u1, q1) in pair.fl1.iter().zip(&p1) {
        for (&u2, q2) in pair.fl2.iter().zip(&p2) {
            let finite = u1.abs() < precision::infinite() && u2.abs() < precision::infinite();
            if finite && q1.is_equal(*q2, tol_conf) {
                *ya_sol = true;
                found = Some(Candidate {
                    u1,
                    u2,
                    p2d: c1.value(u1)?,
                });
            }
        }
    }
    if *ya_sol {
        return Ok(found);
    }
    for (&u1, &p2d) in pair.fl1.iter().zip(&p1) {
        if let Some(u2) = nearest_parameter_2d(p2d, ac2)? {
            *ya_sol = true;
            return Ok(Some(Candidate { u1, u2, p2d }));
        }
    }
    for (&u2, &p2d) in pair.fl2.iter().zip(&p2) {
        if let Some(u1) = nearest_parameter_2d(p2d, ac1)? {
            *ya_sol = true;
            return Ok(Some(Candidate { u1, u2, p2d }));
        }
    }
    Ok(None)
}

/// The parameter of the nearest extremum of the point on the curve when it is within
/// Precision::Confusion().
fn nearest_parameter_2d(p: Point2, ac: &Geom2dAdaptorCurve) -> Result<Option<f64>, Failure> {
    let extr = ExtPC2d::with_curve(p, ac, EXTREMA_TOL_F)?;
    if !extr.is_done() || extr.nb_ext()? == 0 {
        return Ok(None);
    }
    let mut dist2_min = extr.square_distance(1)?;
    let mut index_min = 1;
    for ind in 2..=extr.nb_ext()? {
        let dist2 = extr.square_distance(ind)?;
        if dist2 < dist2_min {
            dist2_min = dist2;
            index_min = ind;
        }
    }
    if dist2_min <= precision::square_confusion() {
        return Ok(Some(extr.point(index_min)?.parameter()));
    }
    Ok(None)
}

/// The 2D intersection branch of `Inter2d`: the first intersection point, else the
/// middle of the first intersection segment.
fn curve_intersection(
    pair: &PCurvePair,
    (ac1, ac2): (&Geom2dAdaptorCurve, &Geom2dAdaptorCurve),
    tol_conf: f64,
    ya_sol: &mut bool,
) -> Result<Option<Candidate>, Failure> {
    let inter = GInter::between(ac1, ac2, tol_conf, tol_conf)?;
    if !inter.is_empty()? && inter.nb_points()? > 0 {
        *ya_sol = true;
        let point = inter.point(1)?;
        return Ok(Some(Candidate {
            u1: point.param_on_first(),
            u2: point.param_on_second(),
            p2d: point.value(),
        }));
    }
    if inter.is_empty()? || inter.nb_segments()? == 0 {
        return Ok(None);
    }
    *ya_sol = true;
    let seg = inter.segment(1)?;
    let (int_p1, int_p2) = (seg.first_point()?, seg.last_point()?);
    let u1 = (int_p1.param_on_first() + int_p2.param_on_first()) / 2.;
    let u2 = (int_p1.param_on_second() + int_p2.param_on_second()) / 2.;
    let p2d1 = live(&pair.c1, "pcurve")?.value(u1)?;
    let p2d2 = live(&pair.c2, "pcurve")?.value(u2)?;
    let p2d = Point2::new((p2d1.x() + p2d2.x()) / 2., (p2d1.y() + p2d2.y()) / 2.);
    Ok(Some(Candidate { u1, u2, p2d }))
}

/// The INTERNAL vertex at the surface point of the candidate with its parameters on
/// both edges.
fn intersection_vertex(
    s: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    c: &Candidate,
    (e1, e2): (&Edge, &Edge),
    tol_conf: f64,
) -> Result<Vertex, Failure> {
    let p = live(s, "surface")?.value(c.p2d.x(), c.p2d.y())?;
    let mut v = BRepLibMakeVertex::new(p)?.vertex()?;
    v.set_orientation(Orientation::Internal);
    let builder = BRepBuilder::default();
    builder.update_vertex_on_edge(
        &v,
        c.u1,
        &topods::edge(&e1.oriented(Orientation::Forward))?,
        tol_conf,
    )?;
    builder.update_vertex_on_edge(
        &v,
        c.u2,
        &topods::edge(&e2.oriented(Orientation::Forward))?,
        tol_conf,
    )?;
    Ok(v)
}

/// Of several vertices, only those of least and greatest parameter on `E1`.
fn keep_extreme_vertices(lv: &mut List<Shape>, e1: &Edge) -> Result<(), Failure> {
    if lv.len() <= 1 {
        return Ok(());
    }
    let forward = topods::edge(&e1.oriented(Orientation::Forward))?;
    let (mut vf, mut vl) = (Shape::default(), Shape::default());
    let (mut u_min, mut u_max) = (precision::infinite(), -precision::infinite());
    for cv in lv.iter() {
        let u = BRepTool::parameter(&topods::vertex(cv)?, &forward)?;
        if u < u_min {
            vf = cv.clone();
            u_min = u;
        }
        if u > u_max {
            vl = cv.clone();
            u_max = u;
        }
    }
    lv.clear();
    lv.append(vf);
    lv.append(vl);
    Ok(())
}

/// `ProjectVertexOnEdge(V, E, TolConf)`: the parameter of the vertex on the edge (its
/// first or last parameter by the vertex orientation when within the tolerance, else
/// the nearest projection) set on the forward edge; whether one was found.
pub(super) fn project_vertex_on_edge(v: &Vertex, e: &Edge, tol_conf: f64) -> Result<bool, Failure> {
    let p = BRepTool::pnt(v)?;
    let c = BRepAdaptorCurve::with_edge(e)?;
    let (f, l) = (c.first_parameter()?, c.last_parameter()?);
    let mut found = None;
    if v.orientation() == Orientation::Forward
        && f.abs() < precision::infinite()
        && c.value(f)?.is_equal(&p, tol_conf)
    {
        found = Some(f);
    }
    if v.orientation() == Orientation::Reversed
        && found.is_none()
        && l.abs() < precision::infinite()
        && c.value(l)?.is_equal(&p, tol_conf)
    {
        found = Some(l);
    }
    if found.is_none() {
        found = nearest_parameter_3d(p, &c)?;
    }
    let Some(u) = found else {
        return Ok(false);
    };
    let ee = topods::edge(&e.oriented(Orientation::Forward))?;
    let internal = topods::vertex(&v.oriented(Orientation::Internal))?;
    BRepBuilder::default().update_vertex_on_edge(
        &internal,
        u,
        &ee,
        BRepTool::edge_tolerance(e)?,
    )?;
    Ok(true)
}

/// The parameter of the nearest extremum of the point on the curve (the first of equal
/// distances), when the extrema find one.
fn nearest_parameter_3d(
    p: rustycad_tkmath::gp::Point3,
    c: &BRepAdaptorCurve,
) -> Result<Option<f64>, Failure> {
    let proj = ExtPC::with_curve(p, c, EXTREMA_TOL_F)?;
    if !proj.is_done() || proj.nb_ext()? == 0 {
        return Ok(None);
    }
    let mut dist2_min = proj.square_distance(1)?;
    let mut u = proj.point(1)?.parameter();
    for i in 2..=proj.nb_ext()? {
        let dist2 = proj.square_distance(i)?;
        if dist2 < dist2_min {
            dist2_min = dist2;
            u = proj.point(i)?.parameter();
        }
    }
    Ok(Some(u))
}

/// `TryParameter(OE, V, NE, TolConf)`: when the vertex is at an end of the old edge
/// whose parameter is strictly inside the new edge range and at the same point on the
/// new edge, that parameter set on the new edge; whether it was.
pub(super) fn try_parameter(
    oe: &Edge,
    v: &Vertex,
    ne: &Edge,
    tol_conf: f64,
) -> Result<bool, Failure> {
    let oc = BRepAdaptorCurve::with_edge(oe)?;
    let nc = BRepAdaptorCurve::with_edge(ne)?;
    let (of, ol) = (oc.first_parameter()?, oc.last_parameter()?);
    let (nf, nl) = (nc.first_parameter()?, nc.last_parameter()?);
    let p = BRepTool::pnt(v)?;
    let mut found = None;
    if p.distance(&oc.value(of)?) < tol_conf
        && of > nf
        && of < nl
        && p.distance(&nc.value(of)?) < tol_conf
    {
        found = Some(of);
    }
    if p.distance(&oc.value(ol)?) < tol_conf
        && ol > nf
        && ol < nl
        && p.distance(&nc.value(ol)?) < tol_conf
    {
        found = Some(ol);
    }
    let Some(u) = found else {
        return Ok(false);
    };
    let internal = topods::vertex(&v.oriented(Orientation::Internal))?;
    BRepBuilder::default().update_vertex_on_edge(
        &internal,
        u,
        ne,
        BRepTool::edge_tolerance(ne)?,
    )?;
    Ok(true)
}
