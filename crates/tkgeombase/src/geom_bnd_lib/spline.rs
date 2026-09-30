// Rust translation of GeomBndLib_SplineHelpers.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_SplineHelpers`: sampled boxes with their deflection, the pole-hull
//! reduction and the sampled + optimized "optimal" curve box.
//!
//! The header also defines the templates `BezierCurveBox` and `BSplineCurveBox` and the
//! pole-range overload of `ReduceSplineBox` they use; no OCCT source instantiates them
//! (the Bezier and B-spline classes carry their own bodies), so they have no Rust
//! counterpart.
use super::space::{SampleBounds, SamplePoint, SampledCurve};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bvh::{std_max, std_min};

/// `FillBox`: add `2 * n + 1` points of `[first, last]` to `bounds` and return the
/// largest distance between a middle sample and the chord midpoint; a range shorter
/// than Precision::PConfusion() adds its two ends only;
/// `fill_box(&mut bounds, curve, 0.0, 1.0, 3)?`.
pub fn fill_box<C: ?Sized + SampledCurve>(
    bounds: &mut C::Bounds,
    curve: &C,
    first: f64,
    last: f64,
    n: i32,
) -> Result<f64, Failure> {
    bounds.add_sample(curve.sample_point(first)?);
    let span: f64 = last - first;
    // The native test is `abs(span) > PConfusion`, so a NaN span takes the two-ends branch.
    if span.abs() > precision::p_confusion() {
        return fill_samples(bounds, curve, first, span, n);
    }
    bounds.add_sample(curve.sample_point(last)?);
    Ok(0.0)
}

// The `n` sample pairs of FillBox and their largest deflection.
fn fill_samples<C: ?Sized + SampledCurve>(
    bounds: &mut C::Bounds,
    curve: &C,
    first: f64,
    span: f64,
    n: i32,
) -> Result<f64, Failure> {
    let step: f64 = span / f64::from(2 * n);
    let mut p: f64 = first;
    let mut tolerance: f64 = 0.0;
    let mut p1 = curve.sample_point(first)?;
    for _ in 1..=n {
        p += step;
        let p2 = curve.sample_point(p)?;
        bounds.add_sample(p2);
        p += step;
        let p3 = curve.sample_point(p)?;
        bounds.add_sample(p3);
        let centre = SamplePoint::mid(p1, p3);
        tolerance = std_max(tolerance, centre.distance_to(p2));
        p1 = p3;
    }
    Ok(tolerance)
}

// `Update` of the intersection of `original` with the hull box of `poles` (the original
// box alone when the hull is void).
fn intersect_with_hull<B: SampleBounds>(poles: B, original: &B, reduced: &mut B) {
    let (min, max) = original.min_max();
    if poles.void() {
        reduced.update_min_max(&min, &max);
        return;
    }
    let (pole_min, pole_max) = poles.min_max();
    let lower: Vec<f64> = min
        .iter()
        .zip(&pole_min)
        .map(|(&a, &b)| std_max(a, b))
        .collect();
    let upper: Vec<f64> = max
        .iter()
        .zip(&pole_max)
        .map(|(&a, &b)| std_min(a, b))
        .collect();
    reduced.update_min_max(&lower, &upper);
}

/// `ReduceSplineBox(Poles, OrigBox, ReducedBox)`: update `reduced` with the part of
/// `original` inside the hull box of every pole;
/// `reduce_spline_box(&poles, &sampled, &mut reduced)`.
pub fn reduce_spline_box<B: SampleBounds>(
    poles: &Array1<'_, B::Point>,
    original: &B,
    reduced: &mut B,
) {
    let mut hull: B = B::default();
    poles.iter().for_each(|&pole| hull.add_sample(pole));
    intersect_with_hull(hull, original, reduced);
}

/// `ComputePoleIndexRange` (and the identical `ComputePolesIndexes` of
/// GeomBndLib_BSplineSurface.cxx): the 1-based pole range covering `[min, max]` of a
/// B-spline; on a periodic curve the upper index may exceed `max_pole`;
/// `compute_pole_index_range(&knots, &mults, 3, 0.2, 0.4, 7, false)?`.
pub fn compute_pole_index_range(
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    degree: i32,
    (min, max): (f64, f64),
    max_pole: i32,
    periodic: bool,
) -> Result<(i32, i32), Failure> {
    let mut min_index: i32 = knots.lower();
    bsplclib::hunt(knots, min, &mut min_index)?;
    min_index = min_index.clamp(knots.lower(), knots.upper());
    let mut max_index: i32 = knots.lower();
    bsplclib::hunt(knots, max, &mut max_index)?;
    max_index = (max_index + 1).clamp(knots.lower(), knots.upper());
    let multiplier: i32 = mults[max_index];
    min_index = 1.max(bsplclib::pole_index(degree, min_index, periodic, mults)? + 1);
    max_index = bsplclib::pole_index(degree, max_index, periodic, mults)? + 1;
    max_index += degree - multiplier;
    if !periodic {
        max_index = max_index.min(max_pole);
    }
    Ok((min_index, max_index))
}

// The per-coordinate state of the sampled pass of CurveBoxOptimal.
pub(super) struct CoordinateSweep {
    pub(super) min: Vec<f64>,
    pub(super) max: Vec<f64>,
    pub(super) deflection: Vec<f64>,
}

impl CoordinateSweep {
    pub(super) fn new(dim: usize) -> Self {
        Self {
            min: vec![real_last(); dim],
            max: vec![-real_last(); dim],
            deflection: vec![-real_last(); dim],
        }
    }
    pub(super) fn widen<P: SamplePoint>(&mut self, point: P) {
        for k in 0..P::DIM {
            let value: f64 = point.coordinate(k);
            if self.min[k] > value {
                self.min[k] = value;
            }
            if self.max[k] < value {
                self.max[k] = value;
            }
        }
    }
    // The middle sample `mid` against the chord midpoint of `a` and `b`.
    pub(super) fn deflect<P: SamplePoint>(&mut self, a: P, b: P, mid: P) {
        self.widen(mid);
        for k in 0..P::DIM {
            let chord: f64 = 0.5 * (a.coordinate(k) + b.coordinate(k));
            let difference: f64 = (mid.coordinate(k) - chord).abs();
            if self.deflection[k] < difference {
                self.deflection[k] = difference;
            }
        }
    }
}

/// The sampled grid and extremum refinement shared by `CurveBoxOptimal` and the
/// `BoxOptimal` of GeomBndLib_OtherCurve(2d): `count` samples of `[u1, u2]` with their
/// midpoints, then `adjust(range, extr0, index, eps, is_min)` near each coordinate
/// extremum whose deflection exceeds `eps = max(tol, Precision::Confusion())`.
pub(super) struct OptimalCurveBox<'a, C: ?Sized + SampledCurve> {
    pub curve: &'a C,
    pub u1: f64,
    pub u2: f64,
    pub count: i32,
}

/// The point type a sampled curve adds to its box.
type SampleOf<C> = <<C as SampledCurve>::Bounds as SampleBounds>::Point;

type Adjust<'f> = dyn Fn((f64, f64), f64, i32, f64, bool) -> Result<f64, Failure> + 'f;

impl<C: ?Sized + SampledCurve> OptimalCurveBox<'_, C> {
    fn step(&self) -> f64 {
        (self.u2 - self.u1) / f64::from(self.count - 1)
    }
    fn sweep(&self) -> Result<(Vec<SampleOf<C>>, CoordinateSweep), Failure> {
        let step: f64 = self.step();
        let half: f64 = step / 2.0;
        let mut sweep =
            CoordinateSweep::new(<<C::Bounds as SampleBounds>::Point as SamplePoint>::DIM);
        let mut points = Vec::with_capacity(self.count.max(0) as usize);
        for j in 0..self.count {
            let u: f64 = self.u1 + f64::from(j) * step;
            let point = self.curve.sample_point(u)?;
            sweep.widen(point);
            if let Some(&previous) = points.last() {
                let mid = self.curve.sample_point(u - half)?;
                sweep.deflect(previous, point, mid);
            }
            points.push(point);
        }
        Ok((points, sweep))
    }
    // The refinement range of sample `i` (1-based): `[u1 + max(0, i-2) du, u1 + min(N-1, i) du]`.
    fn range(&self, i: i32) -> (f64, f64) {
        let step: f64 = self.step();
        let lower: f64 = self.u1 + f64::from(0.max(i - 2)) * step;
        (lower, self.u1 + f64::from((self.count - 1).min(i)) * step)
    }
    /// Compute the box, refining with `adjust`; `optimal.compute(1e-7, &adjust)?`.
    pub fn compute(&self, tolerance: f64, adjust: &Adjust<'_>) -> Result<C::Bounds, Failure> {
        let (points, mut sweep) = self.sweep()?;
        let eps: f64 = std_max(tolerance, precision::confusion());
        for k in 0..<<C::Bounds as SampleBounds>::Point as SamplePoint>::DIM {
            if sweep.deflection[k] <= eps {
                continue;
            }
            let (min, max) = self.refine(&points, &sweep, k, eps, adjust)?;
            sweep.min[k] = min;
            sweep.max[k] = max;
        }
        let mut bounds: C::Bounds = C::Bounds::default();
        bounds.add_sample(SamplePoint::from_coordinates(&sweep.min));
        bounds.add_sample(SamplePoint::from_coordinates(&sweep.max));
        bounds.enlarge_by(eps);
        Ok(bounds)
    }
    fn refine(
        &self,
        points: &[<C::Bounds as SampleBounds>::Point],
        sweep: &CoordinateSweep,
        k: usize,
        eps: f64,
        adjust: &Adjust<'_>,
    ) -> Result<(f64, f64), Failure> {
        let difference: f64 = sweep.deflection[k];
        let (mut c_min, mut c_max) = (sweep.min[k], sweep.max[k]);
        let index: i32 = k as i32 + 1;
        for (i, point) in (1..).zip(points) {
            let value: f64 = point.coordinate(k);
            if value - c_min < difference {
                let local: f64 = adjust(self.range(i), c_min, index, eps, true)?;
                if local < c_min {
                    c_min = local;
                }
            } else if c_max - value < difference {
                let local: f64 = adjust(self.range(i), c_max, index, eps, false)?;
                if local > c_max {
                    c_max = local;
                }
            }
        }
        Ok((c_min, c_max))
    }
}

/// `CurveBoxOptimal`: the precise box of `[u1, u2]` from `ComputeNbSamplesT` samples
/// refined by `AdjustExtrT` (MathOpt PSO + Brent);
/// `curve_box_optimal(adaptor, 0.0, 1.0, 1e-7)?`.
pub fn curve_box_optimal<C: ?Sized + SampledCurve>(
    curve: &C,
    u1: f64,
    u2: f64,
    tolerance: f64,
) -> Result<C::Bounds, Failure> {
    let count: i32 = curve.sample_count(u1, u2)?;
    let optimal = OptimalCurveBox {
        curve,
        u1,
        u2,
        count,
    };
    optimal.compute(tolerance, &|range, extr0, index, eps, is_min| {
        curve.adjust_extremum(range, extr0, index, eps, is_min)
    })
}
