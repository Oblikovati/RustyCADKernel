// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of Convert_CompBezierCurvesToBSplineCurveBase.hxx, Convert_CompBezierCurvesToBSplineCurve.hxx/.cxx and Convert_CompBezierCurves2dToBSplineCurve2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! A chain of adjacent non-rational Bezier curves joined into one B-spline, C1
//! where the tangents at a junction are parallel.
use super::arrays::owned_copy;
use crate::bsplclib;
use crate::gp::{Point2, Point3, Vector2, Vector3};
use crate::math::real_epsilon;
use crate::plib::PolePoint;
use crate::plib::access::array_cell;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// The pole types the conversion is instantiated with (gp_Pnt, gp_Pnt2d), with the
/// vector operations of the junction test; `Point3::GUARDS_KNOT_RATIO`.
pub trait BezierJunctionPoint: PolePoint + Copy + Default {
    /// True for the 3D instantiation, which guards the knot ratio against Epsilon.
    const GUARDS_KNOT_RATIO: bool;
    /// `VecType(self, to).SquareMagnitude()`; `Point2::new(0.0, 0.0).chord_square(Point2::new(1.0, 0.0)) == 1.0`.
    fn chord_square(self, to: Self) -> f64;
    /// `VecType(first, second).IsParallel(VecType(second, third), angular)`;
    /// `Point2::chords_parallel(a, b, c, 1e-4)?`.
    fn chords_parallel(
        first: Self,
        second: Self,
        third: Self,
        angular: f64,
    ) -> Result<bool, Failure>;
}

impl BezierJunctionPoint for Point3 {
    const GUARDS_KNOT_RATIO: bool = true;
    fn chord_square(self, to: Self) -> f64 {
        Vector3::between(self, to).square_magnitude()
    }
    fn chords_parallel(
        first: Self,
        second: Self,
        third: Self,
        angular: f64,
    ) -> Result<bool, Failure> {
        Vector3::between(first, second).is_parallel(Vector3::between(second, third), angular)
    }
}

impl BezierJunctionPoint for Point2 {
    const GUARDS_KNOT_RATIO: bool = false;
    fn chord_square(self, to: Self) -> f64 {
        Vector2::between(self, to).square_magnitude()
    }
    fn chords_parallel(
        first: Self,
        second: Self,
        third: Self,
        angular: f64,
    ) -> Result<bool, Failure> {
        Vector2::between(first, second).is_parallel(Vector2::between(second, third), angular)
    }
}

/// Converts a sequence of adjacent Bezier curves into one B-spline
/// (`Convert_CompBezierCurvesToBSplineCurveBase<PointType, VecType>`);
/// `CompBezierCurvesToBSplineCurve::new(1.0e-4)`.
#[derive(Clone, Debug)]
pub struct CompBezierCurvesToBSplineCurveBase<P: BezierJunctionPoint> {
    sequence: Vec<Array1<'static, P>>,
    curve_poles: Vec<P>,
    curve_knots: Vec<f64>,
    knots_mults: Vec<i32>,
    degree: i32,
    angular: f64,
}

/// The 3D conversion (`Convert_CompBezierCurvesToBSplineCurve`);
/// `CompBezierCurvesToBSplineCurve::default()`.
pub type CompBezierCurvesToBSplineCurve = CompBezierCurvesToBSplineCurveBase<Point3>;

/// The 2D conversion (`Convert_CompBezierCurves2dToBSplineCurve2d`);
/// `CompBezierCurves2dToBSplineCurve2d::default()`.
pub type CompBezierCurves2dToBSplineCurve2d = CompBezierCurvesToBSplineCurveBase<Point2>;

impl<P: BezierJunctionPoint> Default for CompBezierCurvesToBSplineCurveBase<P> {
    /// The native default angular tolerance of 1.0e-4 radians.
    fn default() -> Self {
        Self::new(1.0e-4)
    }
}

impl<P: BezierJunctionPoint> CompBezierCurvesToBSplineCurveBase<P> {
    /// Empty conversion with the angular tolerance of the tangent test at junctions;
    /// `CompBezierCurves2dToBSplineCurve2d::new(1.0e-6)`.
    pub fn new(angular_tolerance: f64) -> Self {
        Self {
            sequence: Vec::new(),
            curve_poles: Vec::new(),
            curve_knots: Vec::new(),
            knots_mults: Vec::new(),
            degree: 0,
            angular: angular_tolerance,
        }
    }

    /// Appends a copy of the Bezier poles; `conversion.add_curve(&poles)`.
    pub fn add_curve(&mut self, poles: &Array1<'_, P>) {
        self.sequence.push(owned_copy(poles));
    }

    /// Computes poles, knots on `[0, 1]` and multiplicities; an empty sequence
    /// clears the results and keeps the previous degree; `conversion.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.curve_poles.clear();
        self.curve_knots.clear();
        self.knots_mults.clear();
        if self.sequence.is_empty() {
            return Ok(());
        }
        self.degree = self
            .sequence
            .iter()
            .fold(0, |d, c| d.max(c.len() as i32 - 1));
        let mut knot_values: Vec<f64> = vec![0.0; self.sequence.len()];
        let mut det: f64 = 0.0;
        let mut last_pole = P::default();
        let mut points: Array1<'static, P> = Array1::with_size(self.degree as usize + 1);
        points.update_lower_bound(1);
        for i in 0..self.sequence.len() {
            self.raise_into(i, &mut points)?;
            if i == 0 {
                self.open_first(&points, &mut knot_values, &mut det)?;
            } else {
                self.join(i, &points, last_pole, &mut knot_values, &mut det)?;
            }
            if i + 1 == self.sequence.len() {
                self.curve_poles.push(pole(&points, self.degree + 1)?);
                self.knots_mults.push(self.degree + 1);
            }
            last_pole = pole(&points, self.degree)?;
        }
        self.normalize_knots(&knot_values, det);
        Ok(())
    }

    // 1- Raise the Bezier curve to the maximum degree into the work array. A curve
    // already at that degree is assigned, which (NCollection_Array1::Assign) also
    // takes its bounds: the work array keeps them for the following curves, and
    // the one-based pole reads below shift or fail for a curve not based at 1.
    fn raise_into(&self, index: usize, points: &mut Array1<'static, P>) -> Result<(), Failure> {
        let curve: &Array1<'static, P> = &self.sequence[index];
        if self.degree - (curve.len() as i32 - 1) <= 0 {
            points.assign(curve);
            return Ok(());
        }
        bsplclib::increase_degree_bezier(self.degree, curve, None, points, None)
    }

    // Processing of the initial node of the BSpline.
    fn open_first(
        &mut self,
        points: &Array1<'_, P>,
        knot_values: &mut [f64],
        det: &mut f64,
    ) -> Result<(), Failure> {
        for j in 1..=self.degree {
            self.curve_poles.push(pole(points, j)?);
        }
        knot_values[0] = 1.; // To begin the series.
        self.knots_mults.push(self.degree + 1);
        *det = 1.;
        Ok(())
    }

    // Processing of the tangency between Bezier and the previous; this allows to
    // guarantee at least a C1 continuity if the tangents are coherent.
    fn join(
        &mut self,
        index: usize,
        points: &Array1<'_, P>,
        previous_pole: P,
        knot_values: &mut [f64],
        det: &mut f64,
    ) -> Result<(), Failure> {
        let (p2, p3) = (pole(points, 1)?, pole(points, 2)?);
        knot_values[index] = match self.tangent_ratio(previous_pole, p2, p3)? {
            Some(lambda)
                if !P::GUARDS_KNOT_RATIO || guarded(knot_values[index - 1], lambda, *det) =>
            {
                self.knots_mults.push(self.degree - 1);
                knot_values[index - 1] * lambda
            }
            _ => {
                self.curve_poles.push(p2);
                self.knots_mults.push(self.degree);
                1.0
            }
        };
        *det += knot_values[index];
        for j in 2..=self.degree {
            self.curve_poles.push(pole(points, j)?);
        }
        Ok(())
    }

    // sqrt(D2 / D1) when both chords are non-null and parallel at a degree above 1.
    fn tangent_ratio(&self, p1: P, p2: P, p3: P) -> Result<Option<f64>, Failure> {
        let (d1, d2) = (p1.chord_square(p2), p2.chord_square(p3));
        let resolution: f64 = f64::MIN_POSITIVE;
        if self.degree > 1
            && d1 > resolution
            && d2 > resolution
            && P::chords_parallel(p1, p2, p3, self.angular)?
        {
            return Ok(Some((d2 / d1).sqrt()));
        }
        Ok(None)
    }

    // Correct nodal values to make them variable within [0., 1.].
    fn normalize_knots(&mut self, knot_values: &[f64], det: f64) {
        self.curve_knots.push(0.0);
        for i in 1..knot_values.len() {
            let previous: f64 = self.curve_knots[i - 1];
            self.curve_knots.push(previous + (knot_values[i - 1] / det));
        }
        self.curve_knots.push(1.0);
    }

    /// Degree of the B-spline; `conversion.degree()`.
    pub fn degree(&self) -> i32 {
        self.degree
    }

    /// Number of poles; `conversion.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        self.curve_poles.len() as i32
    }

    /// Fills `poles` from its lower bound with the computed poles; an array longer
    /// than the result fails as the native sequence access does; `conversion.poles(&mut poles)?`.
    pub fn poles(&self, poles: &mut Array1<'_, P>) -> Result<(), Failure> {
        fill_from(poles, &self.curve_poles)
    }

    /// Number of knots; `conversion.nb_knots()`.
    pub fn nb_knots(&self) -> i32 {
        self.curve_knots.len() as i32
    }

    /// Fills the knots and multiplicities from their lower bounds;
    /// `conversion.knots_and_mults(&mut knots, &mut mults)?`.
    pub fn knots_and_mults(
        &self,
        knots: &mut Array1<'_, f64>,
        mults: &mut Array1<'_, i32>,
    ) -> Result<(), Failure> {
        fill_from(knots, &self.curve_knots)?;
        fill_from(mults, &self.knots_mults)
    }
}

// The 3D guard against knot values too small relative to the accumulated sum.
fn guarded(previous_knot: f64, lambda: f64, det: f64) -> bool {
    previous_knot * lambda > 10. * real_epsilon(det)
}

fn pole<P: Copy>(points: &Array1<'_, P>, index: i32) -> Result<P, Failure> {
    crate::plib::access::array_value(points, index)
}

fn fill_from<T: Copy>(target: &mut Array1<'_, T>, values: &[T]) -> Result<(), Failure> {
    for (k, i) in (target.lower()..=target.upper()).enumerate() {
        let value: T = *values.get(k).ok_or_else(|| {
            Failure::out_of_range(&format!(
                "NCollection_Sequence: index {} outside 1..={}",
                k + 1,
                values.len()
            ))
        })?;
        *array_cell(target, i)? = value;
    }
    Ok(())
}
