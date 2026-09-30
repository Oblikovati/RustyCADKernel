// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_CurveToAnaCurve.cxx
// (ComputeCurve) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_CurveToAnaCurve::ComputeCurve`: the choice among the line, circle
//! and ellipse candidates.
use super::ConvType;
use super::curve_to_ana_curve::{AnaCurveFit, CurveToAnaCurve};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point3;

// RealLast(): the initial deviation of every candidate.
const REAL_LAST: f64 = f64::MAX;

/// The candidate curves (line, circle, ellipse) with their ranges and deviations.
struct Candidates {
    curves: [Handle<dyn GeomCurve>; 3],
    fits: [AnaCurveFit; 3],
}

impl Candidates {
    // Null curves, zero ranges and RealLast deviations.
    fn new() -> Self {
        let fit = AnaCurveFit {
            first: 0.0,
            last: 0.0,
            deviation: REAL_LAST,
        };
        Self {
            curves: [Handle::default(), Handle::default(), Handle::default()],
            fits: [fit; 3],
        }
    }

    // Recognizes candidate `index` (0 line, 1 circle, 2 ellipse).
    fn compute(
        &mut self,
        index: usize,
        curve: &Handle<dyn GeomCurve>,
        tolerance: f64,
        range: (f64, f64),
    ) -> Result<(), Failure> {
        let fit: &mut AnaCurveFit = &mut self.fits[index];
        self.curves[index] = match index {
            0 => CurveToAnaCurve::compute_line(curve, tolerance, range, fit)?.cast(),
            1 => CurveToAnaCurve::compute_circle(curve, tolerance, range, fit)?,
            _ => CurveToAnaCurve::compute_ellipse(curve, tolerance, range, fit)?,
        };
        Ok(())
    }
}

impl CurveToAnaCurve {
    /// `ComputeCurve(curve, tolerance, c1, c2, cf, cl, theGap, theCurvType =
    /// MinGap, theTarget = Line)`: the analytic curve of `curve` on `[c1, c2]`,
    /// null when none is within `tolerance`. Target tries only `target` (a target
    /// other than a line, circle or ellipse falls through to MinGap); Simplest
    /// returns the first of line, circle, ellipse found; MinGap the one of smallest
    /// deviation (only the line and circle for a curve shorter than 2*tolerance).
    /// `fit.deviation` is theGap: -1 when MinGap finds nothing;
    /// `CurveToAnaCurve::compute_curve(&c, 1e-7, (0.0, 1.0), &mut fit, ConvType::MinGap, CurveType::Line)?`.
    pub fn compute_curve(
        curve: &Handle<dyn GeomCurve>,
        tolerance: f64,
        (c1, c2): (f64, f64),
        fit: &mut AnaCurveFit,
        conv_type: ConvType,
        target: CurveType,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        (fit.first, fit.last) = (c1, c2);
        let object: &dyn GeomCurve = match curve.get() {
            Some(object) => object,
            None => return Ok(Handle::default()),
        };
        let p1: Point3 = object.value(c1)?;
        let p2: Point3 = object.value(c2)?;
        let p3: Point3 = object.value(c1 + (c2 - c1) / 2.0)?;
        let c3d: Handle<dyn GeomCurve> = match curve.cast::<GeomTrimmedCurve>().get() {
            Some(trimmed) => trimmed.basis_curve(),
            None => curve.clone(),
        };
        if conv_type == ConvType::Target {
            fit.deviation = REAL_LAST;
            if let Some(index) = target_index(target) {
                return single_candidate(index, &c3d, tolerance, (c1, c2), fit);
            }
        }
        if conv_type == ConvType::Simplest {
            return simplest(&c3d, tolerance, (c1, c2), fit);
        }
        min_gap(&c3d, tolerance, (c1, c2), (p1, p2, p3), fit)
    }
}

// The candidate index of a Target curve type (line, circle, ellipse).
fn target_index(target: CurveType) -> Option<usize> {
    match target {
        CurveType::Line => Some(0),
        CurveType::Circle => Some(1),
        CurveType::Ellipse => Some(2),
        _ => None,
    }
}

// One recognizer, its range copied to `fit` and its deviation written to theGap.
fn single_candidate(
    index: usize,
    c3d: &Handle<dyn GeomCurve>,
    tolerance: f64,
    range: (f64, f64),
    fit: &mut AnaCurveFit,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut candidates = Candidates::new();
    candidates.fits[index].deviation = fit.deviation;
    let computed = candidates.compute(index, c3d, tolerance, range);
    let found: AnaCurveFit = candidates.fits[index];
    *fit = found;
    computed?;
    Ok(candidates.curves[index].clone())
}

// GeomConvert_Simplest: the line, else the circle, else the ellipse; the range of
// the last recognizer tried is kept only on success.
fn simplest(
    c3d: &Handle<dyn GeomCurve>,
    tolerance: f64,
    range: (f64, f64),
    fit: &mut AnaCurveFit,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut candidates = Candidates::new();
    for index in 0..3 {
        candidates.fits[index].deviation = REAL_LAST;
        let computed = candidates.compute(index, c3d, tolerance, range);
        fit.deviation = candidates.fits[index].deviation;
        computed?;
        if candidates.curves[index].is_some() {
            (fit.first, fit.last) = (candidates.fits[index].first, candidates.fits[index].last);
            return Ok(candidates.curves[index].clone());
        }
    }
    // Conversion failed, returns null curve.
    Ok(Handle::default())
}

// GeomConvert_MinGap: the candidate of smallest deviation; for a small curve
// (ends and middle within 2*tolerance) the circle unless the line deviates less.
fn min_gap(
    c3d: &Handle<dyn GeomCurve>,
    tolerance: f64,
    (c1, c2): (f64, f64),
    (p1, p2, p3): (Point3, Point3, Point3),
    fit: &mut AnaCurveFit,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut candidates = Candidates::new();
    let imin: Option<usize> =
        if p1.distance(&p2) < 2.0 * tolerance && p1.distance(&p3) < 2.0 * tolerance {
            small_curve_choice(&mut candidates, c3d, tolerance, (c1, c2))?
        } else {
            general_choice(&mut candidates, c3d, tolerance, (c1, c2))?
        };
    let Some(imin) = imin else {
        *fit = AnaCurveFit {
            first: c1,
            last: c2,
            deviation: -1.0,
        };
        return Ok(Handle::default()); // must be null curve
    };
    *fit = candidates.fits[imin];
    Ok(candidates.curves[imin].clone())
}

// The small-curve choice: the circle (1) unless it is null or the line deviates
// less, the line (0) otherwise, even when the line is null too.
fn small_curve_choice(
    candidates: &mut Candidates,
    c3d: &Handle<dyn GeomCurve>,
    tolerance: f64,
    range: (f64, f64),
) -> Result<Option<usize>, Failure> {
    candidates.compute(1, c3d, tolerance, range)?;
    candidates.compute(0, c3d, tolerance, range)?;
    if candidates.curves[1].is_null() || candidates.fits[0].deviation < candidates.fits[1].deviation
    {
        return Ok(Some(0));
    }
    Ok(Some(1))
}

// The general choice: the line, then (for a finite range) the circle and the
// ellipse within the smallest deviation found so far; the first non-null
// candidate of strictly smallest deviation.
fn general_choice(
    candidates: &mut Candidates,
    c3d: &Handle<dyn GeomCurve>,
    tolerance: f64,
    (c1, c2): (f64, f64),
) -> Result<Option<usize>, Failure> {
    candidates.compute(0, c3d, tolerance, (c1, c2))?;
    let mut tol: f64 = std_min(tolerance, candidates.fits[0].deviation);
    if !precision::is_infinite(c1) && !precision::is_infinite(c2) {
        candidates.compute(1, c3d, tol, (c1, c2))?;
        tol = std_min(tol, candidates.fits[1].deviation);
        candidates.compute(2, c3d, tol, (c1, c2))?;
    }
    let mut dd: f64 = REAL_LAST;
    let mut imin: Option<usize> = None;
    for index in 0..3 {
        if candidates.curves[index].is_null() {
            continue;
        }
        if candidates.fits[index].deviation < dd {
            dd = candidates.fits[index].deviation;
            imin = Some(index);
        }
    }
    Ok(imin)
}
