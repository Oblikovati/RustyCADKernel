// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `LocOpe` package functions: closed wires and edges on a face, tangent faces along
//! an edge and the sample points of a shape.
use crate::errors::null_object;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve2d, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// `NECHANT`: the number of sample intervals on each edge.
const NECHANT: i32 = 10;

/// The first edge of the FORWARD wire `w` holding `v` with `orientation`; the wire
/// itself when none does, as the native explorer returns its shape at the end.
fn edge_holding(w: &Wire, v: &Vertex, orientation: Orientation) -> Result<Edge, Failure> {
    let forward = w.oriented(Orientation::Forward);
    for e in Explorer::new(&forward, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let mut vertices = Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        if vertices.any(|x| x.is_same(v) && x.orientation() == orientation) {
            return topods::edge(&e);
        }
    }
    topods::edge(&forward)
}

/// The point of the p-curve of `e` on `f` at its first (`at_first`) or last end along
/// the orientation of `e`.
fn pcurve_end(e: &Edge, f: &Face, at_first: bool) -> Result<Point2, Failure> {
    let pc = BRepTool::curve_on_face(e, f)?;
    let c2d = pc
        .pcurve
        .get()
        .ok_or_else(|| null_object("LocOpe::Closed", "edge without p-curve on the face"))?;
    let forward = e.orientation() == Orientation::Forward;
    if forward == at_first {
        c2d.value(pc.first)
    } else {
        c2d.value(pc.last)
    }
}

/// `LocOpe::Closed(W, OnF)`: the wire `w` has the same first and last vertex and its
/// ends meet in the parametric space of `f`; `loc_ope::closed_wire(&w, &f)?`.
pub fn closed_wire(w: &Wire, f: &Face) -> Result<bool, Failure> {
    let (vf, vl) = topexp::wire_vertices(w);
    if !vf.is_same(&vl) {
        return Ok(false);
    }
    // On recherche l`edge contenant Vf FORWARD
    let ef = edge_holding(w, &vf, Orientation::Forward)?;
    // On recherche l`edge contenant Vl REVERSED
    let el = edge_holding(w, &vl, Orientation::Reversed)?;
    let pf = pcurve_end(&ef, f, true)?;
    let pl = pcurve_end(&el, f, false)?;
    Ok(pf.distance(pl) <= precision::p_confusion_with_speed(precision::confusion()))
}

/// `LocOpe::Closed(E, OnF)`: the edge `e` alone in a wire is closed on `f`;
/// `loc_ope::closed_edge(&e, &f)?`.
pub fn closed_edge(e: &Edge, f: &Face) -> Result<bool, Failure> {
    let b = BRepBuilder::default();
    let mut w = Wire::default();
    b.make_wire(&mut w);
    b.add(&mut w, &e.oriented(Orientation::Forward))?;
    closed_wire(&w, f)
}

/// The unit normal of `surface` at `(u, v)`, reversed when `reversed`.
fn unit_normal(
    surface: &BRepAdaptorSurface,
    u: f64,
    v: f64,
    reversed: bool,
) -> Result<Vector3, Failure> {
    let d1 = surface.d1(u, v)?;
    let d = d1.d1u.crossed(d1.d1v).normalized()?;
    Ok(if reversed { d.reversed() } else { d })
}

/// `LocOpe::TgtFaces(E, F1, F2)`: the normals of `f1` and `f2` make an angle of at
/// most 1e-4 at 21 points of `e` (its ends trimmed by 1% of its range);
/// `loc_ope::tgt_faces(&e, &f1, &f2)?`.
pub fn tgt_faces(e: &Edge, f1: &Face, f2: &Face) -> Result<bool, Failure> {
    let ta = 0.0001;
    let hs1 = BRepAdaptorSurface::with_face(f1, true)?;
    let hs2 = BRepAdaptorSurface::with_face(f2, true)?;
    let e = topods::edge(&e.oriented(Orientation::Forward))?;
    let hc2d = BRepAdaptorCurve2d::with_edge_on_face(&e, f1)?;
    let hc2d2 = BRepAdaptorCurve2d::with_edge_on_face(&e, f2)?;
    let rev1 = f1.orientation() == Orientation::Reversed;
    let rev2 = f2.orientation() == Orientation::Reversed;
    let (mut f, mut l) = BRepTool::range(&e)?;
    let (mut angmin, mut angmax) = (std::f64::consts::PI, -std::f64::consts::PI);
    let eps = (l - f) / 100.;
    f += eps; // pour eviter de faire des calculs sur les
    l -= eps; // pointes des carreaux pointus.
    for i in 0..=20 {
        let u = f + (l - f) * f64::from(i) / 20.;
        let p = hc2d.d0(u)?;
        let d1 = unit_normal(&hs1, p.x(), p.y(), rev1)?;
        let p = hc2d2.d0(u)?;
        let d2 = unit_normal(&hs2, p.x(), p.y(), rev2)?;
        let ang = d1.angle(d2)?;
        if ang <= angmin {
            angmin = ang;
        }
        if ang >= angmax {
            angmax = ang;
        }
    }
    Ok(angmax <= ta)
}

/// The NECHANT - 1 interior samples of the (located) 3D curve of `edge`.
fn edge_samples(edge: &Edge, seq: &mut Sequence<Point3>) -> Result<(), Failure> {
    let located = BRepTool::curve_with_location(edge)?;
    let curve = located
        .curve
        .get()
        .ok_or_else(|| null_object("LocOpe::SampleEdges", "edge without 3D curve"))?;
    let c: Handle<dyn GeomCurve> = curve.transformed(located.location.transformation())?.cast();
    let c = c
        .get()
        .ok_or_else(|| null_object("LocOpe::SampleEdges", "transformed geometry is not a curve"))?;
    let (f, l) = (located.first, located.last);
    let delta = (l - f) / f64::from(NECHANT) * 0.123456;
    for i in 1..NECHANT {
        let prm = delta + (f64::from(NECHANT - i) * f + f64::from(i) * l) / f64::from(NECHANT);
        seq.append(c.value(prm)?);
    }
    Ok(())
}

/// `LocOpe::SampleEdges(S, Pt)`: interior points of every non-degenerated edge of `s`
/// (not their extremities), then every vertex point, each sub-shape once;
/// `loc_ope::sample_edges(&shape, &mut points)?`.
pub fn sample_edges(s: &Shape, seq: &mut Sequence<Point3>) -> Result<(), Failure> {
    seq.clear(None);
    let mut seen = MapOfShape::default();
    // Computes points on edge, but does not take the extremities into account
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edg = topods::edge(&e)?;
        if !seen.add(e) {
            continue;
        }
        if !BRepTool::degenerated(&edg)? {
            edge_samples(&edg, seq)?;
        }
    }
    // Adds every vertex
    for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        if seen.add(v.clone()) {
            seq.append(BRepTool::pnt(&topods::vertex(&v)?)?);
        }
    }
    Ok(())
}
