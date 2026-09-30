// Rust translation of the file-local helpers of IntAna_IntQuadQuad.cxx (AddSpecialPoints,
// TrigonometricRoots, MyTrigonometricFunction) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trigonometric helpers of IntAna_IntQuadQuad: sorted, verified roots, the
//! value of a trigonometric polynomial and the special-point widening of a domain.
use crate::int_ana::Quadric;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Cylinder3, Point3};
use rustycad_tkmath::math::TrigonometricRoots;
use std::f64::consts::PI;

/// The native file-local `TrigonometricRoots`: the roots in `[0, 2 PI]` of
/// `CC cos² + 2 SC cos sin + C cos + S sin + Cte`, sorted, and rejected (not done)
/// when one of them leaves a residual above 1e-8.
pub(super) struct SortedTrigRoots {
    roots: [f64; 4],
    done: bool,
    nb_roots: i32,
    infinite_roots: bool,
}

impl SortedTrigRoots {
    /// Solves with math_TrigonometricFunctionRoots on `[binf, bsup]`, brings the
    /// roots into `[0, 2 PI]`, checks their residual and sorts them; a polynomial
    /// below 1e-10 without roots is an identity.
    pub fn new(coefficients: [f64; 5], binf: f64, bsup: f64) -> Result<Self, Failure> {
        let [cc, sc, c, s, cte] = coefficients;
        let mut result = Self {
            roots: [0.0; 4],
            done: false,
            nb_roots: 0,
            infinite_roots: false,
        };
        let solver = TrigonometricRoots::new(cc, sc, c, s, cte, binf, bsup);
        if !solver.is_done() {
            return Ok(result);
        }
        result.done = true;
        if solver.infinite_roots() {
            result.infinite_roots = true;
            return Ok(result);
        }
        result.nb_roots = solver.solutions()?;
        for i in 0..result.nb_roots as usize {
            result.roots[i] = periodic_root(solver.value(i as i32 + 1)?);
        }
        // The direct search gives unreliable results.
        if result.roots[..result.nb_roots as usize]
            .iter()
            .any(|&root| trig_residual(coefficients, root).abs() > 1e-8)
        {
            result.done = false;
            return Ok(result);
        }
        result.sort();
        let small: bool = (cc.abs() + sc.abs() + c.abs() + s.abs()) < 1e-10;
        result.infinite_roots = result.nb_roots == 0 && small && cte.abs() < 1e-10;
        Ok(result)
    }

    /// The native bubble sort of adjacent roots.
    fn sort(&mut self) {
        let count: usize = self.nb_roots as usize;
        let mut sorted: bool = false;
        while !sorted {
            sorted = true;
            for i in 1..count {
                if self.roots[i] < self.roots[i - 1] {
                    self.roots.swap(i, i - 1);
                    sorted = false;
                }
            }
        }
    }

    /// IsDone(). `roots.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// IsARoot(u): `u` is within RealEpsilon() of a root or of a root plus 2 PI.
    pub fn is_a_root(&self, u: f64) -> bool {
        self.roots[..self.nb_roots as usize].iter().any(|&root| {
            (u - root).abs() <= f64::EPSILON || (u - root - (PI + PI)).abs() <= f64::EPSILON
        })
    }

    /// NbSolutions(): StdFail_NotDone when not done.
    pub fn nb_solutions(&self) -> Result<i32, Failure> {
        self.check_done()?;
        Ok(self.nb_roots)
    }

    /// InfiniteRoots(): StdFail_NotDone when not done.
    pub fn infinite_roots(&self) -> Result<bool, Failure> {
        self.check_done()?;
        Ok(self.infinite_roots)
    }

    /// Value(n): StdFail_NotDone when not done or `n > NbSolutions()`.
    pub fn value(&self, n: i32) -> Result<f64, Failure> {
        if !self.done || n > self.nb_roots || n < 1 {
            return Err(Failure::with_type(
                "StdFail_NotDone",
                &format!(
                    "TrigonometricRoots::Value() - root {n} of {} (done {})",
                    self.nb_roots, self.done
                ),
                "",
            ));
        }
        Ok(self.roots[(n - 1) as usize])
    }

    fn check_done(&self) -> Result<(), Failure> {
        if self.done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            "TrigonometricRoots - the roots are not done",
            "",
        ))
    }
}

/// A root brought into `[0, 2 PI]` by one period.
fn periodic_root(root: f64) -> f64 {
    let mut root: f64 = root;
    if root < 0. {
        root += PI + PI;
    }
    if root > PI + PI {
        root -= PI + PI;
    }
    root
}

/// `cos (CC cos + 2 SC sin + C) + S sin + Cte` at `root`.
fn trig_residual([cc, sc, c, s, cte]: [f64; 5], root: f64) -> f64 {
    let (co, si) = (root.cos(), root.sin());
    co * (cc * co + (sc + sc) * si + c) + s * si + cte
}

/// The native MyTrigonometricFunction:
/// `CC cos² + SS sin² + 2 (sin (SC cos + S) + cos C) + Cte`.
pub(super) struct TrigFunction {
    pub cc: f64,
    pub ss: f64,
    pub sc: f64,
    pub c: f64,
    pub s: f64,
    pub cte: f64,
}

impl TrigFunction {
    /// The value at `u`.
    pub fn value(&self, u: f64) -> f64 {
        let (sinus, cosinus) = (u.sin(), u.cos());
        self.cc * cosinus * cosinus
            + self.ss * sinus * sinus
            + 2.0 * (sinus * (self.sc * cosinus + self.s) + cosinus * self.c)
            + self.cte
    }
}

/// AddSpecialPoints: widens `[theta1, theta2]` symmetrically so that every
/// special point of the quadric lying on the cylinder (within
/// Precision::SquareConfusion()) falls in the domain; the width stays at most 2 PI.
pub(super) fn add_special_points(
    quadric: &Quadric,
    cylinder: &Cylinder3,
    theta1: &mut f64,
    theta2: &mut f64,
) {
    let points = quadric.special_points();
    if points.is_empty() {
        return;
    }
    let mut max_delta: f64 = 0.0;
    for point in points.iter() {
        if let Some(delta) = special_point_delta(cylinder, *point, *theta1, *theta2) {
            max_delta = std_max(max_delta, delta);
        }
    }
    if max_delta != 0.0 {
        *theta1 -= max_delta;
        *theta2 += max_delta;
        if (*theta2 - *theta1) > PI + PI {
            *theta2 = *theta1 + (PI + PI);
        }
    }
}

/// How far the angle of `point` lies outside `[theta1, theta2]` (a gap above PI
/// counts as none), or `None` when the point is not on the cylinder.
fn special_point_delta(
    cylinder: &Cylinder3,
    point: Point3,
    theta1: f64,
    theta2: f64,
) -> Option<f64> {
    let (u, v) = elslib::parameters(cylinder, point);
    let projection: Point3 = elslib::value(u, v, cylinder);
    if point.square_distance(&projection) > precision::square_confusion() {
        return None;
    }
    let mut delta1: f64 = std_min(u - theta1, 0.0);
    let mut delta2: f64 = std_max(u - theta2, 0.0);
    if delta1 < -PI {
        // Must be min(u - theta1 + period, 0), but u - theta1 + period >= 0 always.
        delta1 = 0.0;
    }
    if delta2 > PI {
        // Must be max(u - theta2 - period, 0), but u - theta2 - period <= 0 always.
        delta2 = 0.0;
    }
    Some(std_max(-delta1, delta2))
}
