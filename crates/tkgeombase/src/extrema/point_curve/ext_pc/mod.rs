// Rust translation of Extrema_GGExtPC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GGExtPC`: every extremum of the distance between a point and a curve on
//! a parameter range: the closed form for the conics, math_FunctionRoots on the
//! Bezier range, on selected B-spline spans and on the C2 (or deflection) intervals
//! of the other curves, and the range ends that touch the point.
mod bspline;
mod general;

use super::gen_ext_pc::GGenExtPC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::elementary_point_curve::ElementaryPointCurve;
use crate::extrema::failures::{extremum_index, not_done, null_object, sequence_value};
use crate::extrema::gp_space::GpPoint;
use crate::extrema::pon::POnCurve;
use crate::gcpnts::PoleCurve;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;

/// `Extrema_GGExtPC` over one [`ExtremaCurveTool`]; `Extrema_ExtPC` is
/// [`ExtPC`](crate::extrema::ExtPC), `Extrema_ExtPC2d`
/// [`ExtPC2d`](crate::extrema::ExtPC2d). The object borrows the curve (the native
/// `TheCurve*`); a Perform before any curve raises Standard_NullObject (a null
/// dereference natively);
/// `ExtPC::with_curve(point, &adaptor, 1.0e-10)?.nb_ext()?`.
pub struct GGExtPC<'a, T: ExtremaCurveTool> {
    curve: Option<&'a T::Curve>,
    pf: T::Point,
    pl: T::Point,
    ext_p_el_c: T::ExtPElC,
    points: Vec<POnCurve<T::Point>>,
    done: bool,
    dist1: f64,
    dist2: f64,
    ext_pc: GGenExtPC<'a, T>,
    tol_u: f64,
    tol_f: f64,
    sample: i32,
    int_uinf: f64,
    int_usup: f64,
    uinf: f64,
    usup: f64,
    kind: CurveType,
    is_min: Vec<bool>,
    sq_dist: Vec<f64>,
}

impl<T: ExtremaCurveTool> Default for GGExtPC<'_, T> {
    fn default() -> Self {
        let infinite: f64 = precision::infinite();
        Self {
            curve: None,
            pf: T::Point::default(),
            pl: T::Point::default(),
            ext_p_el_c: T::ExtPElC::default(),
            points: Vec::new(),
            done: false,
            dist1: f64::MAX,
            dist2: f64::MAX,
            ext_pc: GGenExtPC::new(),
            tol_u: 0.0,
            tol_f: 0.0,
            sample: 17,
            int_uinf: infinite,
            int_usup: infinite,
            uinf: infinite,
            usup: infinite,
            kind: CurveType::OtherCurve,
            is_min: Vec::new(),
            sq_dist: Vec::new(),
        }
    }
}

impl<'a, T: ExtremaCurveTool> GGExtPC<'a, T> {
    /// `Extrema_GGExtPC()`; `ExtPC::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GGExtPC(P, C, Uinf, Usup, TolF)` (the native TolF default is 1e-10);
    /// `ExtPC::with_range(point, &adaptor, 0.0, 1.0, 1.0e-10)?`.
    pub fn with_range(
        point: T::Point,
        curve: &'a T::Curve,
        uinf: f64,
        usup: f64,
        tol_f: f64,
    ) -> Result<Self, Failure> {
        let mut extrema: Self = Self::new();
        extrema.initialize(curve, uinf, usup, tol_f)?;
        extrema.perform(point)?;
        Ok(extrema)
    }
    /// `Extrema_GGExtPC(P, C, TolF)`: the whole curve range;
    /// `ExtPC::with_curve(point, &adaptor, 1.0e-10)?`.
    pub fn with_curve(point: T::Point, curve: &'a T::Curve, tol_f: f64) -> Result<Self, Failure> {
        let (uinf, usup) = (T::first_parameter(curve)?, T::last_parameter(curve)?);
        Self::with_range(point, curve, uinf, usup, tol_f)
    }
    /// `Initialize(C, Uinf, Usup, TolF)`: the parameter tolerance is the curve
    /// resolution of Precision::Confusion(); `extrema.initialize(&adaptor, 0.0, 1.0, 1.0e-10)?`.
    pub fn initialize(
        &mut self,
        curve: &'a T::Curve,
        uinf: f64,
        usup: f64,
        tol_f: f64,
    ) -> Result<(), Failure> {
        self.curve = Some(curve);
        self.uinf = uinf;
        self.int_uinf = uinf;
        self.usup = usup;
        self.int_usup = usup;
        self.tol_f = tol_f;
        self.tol_u = T::resolution(curve, precision::confusion())?;
        self.kind = T::get_type(curve)?;
        self.done = false;
        self.dist1 = f64::MAX;
        self.dist2 = f64::MAX;
        self.sample = 17;
        Ok(())
    }
    fn curve(&self) -> Result<&'a T::Curve, Failure> {
        self.curve
            .ok_or_else(|| null_object("Extrema_GGExtPC::Perform()"))
    }
    /// `Perform(P)`; `extrema.perform(point)?`.
    pub fn perform(&mut self, point: T::Point) -> Result<(), Failure> {
        self.sq_dist.clear();
        self.points.clear();
        self.is_min.clear();
        self.sample = 17;
        let curve: &'a T::Curve = self.curve()?;
        self.end_distances(point, curve)?;
        let tol: f64 = precision::confusion();
        let (uinf, usup) = (self.uinf, self.usup);
        match self.kind {
            CurveType::Circle => {
                self.ext_p_el_c
                    .perform_circle(point, &T::circle(curve)?, tol, uinf, usup)?
            }
            CurveType::Ellipse => {
                self.ext_p_el_c
                    .perform_ellipse(point, &T::ellipse(curve)?, tol, uinf, usup)?
            }
            CurveType::Parabola => {
                self.ext_p_el_c
                    .perform_parabola(point, &T::parabola(curve)?, tol, uinf, usup)?
            }
            CurveType::Hyperbola => {
                self.ext_p_el_c
                    .perform_hyperbola(point, &T::hyperbola(curve)?, tol, uinf, usup)?
            }
            CurveType::Line => {
                self.ext_p_el_c
                    .perform_line(point, &T::line(curve)?, tol, uinf, usup)?
            }
            CurveType::BezierCurve => return self.perform_bezier(point, curve),
            CurveType::BSplineCurve => self.perform_bspline(point, curve)?,
            _ => self.perform_general(point, curve)?,
        }
        self.postprocess(point, curve)
    }
    // Pf, Pl and their square distances (RealLast() at an infinite end).
    fn end_distances(&mut self, point: T::Point, curve: &T::Curve) -> Result<(), Failure> {
        self.dist1 = f64::MAX;
        if !precision::is_infinite(self.uinf) {
            self.pf = T::value(curve, self.uinf)?;
            self.dist1 = point.square_distance_to(&self.pf);
        }
        self.dist2 = f64::MAX;
        if !precision::is_infinite(self.usup) {
            self.pl = T::value(curve, self.usup)?;
            self.dist2 = point.square_distance_to(&self.pl);
        }
        Ok(())
    }
    // The whole range with twice the pole count as samples (no postprocessing).
    fn perform_bezier(&mut self, point: T::Point, curve: &'a T::Curve) -> Result<(), Failure> {
        self.int_uinf = self.uinf;
        self.int_usup = self.usup;
        let bezier = T::bezier(curve)?;
        let poles: i32 = bezier
            .get()
            .ok_or_else(|| null_object("Extrema_GGExtPC: Bezier()"))?
            .pole_count();
        self.sample = poles * 2;
        self.ext_pc.initialize_curve(curve)?;
        self.interval_perform(point)
    }
}

impl<'a, T: ExtremaCurveTool> GGExtPC<'a, T> {
    // The range ends of the other curves (added when they touch the point), or the
    // closed-form extrema of the conics moved into the range.
    fn postprocess(&mut self, point: T::Point, curve: &T::Curve) -> Result<(), Failure> {
        let _ = point;
        let spline_like = matches!(
            self.kind,
            CurveType::BSplineCurve | CurveType::OffsetCurve | CurveType::OtherCurve
        );
        if spline_like {
            let sq_conf: f64 = precision::square_confusion();
            if self.dist1 < sq_conf || self.dist2 < sq_conf {
                self.add_touching_ends();
                self.done = true;
            }
            return Ok(());
        }
        let elementary = self.ext_p_el_c.point_extrema().clone();
        self.done = elementary.is_done();
        if !self.done {
            return Ok(());
        }
        for i in 1..=elementary.nb_ext()? {
            let mut pc: POnCurve<T::Point> = elementary.point(i)?;
            let u: f64 = self.periodic_parameter(curve, pc.parameter())?;
            if (u >= self.uinf - self.tol_u) && (u <= self.usup + self.tol_u) {
                pc.set_values(u, *elementary.point(i)?.value());
                self.sq_dist.push(elementary.square_distance(i)?);
                self.is_min.push(elementary.is_min(i)?);
                self.points.push(pc);
            }
        }
        Ok(())
    }
    // Prepends the first end and appends the last one when within
    // Precision::SquareConfusion() of the point and not already found.
    fn add_touching_ends(&mut self) {
        let (mut first_added, mut last_added) = (false, false);
        for point in &self.points {
            let u: f64 = point.parameter();
            if (u - self.uinf).abs() < self.tol_u {
                first_added = true;
            } else if (self.usup - u).abs() < self.tol_u {
                last_added = true;
            }
        }
        let sq_conf: f64 = precision::square_confusion();
        if !first_added && self.dist1 < sq_conf {
            self.sq_dist.insert(0, self.dist1);
            self.is_min.insert(0, true);
            self.points.insert(0, POnCurve::new(self.uinf, self.pf));
        }
        if !last_added && self.dist2 < sq_conf {
            self.sq_dist.push(self.dist2);
            self.is_min.push(true);
            self.points.push(POnCurve::new(self.usup, self.pl));
        }
    }
    // `ElCLib::InPeriod(U, myuinf, myuinf + Period)` on a periodic curve.
    fn periodic_parameter(&self, curve: &T::Curve, u: f64) -> Result<f64, Failure> {
        if T::is_periodic(curve)? {
            return Ok(elclib::in_period(
                u,
                self.uinf,
                self.uinf + T::period(curve)?,
            ));
        }
        Ok(u)
    }
    /// `IntervalPerform(P)` (protected): the roots on `[myintuinf, myintusup]` with
    /// `mysample` samples, each added once in the range.
    fn interval_perform(&mut self, point: T::Point) -> Result<(), Failure> {
        let range: (f64, f64) = (self.int_uinf, self.int_usup);
        self.ext_pc
            .initialize_sampling(self.sample, range, self.tol_u, self.tol_f);
        self.ext_pc.perform(point)?;
        self.done = self.ext_pc.is_done();
        if !self.done {
            return Ok(());
        }
        let curve: &T::Curve = self.curve()?;
        for i in 1..=self.ext_pc.nb_ext()? {
            let pc: POnCurve<T::Point> = self.ext_pc.point(i)?;
            let u: f64 = self.periodic_parameter(curve, pc.parameter())?;
            if (u >= self.uinf - self.tol_u) && (u <= self.usup + self.tol_u) {
                let (sq_dist, is_min) = (self.ext_pc.square_distance(i)?, self.ext_pc.is_min(i)?);
                self.add_sol(u, *pc.value(), sq_dist, is_min);
            }
        }
        Ok(())
    }
    /// `AddSol(U, P, SqDist, IsMin)` (protected): appends unless a solution lies within
    /// the parameter tolerance.
    fn add_sol(&mut self, u: f64, point: T::Point, sq_dist: f64, is_min: bool) {
        if self
            .points
            .iter()
            .any(|p| (p.parameter() - u).abs() <= self.tol_u)
        {
            return;
        }
        self.sq_dist.push(sq_dist);
        self.is_min.push(is_min);
        self.points.push(POnCurve::new(u, point));
    }
    // Appends one extremum without the duplicate test.
    fn push_extremum(&mut self, sq_dist: f64, is_min: bool, point: POnCurve<T::Point>) {
        self.sq_dist.push(sq_dist);
        self.is_min.push(is_min);
        self.points.push(point);
    }
}

impl<T: ExtremaCurveTool> GGExtPC<'_, T> {
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_GGExtPC::NbExt()"));
        }
        Ok(self.sq_dist.len() as i32)
    }
    /// `SquareDistance(N)`; Standard_OutOfRange outside `[1, NbExt()]`;
    /// `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let member: &str = "Extrema_GGExtPC::SquareDistance()";
        extremum_index(member, n, || self.nb_ext())?;
        sequence_value(member, &self.sq_dist, n)
    }
    /// `IsMin(N)`; `extrema.is_min(1)?`.
    pub fn is_min(&self, n: i32) -> Result<bool, Failure> {
        let member: &str = "Extrema_GGExtPC::IsMin()";
        extremum_index(member, n, || self.nb_ext())?;
        sequence_value(member, &self.is_min, n)
    }
    /// `Point(N)`; `extrema.point(1)?.parameter()`.
    pub fn point(&self, n: i32) -> Result<POnCurve<T::Point>, Failure> {
        let member: &str = "Extrema_GGExtPC::Point()";
        extremum_index(member, n, || self.nb_ext())?;
        sequence_value(member, &self.points, n)
    }
    /// `TrimmedSquareDistances(dist1, dist2, P1, P2)`: the square distances to the
    /// range ends and the end points, as `(dist1, dist2, P1, P2)`;
    /// `extrema.trimmed_square_distances()`.
    pub fn trimmed_square_distances(&self) -> (f64, f64, T::Point, T::Point) {
        (self.dist1, self.dist2, self.pf, self.pl)
    }
}
