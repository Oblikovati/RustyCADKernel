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
// Rust translation of Extrema_ExtCC2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtCC2d`: the extrema between two 2D curves: Extrema_ExtElC2d for the
//! elementary pairs it solves, Extrema_ECC2d (over the whole curves) otherwise; the
//! extrema are filtered to the ranges within Precision::PConfusion().
use super::ECC2d;
use crate::extrema::ExtElC2d;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::curve_tools::Curve2dTool;
use crate::extrema::failures::{not_done, null_object, out_of_range, sequence_value};
use crate::extrema::pon::POnCurv2d;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Point2;
use std::f64::consts::PI;

/// `Extrema_ExtCC2d`: the second curve is set by `Initialize`, the first by
/// `Perform`; `ExtCC2d::with_curves(&c1, &c2, 1.0e-10, 1.0e-10)?.nb_ext()?`.
pub struct ExtCC2d<'a> {
    is_find_single_solution: bool,
    done: bool,
    is_par: bool,
    points: Vec<POnCurv2d>,
    sq_dist: Vec<f64>,
    nb_ext: i32,
    inverse: bool,
    curve: Option<&'a dyn Adaptor2dCurve2d>,
    v1: f64,
    v2: f64,
    tol_c1: f64,
    tol_c2: f64,
    p1f: Point2,
    p1l: Point2,
    p2f: Point2,
    p2l: Point2,
    dist: [f64; 4],
}

impl Default for ExtCC2d<'_> {
    fn default() -> Self {
        Self {
            is_find_single_solution: false,
            done: false,
            is_par: false,
            points: Vec::new(),
            sq_dist: Vec::new(),
            nb_ext: 0,
            inverse: false,
            curve: None,
            v1: 0.0,
            v2: 0.0,
            tol_c1: 0.0,
            tol_c2: 0.0,
            p1f: Point2::default(),
            p1l: Point2::default(),
            p2f: Point2::default(),
            p2l: Point2::default(),
            dist: [0.0; 4],
        }
    }
}

/// The period of a range check: 2 PI for a circle or ellipse, 0 (none) otherwise.
fn conic_period(kind: CurveType) -> f64 {
    if kind == CurveType::Circle || kind == CurveType::Ellipse {
        2. * PI
    } else {
        0.
    }
}

impl<'a> ExtCC2d<'a> {
    /// `Extrema_ExtCC2d()`; `ExtCC2d::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtCC2d(C1, C2, TolC1, TolC2)` (the native defaults are 1e-10): on the
    /// curve ranges; `ExtCC2d::with_curves(&c1, &c2, 1.0e-10, 1.0e-10)?`.
    pub fn with_curves(
        c1: &'a dyn Adaptor2dCurve2d,
        c2: &'a dyn Adaptor2dCurve2d,
        tol_c1: f64,
        tol_c2: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(
            c2,
            Curve2dTool::first_parameter(c2)?,
            Curve2dTool::last_parameter(c2)?,
            (tol_c1, tol_c2),
        );
        extrema.perform(
            c1,
            Curve2dTool::first_parameter(c1)?,
            Curve2dTool::last_parameter(c1)?,
        )?;
        Ok(extrema)
    }
    /// `Extrema_ExtCC2d(C1, C2, U1, U2, V1, V2, TolC1, TolC2)`;
    /// `ExtCC2d::with_ranges(&c1, &c2, (0.0, 1.0), (0.0, 1.0), 1.0e-10, 1.0e-10)?`.
    pub fn with_ranges(
        c1: &'a dyn Adaptor2dCurve2d,
        c2: &'a dyn Adaptor2dCurve2d,
        (u1, u2): (f64, f64),
        (v1, v2): (f64, f64),
        tol_c1: f64,
        tol_c2: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(c2, v1, v2, (tol_c1, tol_c2));
        extrema.perform(c1, u1, u2)?;
        Ok(extrema)
    }
    /// `Initialize(C2, V1, V2, TolC1, TolC2)`: the second curve and its range;
    /// `extrema.initialize(&c2, 0.0, 1.0, (1e-10, 1e-10))`.
    pub fn initialize(
        &mut self,
        c2: &'a dyn Adaptor2dCurve2d,
        v1: f64,
        v2: f64,
        (tol_c1, tol_c2): (f64, f64),
    ) {
        self.curve = Some(c2);
        (self.v1, self.v2) = (v1, v2);
        (self.tol_c1, self.tol_c2) = (tol_c1, tol_c2);
    }
    /// `SetSingleSolutionFlag(theFlag)`; `extrema.set_single_solution_flag(true)`.
    pub fn set_single_solution_flag(&mut self, flag: bool) {
        self.is_find_single_solution = flag;
    }
    /// `GetSingleSolutionFlag()`; `extrema.get_single_solution_flag()`.
    pub fn get_single_solution_flag(&self) -> bool {
        self.is_find_single_solution
    }
    /// `Perform(C1, U1, U2)`: the extrema between `C1` on `[U1, U2]` and the initialized
    /// curve; Standard_NullObject before `Initialize`; `extrema.perform(&c1, 0.0, 1.0)?`.
    pub fn perform(&mut self, c1: &dyn Adaptor2dCurve2d, u1: f64, u2: f64) -> Result<(), Failure> {
        let c2: &dyn Adaptor2dCurve2d = self
            .curve
            .ok_or_else(|| null_object("Extrema_ExtCC2d::Perform()"))?;
        self.points.clear();
        self.sq_dist.clear();
        let (type1, type2) = (Curve2dTool::get_type(c1)?, Curve2dTool::get_type(c2)?);
        let tol: f64 = std_min(self.tol_c1, self.tol_c2);
        self.nb_ext = 0;
        self.inverse = false;
        self.is_par = false;
        let ranges: [f64; 4] = [u1, u2, self.v1, self.v2];
        self.p1f = Curve2dTool::value(c1, u1)?;
        self.p1l = Curve2dTool::value(c1, u2)?;
        self.p2f = Curve2dTool::value(c2, self.v1)?;
        self.p2l = Curve2dTool::value(c2, self.v2)?;
        if let Some(xtrem) = self.elementary_extrema(c1, c2, (type1, type2), tol)? {
            let periods: [f64; 2] = [conic_period(type1), conic_period(type2)];
            return self.results_el(&xtrem, ranges, periods);
        }
        self.parametric_extrema(c1, c2, (type1, type2), ranges)
    }

    // The Extrema_ExtElC2d of the pairs that use it (setting `inverse` when the
    // arguments are swapped), None for the pairs of Extrema_ECC2d.
    fn elementary_extrema(
        &mut self,
        c1: &dyn Adaptor2dCurve2d,
        c2: &dyn Adaptor2dCurve2d,
        types: (CurveType, CurveType),
        tol: f64,
    ) -> Result<Option<ExtElC2d>, Failure> {
        use CurveType::{Circle, Ellipse, Hyperbola, Line, Parabola};
        let xtrem = match types {
            (Circle, Line)
            | (Ellipse, Line)
            | (Parabola, Line)
            | (Hyperbola, Line)
            | (Ellipse, Circle)
            | (Parabola, Circle)
            | (Hyperbola, Circle) => {
                // The algorithm takes the arguments in the inverse order.
                self.inverse = true;
                return self.elementary_extrema(c2, c1, (types.1, types.0), tol);
            }
            (Circle, Circle) => ExtElC2d::circles(c1.circle()?, c2.circle()?)?,
            (Circle, Ellipse) => ExtElC2d::circle_ellipse(c1.circle()?, c2.ellipse()?)?,
            (Circle, Parabola) => ExtElC2d::circle_parabola(c1.circle()?, c2.parabola()?)?,
            (Circle, Hyperbola) => ExtElC2d::circle_hyperbola(c1.circle()?, c2.hyperbola()?)?,
            (Line, Line) => ExtElC2d::lines(c1.line()?, c2.line()?, tol)?,
            (Line, Circle) => ExtElC2d::line_circle(c1.line()?, c2.circle()?, tol)?,
            (Line, Ellipse) => ExtElC2d::line_ellipse(c1.line()?, c2.ellipse()?)?,
            (Line, Parabola) => ExtElC2d::line_parabola(c1.line()?, c2.parabola()?)?,
            (Line, Hyperbola) => ExtElC2d::line_hyperbola(c1.line()?, c2.hyperbola()?)?,
            _ => return Ok(None),
        };
        Ok(Some(xtrem))
    }

    // Extrema_ECC2d on the whole curves, with the periods of the range checks.
    fn parametric_extrema(
        &mut self,
        c1: &dyn Adaptor2dCurve2d,
        c2: &dyn Adaptor2dCurve2d,
        (type1, type2): (CurveType, CurveType),
        ranges: [f64; 4],
    ) -> Result<(), Failure> {
        let mut solver = ECC2d::with_curves(c1, c2)?;
        solver.set_single_solution_flag(self.get_single_solution_flag());
        solver.perform()?;
        let elementary = |t: CurveType| (t as i32) <= (CurveType::Parabola as i32);
        let other_period = |curve: &dyn Adaptor2dCurve2d| -> Result<f64, Failure> {
            if Curve2dTool::is_periodic(curve)? {
                Curve2dTool::period(curve)
            } else {
                Ok(0.)
            }
        };
        let period1: f64 = if elementary(type1) {
            conic_period(type1)
        } else {
            other_period(c1)?
        };
        let period2: f64 = if elementary(type1) && elementary(type2) {
            conic_period(type2)
        } else {
            other_period(c2)?
        };
        self.results_ecc(&solver, ranges, [period1, period2])
    }
}

impl ExtCC2d<'_> {
    // `(U, U2)` moved into the periods (0 for none) and checked against the ranges
    // within Precision::PConfusion().
    fn in_ranges(u: f64, u2: f64, [ut11, ut12, ut21, ut22]: [f64; 4]) -> bool {
        let tol: f64 = precision::p_confusion();
        (u >= ut11 - tol) && (u <= ut12 + tol) && (u2 >= ut21 - tol) && (u2 <= ut22 + tol)
    }
    fn in_period(u: f64, start: f64, period: f64) -> f64 {
        if period != 0.0 {
            elclib::in_period(u, start, start + period)
        } else {
            u
        }
    }

    /// `Results(Extrema_ExtElC2d, Ut11, Ut12, Ut21, Ut22, Period1, Period2)`
    /// (protected): `IsParallel()` is read before the done test, so a not-done
    /// extremum raises StdFail_NotDone, as natively.
    fn results_el(
        &mut self,
        alg: &ExtElC2d,
        ranges: [f64; 4],
        [period1, period2]: [f64; 2],
    ) -> Result<(), Failure> {
        self.done = alg.is_done();
        self.is_par = alg.is_parallel()?;
        if !self.done {
            return Ok(());
        }
        if !self.is_par {
            for i in 1..=alg.nb_ext()? {
                // Verification of the validity of parameters for the trimmed case:
                let (mut p1, mut p2) = alg.points(i)?;
                let (raw_u, raw_u2) = if self.inverse {
                    (p2.parameter(), p1.parameter())
                } else {
                    (p1.parameter(), p2.parameter())
                };
                let u: f64 = Self::in_period(raw_u, ranges[0], period1);
                let u2: f64 = Self::in_period(raw_u2, ranges[2], period2);
                if !Self::in_ranges(u, u2, ranges) {
                    continue;
                }
                self.nb_ext += 1;
                self.sq_dist.push(alg.square_distance(i)?);
                if self.inverse {
                    p1.set_values(u2, *p1.value());
                    p2.set_values(u, *p2.value());
                    self.points.extend([p2, p1]);
                } else {
                    p1.set_values(u, *p1.value());
                    p2.set_values(u2, *p2.value());
                    self.points.extend([p1, p2]);
                }
            }
        }
        self.set_trimmed_distances();
        Ok(())
    }

    /// `Results(Extrema_ECC2d, Ut11, Ut12, Ut21, Ut22, Period1, Period2)` (protected).
    fn results_ecc(
        &mut self,
        alg: &ECC2d,
        ranges: [f64; 4],
        [period1, period2]: [f64; 2],
    ) -> Result<(), Failure> {
        self.done = alg.is_done();
        if !self.done {
            return Ok(());
        }
        self.is_par = alg.is_parallel()?;
        for i in 1..=alg.nb_ext()? {
            // Verification of parameter validity for the trimmed case:
            let (p1, p2) = alg.points(i)?;
            let u: f64 = Self::in_period(p1.parameter(), ranges[0], period1);
            let u2: f64 = Self::in_period(p2.parameter(), ranges[2], period2);
            if Self::in_ranges(u, u2, ranges) {
                self.nb_ext += 1;
                let value: f64 = alg.square_distance(i)?;
                self.sq_dist.push(value);
                self.points.extend([
                    POnCurv2d::new(u, *p1.value()),
                    POnCurv2d::new(u2, *p2.value()),
                ]);
            }
        }
        self.set_trimmed_distances();
        Ok(())
    }

    fn set_trimmed_distances(&mut self) {
        self.dist = [
            self.p1f.square_distance(self.p2f),
            self.p1f.square_distance(self.p2l),
            self.p1l.square_distance(self.p2f),
            self.p1l.square_distance(self.p2l),
        ];
    }

    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: StdFail_NotDone when not done; `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtCC2d::IsParallel()"));
        }
        Ok(self.is_par)
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtCC2d::NbExt()"));
        }
        Ok(self.nb_ext)
    }
    // StdFail_NotDone first, then Standard_OutOfRange outside `[1, NbExt()]`.
    fn check(&self, member: &str, n: i32) -> Result<(), Failure> {
        if !self.done {
            return Err(not_done(member));
        }
        if n <= 0 || n > self.nb_ext {
            return Err(out_of_range(member, n, self.nb_ext));
        }
        Ok(())
    }
    /// `SquareDistance(N)` (the native default is 1); `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let member: &str = "Extrema_ExtCC2d::SquareDistance()";
        self.check(member, n)?;
        sequence_value(member, &self.sq_dist, n)
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; `extrema.points(1)?`.
    pub fn points(&self, n: i32) -> Result<(POnCurv2d, POnCurv2d), Failure> {
        let member: &str = "Extrema_ExtCC2d::Points()";
        self.check(member, n)?;
        let first: POnCurv2d = sequence_value(member, &self.points, 2 * n - 1)?;
        Ok((first, sequence_value(member, &self.points, 2 * n)?))
    }
    /// `TrimmedSquareDistances(...)`, returned as `([dist11, dist12, dist21, dist22],
    /// [P11, P12, P21, P22])`; `extrema.trimmed_square_distances()`.
    pub fn trimmed_square_distances(&self) -> ([f64; 4], [Point2; 4]) {
        (self.dist, [self.p1f, self.p1l, self.p2f, self.p2l])
    }
}
