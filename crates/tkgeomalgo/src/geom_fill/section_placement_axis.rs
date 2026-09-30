// Rust translation of the constructor body and the static Tangente of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SectionPlacement.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The size and the axis of a section: `Gabarit` (half the box diagonal) and
//! `TheAxe` (the conic axis, or the normal of the inertia plane of sample points).
use super::section_placement::GeomFillSectionPlacement;
use super::sweep_failures::live;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::bnd_lib::Add3dCurve;
use rustycad_tkgeombase::geom_lib::axe_of_inertia;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Axis1, Direction3, Point3, Vector3};

/// The sample count on each knot span of a B-spline section.
const NB_LOCAL_PNTS: i32 = 10;

/// The static `Tangente(Path, Param, P, Tang)`: the point and the first derivative
/// of order 1 to 11 whose norm reaches Precision::Confusion(), unit when not null.
pub(crate) fn tangente(
    path: &dyn Adaptor3dCurve,
    param: f64,
) -> Result<(Point3, Vector3), Failure> {
    let (p, mut tang) = path.d1(param)?;
    let mut norm = tang.magnitude();
    let mut ii = 2;
    while ii < 12 && norm < precision::confusion() {
        tang = path.dn(param, ii)?;
        norm = tang.magnitude();
        ii += 1;
    }
    if norm > 100.0 * gp::resolution() {
        tang /= norm;
    }
    Ok((p, tang))
}

/// `Gabarit`: half the diagonal of the box of the section (the curve box enlarged
/// by 1e-4) plus Precision::Confusion(), so a tiny section keeps a size.
pub(crate) fn section_size(placement: &GeomFillSectionPlacement) -> Result<f64, Failure> {
    let mut bbox = BoundingBox3::default();
    if placement.my_is_point {
        bbox.add_point(placement.my_point);
    } else {
        Add3dCurve::add(placement.section_adaptor(), 1.0e-4, &mut bbox)?;
    }
    let [xmin, ymin, zmin, xmax, ymax, zmax] = bbox.get()?;
    let (dx, dy, dz) = (xmax - xmin, ymax - ymin, zmax - zmin);
    let gabarit = (dx * dx + dy * dy + dz * dz).sqrt() / 2.0;
    Ok(gabarit + precision::confusion())
}

/// `TheAxe` and `isplan` of a curve section.
pub(crate) struct SectionAxisInit {
    pub axe: Axis1,
    pub isplan: bool,
}

impl SectionAxisInit {
    /// The tangent at mid range, replaced by the conic axis, or by the normal of the
    /// inertia plane of sample points when the curve has more than two poles and
    /// the points are not on a line.
    pub fn of(adp: &GeomAdaptorCurve) -> Result<Self, Failure> {
        let (first, last) = (adp.first_parameter()?, adp.last_parameter()?);
        let (p, v) = tangente(adp, (first + last) / 2.0)?;
        let axe = Axis1::new(p, Direction3::from_vector(v)?);
        let nb_poles = match adp.get_type()? {
            CurveType::Circle => return Ok(Self::plan(adp.circle()?.axis())),
            CurveType::Ellipse => return Ok(Self::plan(adp.ellipse()?.axis())),
            CurveType::Hyperbola => return Ok(Self::plan(adp.hyperbola()?.axis())),
            CurveType::Parabola => return Ok(Self::plan(adp.parabola()?.axis())),
            CurveType::Line => 0,
            CurveType::BezierCurve | CurveType::BSplineCurve => adp.nb_poles()?,
            _ => 21,
        };
        if nb_poles <= 2 {
            return Ok(Self { axe, isplan: false });
        }
        let points = sample_points(adp, nb_poles)?;
        let inertia = axe_of_inertia(&points, precision::confusion())?;
        if inertia.is_singular {
            return Ok(Self { axe, isplan: false });
        }
        let plane_axe = Axis1::new(inertia.axe.location(), inertia.axe.direction());
        Ok(Self::plan(plane_axe))
    }

    fn plan(axe: Axis1) -> Self {
        Self { axe, isplan: true }
    }
}

/// The section range, snapped to a whole period of the basis curve when an end
/// is within Precision::PConfusion() of a period bound.
fn sampled_range(adp: &GeomAdaptorCurve) -> Result<(f64, f64), Failure> {
    let (mut first, mut last) = (adp.first_parameter()?, adp.last_parameter()?);
    if !adp.is_periodic()? {
        return Ok((first, last));
    }
    let mut a_curve: Handle<dyn GeomCurve> = adp.curve();
    let trimmed: Handle<GeomTrimmedCurve> = a_curve.cast();
    if let Some(t) = trimmed.get() {
        a_curve = t.basis_curve();
    }
    let c = live(&a_curve, "GeomFill_SectionPlacement", "aCurve")?;
    let (ufirst, a_period) = (c.first_parameter(), c.period()?);
    let u1 = ufirst + ((first - ufirst) / a_period).floor() * a_period;
    let u2 = u1 + a_period;
    if (first - u1).abs() <= precision::p_confusion() {
        first = u1;
    }
    if (last - u2).abs() <= precision::p_confusion() {
        last = u2;
    }
    Ok((first, last))
}

/// The points whose inertia plane gives the axis: ten per knot span of a B-spline,
/// `NbPoles - 1` equal steps otherwise, and the last point of an open curve.
fn sample_points(
    adp: &GeomAdaptorCurve,
    nb_poles: i32,
) -> Result<Array1<'static, Point3>, Failure> {
    let range = sampled_range(adp)?;
    if adp.get_type()? == CurveType::BSplineCurve {
        return bspline_points(adp, range);
    }
    let (first, last) = range;
    let closed = adp.is_closed()?;
    let nb_pnts = if closed { nb_poles - 1 } else { nb_poles };
    let mut pnts = one_based(nb_pnts)?;
    let delta = (last - first) / f64::from(nb_poles - 1);
    for i in 0..nb_poles - 1 {
        put(&mut pnts, i + 1, adp.value(first + f64::from(i) * delta)?)?;
    }
    if !closed {
        put(&mut pnts, nb_pnts, adp.value(last)?)?;
    }
    Ok(pnts)
}

/// The knot intervals `(I1, I2)` and `(I3, I4)` of the range ends.
struct SpanBounds {
    i1: i32,
    i2: i32,
    i3: i32,
    tail: bool,
}

fn span_bounds(bc: &GeomBSplineCurve, first: f64, last: f64) -> Result<SpanBounds, Failure> {
    let conf = precision::confusion();
    let (i1, i2) = bc.locate_u(first, conf, false)?;
    let (i3, i4) = bc.locate_u(last, conf, false)?;
    let tail = i3 != i4 && first < bc.knot(i3)?;
    Ok(SpanBounds { i1, i2, i3, tail })
}

impl SpanBounds {
    /// Ten points per whole span, per partial span at each end, and the last point
    /// of an open curve.
    fn point_count(&self, closed: bool) -> i32 {
        let nb_knots = self.i3 - self.i2 + 1;
        let mut nb_pnts = ((nb_knots - 1) * NB_LOCAL_PNTS).max(0);
        nb_pnts += if self.i1 != self.i2 { NB_LOCAL_PNTS } else { 0 };
        nb_pnts += if self.tail { NB_LOCAL_PNTS } else { 0 };
        nb_pnts + if closed { 0 } else { 1 }
    }
}

fn bspline_points(
    adp: &GeomAdaptorCurve,
    (first, last): (f64, f64),
) -> Result<Array1<'static, Point3>, Failure> {
    let curve = adp.curve();
    let spline: Handle<GeomBSplineCurve> = curve.cast();
    let bc = live(&spline, "GeomFill_SectionPlacement", "BC")?;
    let s = span_bounds(bc, first, last)?;
    let closed = adp.is_closed()?;
    let mut pnts = one_based(s.point_count(closed))?;
    let mut nb = 1;
    if s.i1 != s.i2 {
        nb = head_span(adp, &mut pnts, (first, last), bc.knot(s.i2)?)?;
    }
    for i in s.i2..s.i3 {
        let t = bc.knot(i)?;
        let delta = (bc.knot(i + 1)? - t) / f64::from(NB_LOCAL_PNTS);
        nb = walk_span(adp, &mut pnts, nb, t, delta)?;
    }
    if s.tail {
        let t = bc.knot(s.i3)?;
        nb = walk_span(adp, &mut pnts, nb, t, (last - t) / f64::from(NB_LOCAL_PNTS))?;
    }
    if !closed {
        put(&mut pnts, nb, adp.value(last)?)?;
    }
    Ok(pnts)
}

/// Ten points `first + j delta` of the partial span before the knot `knot` (or
/// `last`), stored from 1; returns the next index.
fn head_span(
    adp: &GeomAdaptorCurve,
    pnts: &mut Array1<'static, Point3>,
    (first, last): (f64, f64),
    knot: f64,
) -> Result<i32, Failure> {
    let locallast = if knot < last { knot } else { last };
    let delta = (locallast - first) / f64::from(NB_LOCAL_PNTS);
    for j in 0..NB_LOCAL_PNTS {
        put(pnts, j + 1, adp.value(first + f64::from(j) * delta)?)?;
    }
    Ok(NB_LOCAL_PNTS + 1)
}

/// Ten points from `t` by `delta` (the running sum, as natively) stored from `nb`;
/// returns the next index.
fn walk_span(
    adp: &GeomAdaptorCurve,
    pnts: &mut Array1<'static, Point3>,
    mut nb: i32,
    mut t: f64,
    delta: f64,
) -> Result<i32, Failure> {
    for _ in 0..NB_LOCAL_PNTS {
        put(pnts, nb, adp.value(t)?)?;
        nb += 1;
        t += delta;
    }
    Ok(nb)
}
