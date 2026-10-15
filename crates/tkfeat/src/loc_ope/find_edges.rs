// Rust translation of ModelingAlgorithms/TKFeat/LocOpe/LocOpe_FindEdges.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `LocOpe_FindEdges`: the pairs of edges of two shapes lying on the same curve (lines,
//! circles, ellipses, B-spline and Bezier curves), the edge of the first shape inside the
//! one of the second.
use crate::errors::{not_done, null_object};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Point3, Vector3};
use std::f64::consts::PI;

/// The curve kinds the search compares.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Line,
    Circle,
    Ellipse,
    BSpline,
    Bezier,
}

/// The located 3D curve of an edge, a trimmed curve standing for its basis, and its
/// kind (None for the other curves).
pub(crate) fn basis_curve(edge: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let located = BRepTool::curve_with_location(edge)?;
    let mut c = located.curve;
    if !located.location.is_identity() {
        let curve = c
            .get()
            .ok_or_else(|| null_object("LocOpe_FindEdges::Set", "edge without 3D curve"))?;
        c = curve.transformed(located.location.transformation())?.cast();
    }
    let trimmed: Handle<GeomTrimmedCurve> = c.cast();
    if let Some(trimmed) = trimmed.get() {
        c = trimmed.basis_curve();
    }
    Ok((c, located.first, located.last))
}

fn kind_of(c: &Handle<dyn GeomCurve>) -> Option<Kind> {
    if !c.cast::<GeomLine>().is_null() {
        return Some(Kind::Line);
    }
    if !c.cast::<GeomCircle>().is_null() {
        return Some(Kind::Circle);
    }
    if !c.cast::<GeomEllipse>().is_null() {
        return Some(Kind::Ellipse);
    }
    if !c.cast::<GeomBSplineCurve>().is_null() {
        return Some(Kind::BSpline);
    }
    if !c.cast::<GeomBezierCurve>().is_null() {
        return Some(Kind::Bezier);
    }
    None
}

/// One edge with its basis curve and range.
struct EdgeCurve {
    c: Handle<dyn GeomCurve>,
    f: f64,
    l: f64,
}

fn in_range(prm: f64, t: &EdgeCurve, tol: f64) -> bool {
    prm >= t.f - tol && prm <= t.l + tol
}

/// Two lines: the ends of the first lie on the range of the second.
fn same_lines(cf: &EdgeCurve, ct: &EdgeCurve) -> Result<bool, Failure> {
    let mut tol = precision::confusion();
    let lif =
        cf.c.cast::<GeomLine>()
            .get()
            .map(|l| l.lin())
            .unwrap_or_default();
    let lit =
        ct.c.cast::<GeomLine>()
            .get()
            .map(|l| l.lin())
            .unwrap_or_default();
    let p1 = elclib::line_value(cf.f, lif.position());
    let p2 = elclib::line_value(cf.l, lif.position());
    let prm1 = elclib::line_parameter(lit.position(), p1);
    let prm2 = elclib::line_parameter(lit.position(), p2);
    if !(in_range(prm1, ct, tol) && in_range(prm2, ct, tol)) {
        return Ok(false);
    }
    tol *= tol;
    if elclib::line_value(prm1, lit.position()).square_distance(&p1) > tol {
        return Ok(false);
    }
    Ok(elclib::line_value(prm2, lit.position()).square_distance(&p2) <= tol)
}

/// The parameters on the second conic of the ends of the first, set in the period
/// starting at its first parameter and ordered along the first one: the start point
/// with its tangent and the end point on the first conic, and the parameter and
/// tangent evaluators of the second.
fn conic_parameters(
    first: ((Point3, Vector3), Point3),
    ct: &EdgeCurve,
    snap_start: bool,
    parameter: impl Fn(Point3) -> Result<f64, Failure>,
    d1: impl Fn(f64) -> (Point3, Vector3),
) -> Result<(f64, f64), Failure> {
    let ((p1, tgf), p2) = first;
    let mut prm1 = parameter(p1)?;
    if snap_start && (prm1 - ct.f).abs() <= precision::p_confusion() {
        prm1 = ct.f;
    }
    prm1 = elclib::in_period(prm1, ct.f, ct.f + 2. * PI);
    let (_, tgt) = d1(prm1);
    let mut prm2 = parameter(p2)?;
    if tgt.dot(tgf) > 0. {
        // meme sens
        while prm2 <= prm1 {
            prm2 += 2. * PI;
        }
    } else {
        if (prm1 - ct.f).abs() <= precision::angular() {
            prm1 += 2. * PI;
        }
        while prm2 >= prm1 {
            prm2 -= 2. * PI;
        }
    }
    Ok((prm1, prm2))
}

/// Two circles of the same radius and centre: the arc of the first inside the second
/// (the "a cheval" case is not handled).
fn same_circles(cf: &EdgeCurve, ct: &EdgeCurve) -> Result<bool, Failure> {
    let tol = precision::confusion();
    let null = || null_object("LocOpe_FindEdges::Set", "null circle");
    let cif = cf.c.cast::<GeomCircle>().get().ok_or_else(null)?.circ()?;
    let cit = ct.c.cast::<GeomCircle>().get().ok_or_else(null)?.circ()?;
    if (cif.radius() - cit.radius()).abs() > tol
        || cif.location().square_distance(&cit.location()) > tol * tol
    {
        return Ok(false);
    }
    // Point debut, calage dans periode, et detection meme sens
    let start = elclib::circle_d1(cf.f, cif.position(), cif.radius());
    let end = elclib::circle_value(cf.l, cif.position(), cif.radius());
    let (prm1, prm2) = conic_parameters(
        (start, end),
        ct,
        true,
        |p| Ok(elclib::circle_parameter(cit.position(), p)),
        |u| elclib::circle_d1(u, cit.position(), cit.radius()),
    )?;
    Ok(in_range(prm1, ct, tol) && in_range(prm2, ct, tol))
}

/// Two ellipses of the same radii and centre, as two circles (without the snap of the
/// start parameter).
fn same_ellipses(cf: &EdgeCurve, ct: &EdgeCurve) -> Result<bool, Failure> {
    let tol = precision::confusion();
    let null = || null_object("LocOpe_FindEdges::Set", "null ellipse");
    let cif = cf.c.cast::<GeomEllipse>().get().ok_or_else(null)?.elips()?;
    let cit = ct.c.cast::<GeomEllipse>().get().ok_or_else(null)?.elips()?;
    if (cif.major_radius() - cit.major_radius()).abs() > tol
        || (cif.minor_radius() - cit.minor_radius()).abs() > tol
        || cif.location().square_distance(&cit.location()) > tol * tol
    {
        return Ok(false);
    }
    let start = elclib::ellipse_d1(cf.f, cif.position(), cif.major_radius(), cif.minor_radius());
    let end = elclib::ellipse_value(cf.l, cif.position(), cif.major_radius(), cif.minor_radius());
    let (prm1, prm2) = conic_parameters(
        (start, end),
        ct,
        false,
        |p| elclib::ellipse_parameter(cit.position(), cit.major_radius(), cit.minor_radius(), p),
        |u| elclib::ellipse_d1(u, cit.position(), cit.major_radius(), cit.minor_radius()),
    )?;
    Ok(in_range(prm1, ct, tol) && in_range(prm2, ct, tol))
}

/// Two B-spline curves with the same poles (within the tolerance of `edgt`), knots,
/// multiplicities, rationality and weights. Native quirk kept: the knots compare by
/// their signed difference.
fn same_bsplines(cf: &EdgeCurve, ct: &EdgeCurve, edgt: &Edge) -> Result<bool, Failure> {
    let tol = precision::confusion();
    let null = || null_object("LocOpe_FindEdges::Set", "null B-spline curve");
    let bf = cf.c.cast::<GeomBSplineCurve>();
    let bt = ct.c.cast::<GeomBSplineCurve>();
    let (bf, bt) = (bf.get().ok_or_else(null)?, bt.get().ok_or_else(null)?);
    let nbpoles = bf.nb_poles();
    if nbpoles != bt.nb_poles() || bf.nb_knots() != bt.nb_knots() {
        return Ok(false);
    }
    let tol3d = BRepTool::edge_tolerance(edgt)?;
    for p in 1..=nbpoles {
        if bf.pole(p)?.distance(&bt.pole(p)?) > tol3d {
            return Ok(false);
        }
    }
    for k in 1..=bf.nb_knots() {
        if bf.knot(k)? - bt.knot(k)? > tol
            || f64::from((bf.multiplicity(k)? - bt.multiplicity(k)?).abs()) > tol
        {
            return Ok(false);
        }
    }
    if bf.is_rational() != bt.is_rational() {
        return Ok(false);
    }
    if bf.is_rational() {
        for w in 1..=nbpoles {
            if (bf.weight(w)? - bt.weight(w)?).abs() > tol {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Two Bezier curves with the same poles, rationality and weights.
fn same_beziers(cf: &EdgeCurve, ct: &EdgeCurve) -> Result<bool, Failure> {
    let tol = precision::confusion();
    let null = || null_object("LocOpe_FindEdges::Set", "null Bezier curve");
    let bf = cf.c.cast::<GeomBezierCurve>();
    let bt = ct.c.cast::<GeomBezierCurve>();
    let (bf, bt) = (bf.get().ok_or_else(null)?, bt.get().ok_or_else(null)?);
    let nbpoles = bf.nb_poles();
    if nbpoles != bt.nb_poles() {
        return Ok(false);
    }
    for p in 1..=nbpoles {
        if bf.pole(p)?.distance(&bt.pole(p)?) > tol {
            return Ok(false);
        }
    }
    if bf.is_rational() != bt.is_rational() {
        return Ok(false);
    }
    if bf.is_rational() {
        for w in 1..=nbpoles {
            if (bf.weight(w)? - bt.weight(w)?).abs() > tol {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// Whether the edge curve `cf` lies on `ct` (presumption of confusion by kind).
fn confused(kind: Kind, cf: &EdgeCurve, ct: &EdgeCurve, edgt: &Edge) -> Result<bool, Failure> {
    match kind {
        Kind::Line => same_lines(cf, ct),
        Kind::Circle => same_circles(cf, ct),
        Kind::Ellipse => same_ellipses(cf, ct),
        Kind::BSpline => same_bsplines(cf, ct, edgt),
        Kind::Bezier => same_beziers(cf, ct),
    }
}

/// `LocOpe_FindEdges`: the pairs of confused edges.
///
/// ```
/// use rustycad_tkfeat::loc_ope::FindEdges;
/// let mut find = FindEdges::new();
/// find.init_iterator();
/// assert!(!find.more());
/// ```
#[derive(Clone, Default)]
pub struct FindEdges {
    f_from: Shape,
    f_to: Shape,
    l_from: Vec<Shape>,
    l_to: Vec<Shape>,
    it: usize,
}

impl FindEdges {
    /// `LocOpe_FindEdges()`; `FindEdges::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `LocOpe_FindEdges(FFrom, FTo)`: set; `FindEdges::with_shapes(&from, &to)?`.
    pub fn with_shapes(f_from: &Shape, f_to: &Shape) -> Result<Self, Failure> {
        let mut find = Self::new();
        find.set(f_from, f_to)?;
        Ok(find)
    }

    /// The first edge of `f_to` the curve `cf` of an edge of `f_from` lies on.
    fn match_of(&self, kind: Kind, cf: &EdgeCurve) -> Result<Option<Edge>, Failure> {
        for t in Explorer::new(&self.f_to, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edgt = topods::edge(&t)?;
            let (c, f, l) = basis_curve(&edgt)?;
            if kind_of(&c) != Some(kind) {
                continue;
            }
            // On a presomption de confusion
            if confused(kind, cf, &EdgeCurve { c, f, l }, &edgt)? {
                return Ok(Some(edgt));
            }
        }
        Ok(None)
    }

    /// `Set(FFrom, FTo)`: for each edge of `f_from` on a line, circle, ellipse,
    /// B-spline or Bezier curve, the first edge of `f_to` on the same curve containing
    /// it; `find.set(&from, &to)?`.
    pub fn set(&mut self, f_from: &Shape, f_to: &Shape) -> Result<(), Failure> {
        self.f_from = f_from.clone();
        self.f_to = f_to.clone();
        self.l_from.clear();
        self.l_to.clear();
        for e in Explorer::new(&self.f_from, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edgf = topods::edge(&e)?;
            let (c, f, l) = basis_curve(&edgf)?;
            let Some(kind) = kind_of(&c) else {
                continue;
            };
            if let Some(edgt) = self.match_of(kind, &EdgeCurve { c, f, l })? {
                self.l_from.push(e);
                self.l_to.push(edgt.into_shape());
            }
        }
        Ok(())
    }

    /// `InitIterator()`; `find.init_iterator()`.
    pub fn init_iterator(&mut self) {
        self.it = 0;
    }

    /// `More()`; `find.more()`.
    pub fn more(&self) -> bool {
        self.it < self.l_from.len()
    }

    /// `EdgeFrom()`: the edge of the first shape; `find.edge_from()?`.
    pub fn edge_from(&self) -> Result<Edge, Failure> {
        let e = self
            .l_from
            .get(self.it)
            .ok_or_else(|| not_done("LocOpe_FindEdges::EdgeFrom"))?;
        topods::edge(e)
    }

    /// `EdgeTo()`: the edge of the second shape it lies on; `find.edge_to()?`.
    pub fn edge_to(&self) -> Result<Edge, Failure> {
        let e = self
            .l_to
            .get(self.it)
            .ok_or_else(|| not_done("LocOpe_FindEdges::EdgeTo"))?;
        topods::edge(e)
    }

    /// `Next()`; `find.next()`.
    pub fn next(&mut self) {
        self.it += 1;
    }
}
