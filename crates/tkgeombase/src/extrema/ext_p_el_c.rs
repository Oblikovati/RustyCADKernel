// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of Extrema_ExtPElC.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtPElC`: the extrema of the distance between a 3D point and a segment
//! of an elementary curve, in closed form.
use super::curve_tools::CurveTool;
use super::elementary_point_curve::{ElementaryPointCurve, PointCurveExtrema};
use super::pon::POnCurv;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    self, Circle3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3, Point3, Vector3,
};
use rustycad_tkmath::math::{DirectPolynomialRoots, TrigonometricRoots};
use std::f64::consts::PI;
use std::ops::Deref;

/// `Extrema_ExtPElC`. The result accessors (`IsDone`, `NbExt`, `SquareDistance`,
/// `IsMin`, `Point`) are those of `PointCurveExtrema`, reached through `Deref`;
/// `ExtPElC::with_line(point, line, 1e-7, -1.0, 1.0).nb_ext()? == 1`.
#[derive(Clone, Debug)]
pub struct ExtPElC {
    result: PointCurveExtrema<Point3>,
}

impl Default for ExtPElC {
    fn default() -> Self {
        Self {
            result: PointCurveExtrema::new("Extrema_ExtPElC"),
        }
    }
}

impl Deref for ExtPElC {
    type Target = PointCurveExtrema<Point3>;
    fn deref(&self) -> &Self::Target {
        &self.result
    }
}

// The projection `Pp` of `point` in the plane of a conic of centre `origin` and
// normal `axis` (`P.Translated(Axe.Multiplied(-(gp_Vec(O, P).Dot(Axe))))`).
fn plane_projection(point: Point3, origin: Point3, axis: Vector3) -> Point3 {
    let translation: Vector3 = axis * -(Vector3::between(origin, point).dot(axis));
    point.translated(translation)
}

impl ExtPElC {
    /// `Extrema_ExtPElC()`: not done; `ExtPElC::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtPElC(P, gp_Lin, Tol, Uinf, Usup)`;
    /// `ExtPElC::with_line(point, line, 1e-7, -1.0, 1.0)`.
    pub fn with_line(point: Point3, line: Line3, tol: f64, uinf: f64, usup: f64) -> Self {
        let mut extrema: Self = Self::new();
        extrema.perform_line(point, &line, tol, uinf, usup);
        extrema
    }
    /// `Perform(P, gp_Lin, Tol, Uinf, Usup)`: the projection when its parameter lies
    /// in `[Uinf - Tol, Usup + Tol]`, otherwise not done;
    /// `extrema.perform_line(point, &line, 1e-7, -1.0, 1.0)`.
    pub fn perform_line(&mut self, point: Point3, line: &Line3, tol: f64, uinf: f64, usup: f64) {
        self.result.done = false;
        self.result.nb_ext = 0;
        let v1: Vector3 = Vector3::from(line.direction());
        let origin: Point3 = line.location();
        let distance: f64 = v1.dot(Vector3::between(origin, point));
        if (distance >= uinf - tol) && (distance <= usup + tol) {
            let projection: Point3 = origin.translated(v1 * distance);
            let sq_dist: f64 = point.square_distance(&projection);
            self.result
                .push(sq_dist, true, POnCurv::new(distance, projection));
            self.result.done = true;
        }
    }
    /// `Extrema_ExtPElC(P, gp_Circ, Tol, Uinf, Usup)`;
    /// `ExtPElC::with_circle(point, circle, 1e-7, 0.0, 2.0 * PI)?`.
    pub fn with_circle(
        point: Point3,
        circle: Circle3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_circle(point, &circle, tol, uinf, usup)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Circ, Tol, Uinf, Usup)`: the nearest (minimum) and farthest
    /// points in `[Uinf, Usup]`; not done when the point is on the axis. The
    /// directions built from the projection raise Standard_ConstructionError when it
    /// is within gp::Resolution() of the centre (a tolerance below it);
    /// `extrema.perform_circle(point, &circle, 1e-7, 0.0, 2.0 * PI)?`.
    pub fn perform_circle(
        &mut self,
        point: Point3,
        circle: &Circle3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.result.nb_ext = 0;
        let origin: Point3 = circle.location();
        let axe: Vector3 = Vector3::from(circle.axis().direction());
        let opp: Vector3 = Vector3::between(origin, plane_projection(point, origin, axe));
        if opp.magnitude() < tol {
            return Ok(());
        }
        let x_dir: Direction3 = circle.x_axis().direction();
        let angle: f64 =
            x_dir.angle_with_ref(Direction3::from_vector(opp)?, Direction3::from_vector(axe)?);
        let tol_u: f64 = circle_parameter_tolerance(tol, circle.radius());
        let solutions: [f64; 2] = circle_solutions(angle, uinf, tol_u);
        for (index, us) in solutions.into_iter().enumerate() {
            if ((uinf - us) < tol_u) && ((us - usup) < tol_u) {
                let cu: Point3 = elclib::value(us, circle);
                let sq_dist: f64 = cu.square_distance(&point);
                self.result.push(sq_dist, index == 0, POnCurv::new(us, cu));
            }
        }
        self.result.done = true;
        Ok(())
    }
}

// TolU = Tol / R, or Precision::Infinite() for a radius below gp::Resolution().
fn circle_parameter_tolerance(tol: f64, radius: f64) -> f64 {
    if radius > gp::resolution() {
        return tol / radius;
    }
    precision::infinite()
}

// The minimum parameter (the angle of the projection, snapped to -PI or PI within
// Precision::Angular()) and the maximum one half a turn further, both brought into
// the period starting at `uinf` and moved to `uinf` when one period away from it.
fn circle_solutions(angle: f64, uinf: f64, tol_u: f64) -> [f64; 2] {
    let ang_tol: f64 = precision::angular();
    let mut first: f64 = angle;
    if first + PI < ang_tol {
        first = -PI;
    } else if first - PI > -ang_tol {
        first = PI;
    }
    let mut solutions: [f64; 2] = adjust_pair(uinf, tol_u, [first, first + PI]);
    for solution in &mut solutions {
        let shift: f64 = *solution - 2.0 * PI - uinf;
        if shift < tol_u && shift > -tol_u {
            *solution = uinf;
        }
    }
    solutions
}

/// The two `ElCLib::AdjustPeriodic(Uinf, Uinf + 2 PI, Tol, myuinf, U)` calls of the
/// circle cases: the first call also moves `myuinf`, which the second one starts from.
pub(super) fn adjust_pair(uinf: f64, tol: f64, [first, second]: [f64; 2]) -> [f64; 2] {
    let (moved_uinf, first) = elclib::adjust_periodic(uinf, uinf + 2.0 * PI, tol, uinf, first);
    let second: f64 = elclib::adjust_periodic(uinf, uinf + 2.0 * PI, tol, moved_uinf, second).1;
    [first, second]
}

impl ExtPElC {
    /// `Extrema_ExtPElC(P, gp_Elips, Tol, Uinf, Usup)`;
    /// `ExtPElC::with_ellipse(point, ellipse, 1e-7, 0.0, 2.0 * PI)`.
    pub fn with_ellipse(
        point: Point3,
        ellipse: Ellipse3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_ellipse(point, &ellipse, tol, uinf, usup)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Elips, Tol, Uinf, Usup)`: the roots in `[Uinf, Usup]` of
    /// `(B² - A²)/2 sin 2u - B Y cos u + A X sin u` (math_TrigonometricFunctionRoots);
    /// not done at the centre of a circular ellipse or when the solver fails;
    /// `extrema.perform_ellipse(point, &ellipse, 1e-7, 0.0, 2.0 * PI)`.
    pub fn perform_ellipse(
        &mut self,
        point: Point3,
        ellipse: &Ellipse3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.result.nb_ext = 0;
        let origin: Point3 = ellipse.location();
        let axe: Vector3 = Vector3::from(ellipse.axis().direction());
        let opp: Vector3 = Vector3::between(origin, plane_projection(point, origin, axe));
        let (a, b) = (ellipse.major_radius(), ellipse.minor_radius());
        if opp.magnitude() < tol && (a - b).abs() < tol {
            return Ok(());
        }
        let x: f64 = opp.dot(Vector3::from(ellipse.x_axis().direction()));
        let y: f64 = opp.dot(Vector3::from(ellipse.y_axis().direction()));
        let (ko2, mut ko3, ko4) = ((b * b - a * a) / 2., -b * y, a * x);
        if ko3.abs() < 1.0e-16 * std_max(ko2.abs(), ko3.abs()) {
            ko3 = 0.0;
        }
        let roots = TrigonometricRoots::new(0., ko2, ko3, ko4, 0., uinf, usup);
        if !roots.is_done() {
            return Ok(());
        }
        // NbSolutions() raises StdFail_InfiniteSolutions for an identity equation.
        for index in 1..=roots.solutions()? {
            let us: f64 = roots.value(index)?;
            let cu: Point3 = elclib::value(us, ellipse);
            let sq_dist: f64 = cu.square_distance(&point);
            let next: Point3 = elclib::value(us + 0.1, ellipse);
            let is_min: bool = sq_dist < next.square_distance(&point);
            self.result.push(sq_dist, is_min, POnCurv::new(us, cu));
        }
        self.result.done = true;
        Ok(())
    }
}

impl ExtPElC {
    /// `Extrema_ExtPElC(P, gp_Hypr, Tol, Uinf, Usup)`;
    /// `ExtPElC::with_hyperbola(point, hyperbola, 1e-7, -2.0, 2.0)`.
    pub fn with_hyperbola(
        point: Point3,
        hyperbola: Hyperbola3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_hyperbola(point, &hyperbola, tol, uinf, usup)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Hypr, Tol, Uinf, Usup)`: the logarithms of the positive roots `v`
    /// of `(R²+r²)/4 (v⁴ - 1) - (XR+Yr)/2 v³ + (XR-Yr)/2 v` in `[Uinf, Usup]`; two
    /// solutions closer than Tol count once;
    /// `extrema.perform_hyperbola(point, &hyperbola, 1e-7, -2.0, 2.0)`.
    pub fn perform_hyperbola(
        &mut self,
        point: Point3,
        hyperbola: &Hyperbola3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.result.nb_ext = 0;
        let origin: Point3 = hyperbola.location();
        let axe: Vector3 = Vector3::from(hyperbola.axis().direction());
        let opp: Vector3 = Vector3::between(origin, plane_projection(point, origin, axe));
        let (r_major, r_minor) = (hyperbola.major_radius(), hyperbola.minor_radius());
        let x: f64 = opp.dot(Vector3::from(hyperbola.x_axis().direction()));
        let y: f64 = opp.dot(Vector3::from(hyperbola.y_axis().direction()));
        let c1: f64 = (r_major * r_major + r_minor * r_minor) / 4.;
        let roots = DirectPolynomialRoots::quartic(
            c1,
            -(x * r_major + y * r_minor) / 2.,
            0.,
            (x * r_major - y * r_minor) / 2.,
            -c1,
        );
        if !roots.is_done() {
            return Ok(());
        }
        let parameters: Vec<f64> = polynomial_roots(&roots)?
            .into_iter()
            .filter(|&v| v > 0.)
            .map(f64::ln)
            .collect();
        self.push_distinct(point, hyperbola, &parameters, (uinf, usup), tol * tol);
        self.result.done = true;
        Ok(())
    }
    // The 3D minimum test compares with the point at `u + 1`.
    fn push_distinct<C>(
        &mut self,
        point: Point3,
        curve: &C,
        parameters: &[f64],
        bounds: (f64, f64),
        sq_tol: f64,
    ) where
        C: elclib::ElementaryCurve<Point = Point3>,
    {
        let value = |u: f64| elclib::value(u, curve);
        let is_min = |u: f64, sq_dist: f64| sq_dist < point.square_distance(&value(u + 1.));
        self.result
            .push_distinct(point, parameters, bounds, sq_tol, value, is_min);
    }
}

/// The roots of a done math_DirectPolynomialRoots, in the solver order;
/// StdFail_InfiniteSolutions for an identity polynomial, as NbSolutions() raises.
pub(super) fn polynomial_roots(roots: &DirectPolynomialRoots) -> Result<Vec<f64>, Failure> {
    (1..=roots.solutions()?).map(|i| roots.value(i)).collect()
}

impl ExtPElC {
    /// `Extrema_ExtPElC(P, gp_Parab, Tol, Uinf, Usup)`; the tolerance is unused;
    /// `ExtPElC::with_parabola(point, parabola, 1e-7, -2.0, 2.0)`.
    pub fn with_parabola(
        point: Point3,
        parabola: Parabola3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.perform_parabola(point, &parabola, tol, uinf, usup)?;
        Ok(extrema)
    }
    /// `Perform(P, gp_Parab, Tol, Uinf, Usup)`: the roots in `[Uinf, Usup]` of
    /// `u³/(4F) + (2F - X) u - 2FY`; two solutions closer than
    /// Precision::Confusion() count once;
    /// `extrema.perform_parabola(point, &parabola, 1e-7, -2.0, 2.0)`.
    pub fn perform_parabola(
        &mut self,
        point: Point3,
        parabola: &Parabola3,
        _tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        self.result.done = false;
        self.result.nb_ext = 0;
        let origin: Point3 = parabola.location();
        let axe: Vector3 = Vector3::from(parabola.axis().direction());
        let opp: Vector3 = Vector3::between(origin, plane_projection(point, origin, axe));
        let focal: f64 = parabola.focal();
        let x: f64 = opp.dot(Vector3::from(parabola.x_axis().direction()));
        let y: f64 = opp.dot(Vector3::from(parabola.y_axis().direction()));
        let roots =
            DirectPolynomialRoots::cubic(1. / (4. * focal), 0., 2. * focal - x, -2. * focal * y);
        if !roots.is_done() {
            return Ok(());
        }
        let parameters: Vec<f64> = polynomial_roots(&roots)?;
        let sq_tol: f64 = precision::square_confusion();
        self.push_distinct(point, parabola, &parameters, (uinf, usup), sq_tol);
        self.result.done = true;
        Ok(())
    }
}

impl ElementaryPointCurve<CurveTool> for ExtPElC {
    fn perform_line(
        &mut self,
        point: Point3,
        line: &Line3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC::perform_line(self, point, line, tol, uinf, usup);
        Ok(())
    }
    fn perform_circle(
        &mut self,
        point: Point3,
        circle: &Circle3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC::perform_circle(self, point, circle, tol, uinf, usup)
    }
    fn perform_ellipse(
        &mut self,
        point: Point3,
        ellipse: &Ellipse3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC::perform_ellipse(self, point, ellipse, tol, uinf, usup)
    }
    fn perform_hyperbola(
        &mut self,
        point: Point3,
        hyperbola: &Hyperbola3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC::perform_hyperbola(self, point, hyperbola, tol, uinf, usup)
    }
    fn perform_parabola(
        &mut self,
        point: Point3,
        parabola: &Parabola3,
        tol: f64,
        uinf: f64,
        usup: f64,
    ) -> Result<(), Failure> {
        ExtPElC::perform_parabola(self, point, parabola, tol, uinf, usup)
    }
    fn point_extrema(&self) -> &PointCurveExtrema<Point3> {
        &self.result
    }
}
