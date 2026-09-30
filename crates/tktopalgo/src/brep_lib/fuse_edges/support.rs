// Rust translation of BRepLib_FuseEdges::SameSupport (BRepLib_FuseEdges.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Whether two edges lie on the same line, circle, ellipse, B-spline or Bezier curve.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCircle, GeomCurve, GeomEllipse, GeomLine,
    GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Point3, Vector3};
use std::cmp::Ordering::Greater;

/// A curve of `SameSupport`: moved by its location, the basis of a trimmed curve, with
/// the range of the edge.
struct Support {
    curve: Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
}

/// The support of an edge, None without 3d curve (a degenerated edge).
fn support(edge: &Edge) -> Result<Option<Support>, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    let Some(curve) = found.curve.get() else {
        return Ok(None);
    };
    let mut moved: Handle<dyn GeomCurve> = found.curve.clone();
    if !found.location.is_identity() {
        moved = curve.transformed(found.location.transformation())?.cast();
    }
    if let Some(trimmed) = moved.cast::<GeomTrimmedCurve>().get() {
        moved = trimmed.basis_curve();
    }
    Ok(Some(Support {
        curve: moved,
        first: found.first,
        last: found.last,
    }))
}

/// `SameSupport(E1, E2)`: the same curve type among line, circle, ellipse, B-spline
/// and Bezier (after the basis of a trimmed curve), compared by its kind.
pub(super) fn same_support(e1: &Edge, e2: &Edge, concat_bspl: bool) -> Result<bool, Failure> {
    if e1.is_null() || e2.is_null() {
        return Ok(false);
    }
    let (Some(s1), Some(s2)) = (support(e1)?, support(e2)?) else {
        return Ok(false);
    };
    if let (Some(l1), Some(l2)) = (
        s1.curve.cast::<GeomLine>().get(),
        s2.curve.cast::<GeomLine>().get(),
    ) {
        return same_line(l1, l2, e1, e2);
    }
    if let (Some(c1), Some(c2)) = (
        s1.curve.cast::<GeomCircle>().get(),
        s2.curve.cast::<GeomCircle>().get(),
    ) {
        let (ci1, ci2) = (c1.circ()?, c2.circ()?);
        return Ok(same_conic(
            &[(ci1.radius(), ci2.radius())],
            (ci1.location(), ci2.location()),
            ci1.axis().is_parallel(ci2.axis(), precision::angular()),
        ));
    }
    if let (Some(c1), Some(c2)) = (
        s1.curve.cast::<GeomEllipse>().get(),
        s2.curve.cast::<GeomEllipse>().get(),
    ) {
        let (el1, el2) = (c1.elips()?, c2.elips()?);
        let radii = [
            (el1.major_radius(), el2.major_radius()),
            (el1.minor_radius(), el2.minor_radius()),
        ];
        return Ok(same_conic(
            &radii,
            (el1.location(), el2.location()),
            el1.axis().is_parallel(el2.axis(), precision::angular()),
        ));
    }
    if let (Some(b1), Some(b2)) = (
        s1.curve.cast::<GeomBSplineCurve>().get(),
        s2.curve.cast::<GeomBSplineCurve>().get(),
    ) {
        if concat_bspl && g1_joined(&s1, &s2)? {
            return Ok(true);
        }
        return Ok(joined_ranges(&s1, &s2) && same_bspline(b1, b2, BRepTool::edge_tolerance(e1)?)?);
    }
    if let (Some(b1), Some(b2)) = (
        s1.curve.cast::<GeomBezierCurve>().get(),
        s2.curve.cast::<GeomBezierCurve>().get(),
    ) {
        return same_bezier(b1, b2, joined_ranges(&s1, &s2));
    }
    Ok(false)
}

/// Two lines: parallel, and not two segments looping on each other (both ends of one
/// on the ends of the other).
fn same_line(l1: &GeomLine, l2: &GeomLine, e1: &Edge, e2: &Edge) -> Result<bool, Failure> {
    let (dir1, dir2) = (l1.lin().direction(), l2.lin().direction());
    if !dir1.is_parallel(dir2, precision::angular()) {
        return Ok(false);
    }
    let pnt = |e: &Edge, first: bool| -> Result<Point3, Failure> {
        BRepTool::pnt(&if first {
            topexp::first_vertex(e, true)
        } else {
            topexp::last_vertex(e, true)
        })
    };
    let tollin: f64 = precision::confusion();
    let (pf1, pl1, pf2, pl2) = (
        pnt(e1, true)?,
        pnt(e1, false)?,
        pnt(e2, true)?,
        pnt(e2, false)?,
    );
    Ok(pl1.distance(&pf2) >= tollin || pl2.distance(&pf1) >= tollin)
}

/// Two circles or ellipses: equal radii and centres within Precision::Confusion() and
/// parallel axes.
fn same_conic(radii: &[(f64, f64)], (c1, c2): (Point3, Point3), parallel: bool) -> bool {
    let tollin: f64 = precision::confusion();
    radii.iter().all(|(r1, r2)| (r1 - r2).abs() <= tollin)
        && c1.square_distance(&c2) <= tollin * tollin
        && parallel
}

/// "Check G1 continuity": an end of one curve on an end of the other (last on first,
/// first on last, first on first, last on last) with parallel derivatives.
fn g1_joined(s1: &Support, s2: &Support) -> Result<bool, Failure> {
    let (c1, c2) = (
        s1.curve.get().expect("a curve"),
        s2.curve.get().expect("a curve"),
    );
    let (pf1, df1) = c1.d1(s1.first)?;
    let (pl1, dl1) = c1.d1(s1.last)?;
    let (pf2, df2) = c2.d1(s2.first)?;
    let (pl2, dl2) = c2.d1(s2.last)?;
    let pairs: [(Point3, Vector3, Point3, Vector3); 4] = [
        (pl1, dl1, pf2, df2),
        (pl2, dl2, pf1, df1),
        (pf1, df1, pf2, df2),
        (pl1, dl1, pl2, dl2),
    ];
    for (p, d, q, e) in pairs {
        if p.distance(&q) <= precision::confusion() && d.is_parallel(e, precision::angular())? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// "the end of one curve does not correspond to the beginning of the second": the
/// last parameter of one is the first of the other.
fn joined_ranges(s1: &Support, s2: &Support) -> bool {
    let tollin: f64 = precision::confusion();
    !((s2.first - s1.last).abs() > tollin && (s1.first - s2.last).abs() > tollin)
}

/// Two B-splines: the same poles (within the edge tolerance), knots (a signed
/// difference, as natively), multiplicities, rationality and weights.
fn same_bspline(b1: &GeomBSplineCurve, b2: &GeomBSplineCurve, tol3d: f64) -> Result<bool, Failure> {
    let tollin: f64 = precision::confusion();
    let nbpoles: i32 = b1.nb_poles();
    if nbpoles != b2.nb_poles() || b1.nb_knots() != b2.nb_knots() {
        return Ok(false);
    }
    let (p1, p2) = (b1.poles(), b2.poles());
    if (1..=nbpoles).any(|p| p1[p].distance(&p2[p]) > tol3d) {
        return Ok(false);
    }
    let (k1, k2, m1, m2) = (
        b1.knots(),
        b2.knots(),
        b1.multiplicities(),
        b2.multiplicities(),
    );
    for k in 1..=b1.nb_knots() {
        if k1[k] - k2[k] > tollin || f64::from((m1[k] - m2[k]).abs()) > tollin {
            return Ok(false);
        }
    }
    if b1.is_rational() != b2.is_rational() {
        return Ok(false);
    }
    Ok(!b1.is_rational() || same_weights(b1.weights(), b2.weights(), nbpoles))
}

/// Two Bezier curves: joined ranges, the same poles, rationality and weights.
fn same_bezier(b1: &GeomBezierCurve, b2: &GeomBezierCurve, joined: bool) -> Result<bool, Failure> {
    if !joined {
        return Ok(false);
    }
    let nbpoles: i32 = b1.nb_poles();
    if nbpoles != b2.nb_poles() {
        return Ok(false);
    }
    let (p1, p2) = (b1.poles(), b2.poles());
    if (1..=nbpoles).any(|p| p1[p].distance(&p2[p]) > precision::confusion()) {
        return Ok(false);
    }
    if b1.is_rational() != b2.is_rational() {
        return Ok(false);
    }
    Ok(!b1.is_rational() || same_weights(b1.weights(), b2.weights(), nbpoles))
}

/// The weights within Precision::Confusion() (both curves are rational here).
fn same_weights(w1: Option<Weights>, w2: Option<Weights>, n: i32) -> bool {
    let (w1, w2) = (w1.expect("a rational curve"), w2.expect("a rational curve"));
    (1..=n).all(|w| (w1[w] - w2[w]).abs().partial_cmp(&precision::confusion()) != Some(Greater))
}

/// The weights of a rational curve.
type Weights = rustycad_tkernel::ncollection::Array1<'static, f64>;
