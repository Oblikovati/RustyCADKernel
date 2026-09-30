// Rust translation of BRepTopAdaptor_FClass2d.cxx (wire sampling) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The first polygon of a wire in `BRepTopAdaptor_FClass2d`: `NbSamples` (times 4)
//! points per edge along its pcurve, skipping the points whose 3d point repeats the
//! previous one, with the largest deflection of the middle point of each triple.
use super::fclass2d::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorCurve2d};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::next_after;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_int::Geom2dCurveTool;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Point3, Vector2};

/// The polygon of one wire and the counters of its sampling.
pub(super) struct WirePolygon {
    pub(super) points: LinearVector<Point2>,
    pub(super) nbpnts: i32,
    pub(super) fleche_u: f64,
    pub(super) fleche_v: f64,
    pub(super) wire_is_not_empty: bool,
    /// `aNbE`: the edges of the wire counted by TopExp_Explorer.
    pub(super) nb_edges: i32,
    /// `NbEdges` after the WireExplorer walk: non-zero when the walk missed edges.
    pub(super) nb_edges_left: i32,
}

/// The state that runs along the edges of one wire.
struct EdgeWalk {
    firstpoint: i32,
    ancien_pnt3d: Point3,
    ancien_pnt3d_initialise: bool,
}

/// The pcurve and 3d adaptors of one edge and its sampling step.
struct EdgeSampling {
    curve2d: BRepAdaptorCurve2d,
    curve3d: BRepAdaptorCurve,
    degenerated: bool,
    nbs: i32,
    u: f64,
    du: f64,
}

impl WirePolygon {
    /// The polygon of `wire`, None when an edge has no pcurve (the constructor then
    /// returns); the face bounds of `classifier` grow with the sampled points.
    pub(super) fn sample(
        classifier: &mut BRepTopAdaptorFClass2d,
        wire: &Wire,
    ) -> Result<Option<Self>, Failure> {
        let nb_edges: i32 = Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)?.count() as i32;
        let mut polygon = Self::fallback(LinearVector::default(), 0.0, 0.0);
        (polygon.nb_edges, polygon.nb_edges_left) = (nb_edges, nb_edges);
        let mut walk = EdgeWalk {
            firstpoint: 1,
            ancien_pnt3d: Point3::new(0.0, 0.0, 0.0),
            ancien_pnt3d_initialise: false,
        };
        let face: Face = classifier.face.clone();
        let mut explorer = WireExplorer::with_wire_on_face(wire, &face)?;
        while explorer.more() {
            polygon.nb_edges_left -= 1;
            let edge: Edge = explorer.current().clone();
            if !polygon.sample_edge(classifier, &mut walk, &edge)? {
                return Ok(None);
            }
            explorer.next()?;
        }
        Ok(Some(polygon))
    }

    /// A polygon of the given points and deflections (the fallback of a bad wire).
    pub(super) fn fallback(points: LinearVector<Point2>, fleche_u: f64, fleche_v: f64) -> Self {
        Self {
            nbpnts: points.len() as i32,
            points,
            fleche_u,
            fleche_v,
            wire_is_not_empty: false,
            nb_edges: 0,
            nb_edges_left: 0,
        }
    }

    /// One FORWARD or REVERSED edge of the walk; false when it has no pcurve.
    fn sample_edge(
        &mut self,
        classifier: &mut BRepTopAdaptorFClass2d,
        walk: &mut EdgeWalk,
        edge: &Edge,
    ) -> Result<bool, Failure> {
        let orientation: Orientation = edge.orientation();
        if orientation != Orientation::Forward && orientation != Orientation::Reversed {
            return Ok(true);
        }
        let face: Face = classifier.face.clone();
        let found = BRepTool::curve_on_face(edge, &face)?;
        if found.pcurve.is_null() {
            return Ok(false);
        }
        let (pfbid, plbid) = (found.first, found.last);
        if (plbid - pfbid).abs() < 1.0e-9 {
            return Ok(true);
        }
        let mut sampling = EdgeSampling::new(edge, &face, (pfbid, plbid), orientation)?;
        //-- Check distance uv between the start point of the edge
        //-- and the last point registered in SeqPnt2d
        //-- Try to remote the first point of the current edge
        //-- from the last saved point
        if walk.firstpoint == 2 {
            sampling.u += sampling.du;
        }
        let avant: i32 = self.nbpnts;
        for _ in walk.firstpoint..=sampling.nbs {
            self.add_sample(classifier, walk, &mut sampling, avant)?;
        }
        if walk.firstpoint == 1 {
            walk.firstpoint = 2;
        }
        self.wire_is_not_empty = true;
        Ok(true)
    }

    /// One sample `e` of the edge: grows the face bounds, keeps the point unless its
    /// 3d point repeats the previous one, and updates the deflections.
    fn add_sample(
        &mut self,
        classifier: &mut BRepTopAdaptorFClass2d,
        walk: &mut EdgeWalk,
        sampling: &mut EdgeSampling,
        avant: i32,
    ) -> Result<(), Failure> {
        let u: f64 = sampling.u;
        let p2d: Point2 = sampling.curve2d.value(u)?;
        grow_bounds(classifier, p2d);
        let mut dist3dptcourant_ancienpnt: f64 = 1e+20; // RealLast();
        let mut p3d = Point3::default();
        if !sampling.degenerated {
            p3d = sampling.curve3d.value(u)?;
            if self.nbpnts > 1 && walk.ancien_pnt3d_initialise {
                dist3dptcourant_ancienpnt = p3d.distance(&walk.ancien_pnt3d);
            }
        }
        let mut is_real_curve3d: bool = true; // patch
        if dist3dptcourant_ancienpnt < precision::confusion() {
            let mid_p3d: Point3 = sampling.curve3d.value(u - sampling.du / 2.0)?;
            if p3d.distance(&mid_p3d) < precision::confusion() {
                is_real_curve3d = false;
            }
        }
        if is_real_curve3d {
            if !sampling.degenerated {
                walk.ancien_pnt3d = p3d;
                walk.ancien_pnt3d_initialise = true;
            }
            self.nbpnts += 1;
            self.points.append(p2d);
        }
        sampling.u += sampling.du;
        //  Modified by Sergey KHROMOV - Fri Apr 19 09:46:12 2002: a repeated end point
        //  (null chord) gives no deflection.
        let ii: usize = self.nbpnts as usize;
        if self.nbpnts > avant + 4
            && self.points[ii - 3].square_distance(self.points[ii - 1]) != 0.0
        {
            let (du, dv) = triple_deflection(&self.points, ii)?;
            self.fleche_u = if du > self.fleche_u {
                du
            } else {
                self.fleche_u
            };
            self.fleche_v = if dv > self.fleche_v {
                dv
            } else {
                self.fleche_v
            };
        }
        Ok(())
    }
}

impl EdgeSampling {
    /// The adaptors of `edge` on `face`, whether it counts as degenerated, and the
    /// samples: `NbSamples` of the pcurve (times 4 above 2) from the first parameter
    /// of the oriented edge.
    fn new(
        edge: &Edge,
        face: &Face,
        (pfbid, plbid): (f64, f64),
        orientation: Orientation,
    ) -> Result<Self, Failure> {
        let (va, vb) = topexp::vertices(edge, false);
        let mut degenerated: bool = BRepTool::degenerated(edge)?
            || BRepTool::is_closed_on_face(edge, face)?
            || va.is_null()
            || vb.is_null();
        let curve2d = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
        let curve3d = BRepAdaptorCurve::with_edge_on_face(edge, face)?;
        //-- Check cases when it was forgotten to code degenerated :  PRO17410 (janv 99)
        if !degenerated {
            degenerated = is_degenerated(&curve3d, pfbid, plbid)?;
        }
        let mut nbs: i32 = Geom2dCurveTool::nb_samples(&curve2d)?;
        //-- Attention to rational bsplines of degree 3. (ends of circles among others)
        if nbs > 2 {
            nbs *= 4;
        }
        let du: f64 = (plbid - pfbid) / (nbs - 1) as f64;
        let (u, du) = if orientation == Orientation::Forward {
            (pfbid, du)
        } else {
            (plbid, -du)
        };
        Ok(Self {
            curve2d,
            curve3d,
            degenerated,
            nbs,
            u,
            du,
        })
    }
}

/// The largest bounds of the sampled points, from the running face bounds.
fn grow_bounds(classifier: &mut BRepTopAdaptorFClass2d, p2d: Point2) {
    if p2d.x() < classifier.umin {
        classifier.umin = p2d.x();
    }
    if p2d.x() > classifier.umax {
        classifier.umax = p2d.x();
    }
    if p2d.y() < classifier.vmin {
        classifier.vmin = p2d.y();
    }
    if p2d.y() > classifier.vmax {
        classifier.vmax = p2d.y();
    }
}

/// The distances in U and V from the middle point of the last three points
/// `points[ii-3..ii]` to the chord through the other two.
pub(super) fn triple_deflection(
    points: &LinearVector<Point2>,
    ii: usize,
) -> Result<(f64, f64), Failure> {
    let (first, middle, last) = (points[ii - 3], points[ii - 2], points[ii - 1]);
    let lin = Line2::new(
        first,
        Direction2::from_vector(Vector2::between(first, last))?,
    );
    let ul: f64 = elclib::parameter(&lin, middle)?;
    let pp: Point2 = elclib::value(ul, &lin);
    Ok(((pp.x() - middle.x()).abs(), (pp.y() - middle.y()).abs()))
}

/// `isDegenerated`: whether the 3d curve stays within Precision::Confusion() (in
/// square distance) of its start point at ten steps of its range; a reversed range
/// takes no step and is degenerated.
fn is_degenerated(curve: &BRepAdaptorCurve, start: f64, end: f64) -> Result<bool, Failure> {
    let parametric_step: f64 = (end - start) * 0.1;
    let start_point: Point3 = curve.value(start)?;
    let mut current: f64 = start;
    while current < end {
        let current_point: Point3 = curve.value(current)?;
        if start_point.square_distance(&current_point) > precision::confusion() {
            return Ok(false);
        }
        current = safe_increment(current, end, parametric_step);
    }
    Ok(true)
}

/// `safeIncrement`: `value + increment`, or the next double towards `direction` when
/// the sum rounds back to `value`.
fn safe_increment(value: f64, direction: f64, increment: f64) -> f64 {
    let next: f64 = value + increment;
    if next == value {
        return next_after(value, direction);
    }
    next
}
