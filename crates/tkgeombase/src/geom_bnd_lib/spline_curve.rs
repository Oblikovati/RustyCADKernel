// Rust translation of GeomBndLib_BezierCurve and GeomBndLib_BSplineCurve at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_BezierCurve` and `GeomBndLib_BSplineCurve`: sampled boxes reduced to the
//! pole hull, and the sampled + optimized `BoxOptimal`.
use super::live;
use super::spline::{curve_box_optimal, fill_box, reduce_spline_box};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBezierCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::{std_max, std_min};
use rustycad_tkmath::elclib;

// `1.5`: the factor applied to the sampled deflection before the hull reduction.
const WEAKNESS: f64 = 1.5;

/// A `GeomAdaptor_Curve` on a typed curve handle; `adaptor_of(&bezier)?`.
pub(super) fn adaptor_of<T: GeomCurve + rustycad_tkernel::standard::RuntimeTyped>(
    curve: &Handle<T>,
) -> Result<GeomAdaptorCurve, Failure> {
    GeomAdaptorCurve::with_curve(&curve.cast::<dyn GeomCurve>())
}

/// `GeomBndLib_BezierCurve`: the box of a `Geom_BezierCurve`.
pub struct BezierCurve {
    geom: Handle<GeomBezierCurve>,
}

impl BezierCurve {
    /// The evaluator of `curve`; `BezierCurve::new(handle)`.
    pub fn new(curve: Handle<GeomBezierCurve>) -> Self {
        Self { geom: curve }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomBezierCurve> {
        &self.geom
    }
    /// `Box(Tol)`: the whole curve; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let curve = live(&self.geom, "GeomBndLib_BezierCurve")?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `Box(U1, U2, Tol)`: `Degree` sample pairs of `[U1, U2]`, enlarged by 1.5 times the
    /// deflection, clipped to the hull of all poles (also on a sub-range, a native
    /// looseness), enlarged by `tol`; `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let curve = live(&self.geom, "GeomBndLib_BezierCurve")?;
        let adaptor: GeomAdaptorCurve = adaptor_of(&self.geom)?;
        let mut sampled: BoundingBox3 = BoundingBox3::default();
        let adaptor: &dyn Adaptor3dCurve = &adaptor;
        let deflection: f64 = fill_box(&mut sampled, adaptor, u1, u2, curve.degree())?;
        sampled.enlarge(WEAKNESS * deflection);
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        reduce_spline_box(&curve.poles(), &sampled, &mut bounds);
        bounds.enlarge(tolerance);
        Ok(bounds)
    }
    /// `BoxOptimal(U1, U2, Tol)`: `CurveBoxOptimal` on a `GeomAdaptor_Curve`;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let adaptor: GeomAdaptorCurve = adaptor_of(&self.geom)?;
        let adaptor: &dyn Adaptor3dCurve = &adaptor;
        curve_box_optimal(adaptor, u1, u2, tolerance)
    }
}

/// `GeomBndLib_BSplineCurve`: the box of a `Geom_BSplineCurve`.
pub struct BSplineCurve {
    geom: Handle<GeomBSplineCurve>,
}

// The Segment tolerance of a range: 2 PConfusion, or 1% of the (cross-period) span
// when that is smaller.
fn segment_tolerance(curve: &GeomBSplineCurve, u1: f64, u2: f64) -> f64 {
    let default: f64 = 2.0 * precision::p_confusion();
    let span: f64 = if curve.is_periodic() {
        let period: f64 = curve.last_parameter() - curve.first_parameter();
        let direct: f64 = (u2 - u1).abs();
        let cross1: f64 = (u2 - period - u1).abs();
        let cross2: f64 = (u1 - period - u2).abs();
        std_min(direct, std_min(cross1, cross2))
    } else {
        (u2 - u1).abs()
    };
    if span < default { span * 0.01 } else { default }
}

// The native copy segmented to `[u1, u2]`: periodic ranges are brought into the
// period, others clamped to the curve.
fn segmented_copy(
    curve: &GeomBSplineCurve,
    u1: f64,
    u2: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let copy: GeomBSplineCurve = GeomBSplineCurve::copy_of(curve)?;
    let (first, last) = (copy.first_parameter(), copy.last_parameter());
    let (a1, a2) = if copy.is_periodic() {
        elclib::adjust_periodic(first, last, precision::p_confusion(), u1, u2)
    } else {
        (
            if first > u1 { first } else { u1 },
            if last < u2 { last } else { u2 },
        )
    };
    let tolerance: f64 = segment_tolerance(&copy, a1, a2);
    copy.segment(a1, a2, tolerance)?;
    Ok(Handle::new(copy))
}

impl BSplineCurve {
    /// The evaluator of `curve`; `BSplineCurve::new(handle)`.
    pub fn new(curve: Handle<GeomBSplineCurve>) -> Self {
        Self { geom: curve }
    }
    /// `Geometry()`; `evaluator.geometry()`.
    pub fn geometry(&self) -> &Handle<GeomBSplineCurve> {
        &self.geom
    }
    /// `Box(Tol)`: the whole curve; `evaluator.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        let curve = live(&self.geom, "GeomBndLib_BSplineCurve")?;
        self.bounding_box_range(curve.first_parameter(), curve.last_parameter(), tolerance)
    }
    /// `Box(U1, U2, Tol)`: a range off the curve bounds (by more than
    /// `Parametric(tol)`) is segmented on a copy; each knot span is sampled `Degree`
    /// times, the box enlarged by 1.5 times the deflection and clipped to the hull of
    /// the ORIGINAL curve's poles (a native looseness on sub-ranges);
    /// `evaluator.bounding_box_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let original = live(&self.geom, "GeomBndLib_BSplineCurve")?;
        let limit: f64 = precision::parametric(tolerance);
        let mut curve: Handle<GeomBSplineCurve> = self.geom.clone();
        if (original.first_parameter() - u1).abs() > limit
            || (original.last_parameter() - u2).abs() > limit
        {
            curve = segmented_copy(original, u1, u2)?;
        }
        let (sampled, deflection) = sample_knot_spans(&curve)?;
        let mut bounds: BoundingBox3 = BoundingBox3::default();
        if !sampled.is_void() {
            let mut sampled: BoundingBox3 = sampled;
            sampled.enlarge(WEAKNESS * deflection);
            reduce_spline_box(&original.poles(), &sampled, &mut bounds);
            bounds.enlarge(tolerance);
        }
        Ok(bounds)
    }
    /// `BoxOptimal(U1, U2, Tol)`: `CurveBoxOptimal` on a `GeomAdaptor_Curve`;
    /// `evaluator.bounding_box_optimal_range(0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u1: f64,
        u2: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        let adaptor: GeomAdaptorCurve = adaptor_of(&self.geom)?;
        let adaptor: &dyn Adaptor3dCurve = &adaptor;
        curve_box_optimal(adaptor, u1, u2, tolerance)
    }
}

// `FillBox` over every knot span `[FirstUKnotIndex, LastUKnotIndex]`; the largest
// deflection.
fn sample_knot_spans(curve: &Handle<GeomBSplineCurve>) -> Result<(BoundingBox3, f64), Failure> {
    let spline = live(curve, "GeomBndLib_BSplineCurve")?;
    let (first_knot, last_knot) = (spline.first_u_knot_index()?, spline.last_u_knot_index()?);
    let knots = spline.knots();
    let adaptor: GeomAdaptorCurve = adaptor_of(curve)?;
    let adaptor: &dyn Adaptor3dCurve = &adaptor;
    let mut sampled: BoundingBox3 = BoundingBox3::default();
    let mut deflection: f64 = 0.0;
    let mut first: f64 = knots[first_knot];
    for knot in first_knot + 1..=last_knot {
        let last: f64 = knots[knot];
        let span: f64 = fill_box(&mut sampled, adaptor, first, last, spline.degree())?;
        deflection = std_max(span, deflection);
        first = last;
    }
    Ok((sampled, deflection))
}
