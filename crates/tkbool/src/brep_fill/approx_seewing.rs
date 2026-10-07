// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_ApproxSeewing.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_ApproxSeewing`: the B-spline approximation of a multi-line through 50
//! points at about equal chords (degree-1 polygons when the approximation fails).
use super::BRepFillMultiLine;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkgeombase::app_def::{Compute, MultiLine, MultiPointConstraint};
use rustycad_tkgeombase::app_par_curves::MultiCurveEval;
use rustycad_tkgeombase::approx::ComputeLineOptions;
use rustycad_tkmath::gp::{Point2, Point3};

/// `NbPoints`: the points of the fit.
const NB_POINTS: i32 = 50;

/// The B-spline approximation of a multi-line (`BRepFill_ApproxSeewing`);
/// `let app = BRepFillApproxSeewing::with_multi_line(&ml)?;`.
#[derive(Default)]
pub struct BRepFillApproxSeewing {
    my_is_done: bool,
    my_curve: Handle<dyn GeomCurve>,
    my_p_curve1: Handle<dyn Geom2dCurve>,
    my_p_curve2: Handle<dyn Geom2dCurve>,
}

/// An array error as a native exception.
fn array_failure(e: impl std::fmt::Debug) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill_ApproxSeewing: {e:?}; expected an index in the array bounds"),
        "NCollection_Array1",
    )
}

/// Standard_OutOfRange of a chord past the length table (an out-of-bounds read
/// natively).
fn off_table(index: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("BRepFill_ApproxSeewing: length table index {index}; expected 1..=100"),
        "NCollection_Array1",
    )
}

/// The point constraint of one 3d point and the points on both faces (at 2 and 3).
fn constraint_of(ml: &BRepFillMultiLine, u: f64) -> Result<MultiPointConstraint, Failure> {
    let (p3d, pf1, pf2) = ml.value_3d_on_f1_on_f2(u)?;
    let mp = MultiPointConstraint::with_counts(1, 2)?;
    mp.set_point(1, p3d)?;
    mp.set_point2d(2, pf1)?;
    mp.set_point2d(3, pf2)?;
    Ok(mp)
}

/// The table length <-> parameter of `2 * NbPoints` points of the 3d curve.
fn length_table(
    ml: &BRepFillMultiLine,
    (u1, u2): (f64, f64),
) -> Result<(Vec<Point2>, f64), Failure> {
    let du: f64 = (u2 - u1) / f64::from(2 * NB_POINTS - 1);
    let mut lp: Vec<Point2> = Vec::with_capacity((2 * NB_POINTS) as usize);
    let mut length: f64 = 0.;
    let mut a_pnt1: Point3 = ml.value_3d(u1)?;
    for i in 0..2 * NB_POINTS {
        let a_pnt2: Point3 = ml.value_3d(u1 + f64::from(i) * du)?;
        length += a_pnt1.distance(&a_pnt2);
        lp.push(Point2::new(length, u1 + f64::from(i) * du));
        a_pnt1 = a_pnt2;
    }
    Ok((lp, length))
}

/// The multi-line of NbPoints points at about equal chords of the 3d curve.
fn chord_points(ml: &BRepFillMultiLine, (u1, u2): (f64, f64)) -> Result<MultiLine, Failure> {
    let (lp, length) = length_table(ml, (u1, u2))?;
    let mls = MultiLine::with_nb_multi_points(NB_POINTS)?;
    mls.set_value(1, &constraint_of(ml, u1)?)?;
    let d_corde: f64 = length / f64::from(NB_POINTS - 1);
    let mut corde: f64 = d_corde;
    // `Index` is 1-based natively: lp[index - 1] is LP(Index).
    let mut index: usize = 1;
    for i in 2..NB_POINTS {
        while lp.get(index - 1).ok_or_else(|| off_table(index))?.x() < corde {
            index += 1;
        }
        // LP(Index - 1) is read out of the table natively when the first chord is null.
        let prev: Point2 = *lp
            .get(index.wrapping_sub(2))
            .ok_or_else(|| off_table(index - 1))?;
        let cur: Point2 = lp[index - 1];
        let alpha: f64 = (corde - prev.x()) / (cur.x() - prev.x());
        let u: f64 = prev.y() + alpha * (cur.y() - prev.y());
        mls.set_value(i, &constraint_of(ml, u)?)?;
        corde = f64::from(i) * d_corde;
    }
    mls.set_value(NB_POINTS, &constraint_of(ml, u2)?)?;
    Ok(mls)
}

impl BRepFillApproxSeewing {
    /// `BRepFill_ApproxSeewing()`: not done; `BRepFillApproxSeewing::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepFill_ApproxSeewing(ML)`: `Perform` of the multi-line.
    pub fn with_multi_line(ml: &BRepFillMultiLine) -> Result<Self, Failure> {
        let mut app = Self::new();
        app.perform(ml)?;
        Ok(app)
    }

    /// `Perform(ML)`: the AppDef_Compute fit (default parameters) of 50 points at
    /// about equal chords; the degree-1 polygons through them on the multi-line
    /// parameters when it gives no curve; `app.perform(&ml)?`.
    pub fn perform(&mut self, ml: &BRepFillMultiLine) -> Result<(), Failure> {
        let range = (ml.first_parameter_of()?, ml.last_parameter_of()?);
        let mls = chord_points(ml, range)?;
        let mut fit = Compute::with_line(&mls, ComputeLineOptions::default())?;
        if fit.nb_multi_curves() == 0 {
            // The approximation failed: the polygons.
            return self.polygons(&mls, range);
        }
        let mbsp = fit.spline_value()?;
        let nb_poles: i32 = mbsp.nb_poles();
        let mut poles: Array1<'static, Point3> =
            Array1::with_bounds(1, nb_poles).map_err(array_failure)?;
        let mut poles2d1: Array1<'static, Point2> =
            Array1::with_bounds(1, nb_poles).map_err(array_failure)?;
        let mut poles2d2: Array1<'static, Point2> =
            Array1::with_bounds(1, nb_poles).map_err(array_failure)?;
        mbsp.curve_3d(1, &mut poles)?;
        mbsp.curve_2d(2, &mut poles2d1)?;
        mbsp.curve_2d(3, &mut poles2d2)?;
        let (knots, mults, degree) = (mbsp.knots()?, mbsp.multiplicities()?, mbsp.degree()?);
        self.set_curves((poles, poles2d1, poles2d2), (&knots, &mults, degree))
    }

    /// The degree-1 polygons through the points of the multi-line at its parameters.
    fn polygons(&mut self, mls: &MultiLine, (uf, ul): (f64, f64)) -> Result<(), Failure> {
        let n = NB_POINTS;
        let mut knots: Array1<'static, f64> = Array1::with_bounds(1, n).map_err(array_failure)?;
        let mut mults: Array1<'static, i32> = Array1::with_bounds(1, n).map_err(array_failure)?;
        let mut p: Array1<'static, Point3> = Array1::with_bounds(1, n).map_err(array_failure)?;
        let mut p1: Array1<'static, Point2> = Array1::with_bounds(1, n).map_err(array_failure)?;
        let mut p2: Array1<'static, Point2> = Array1::with_bounds(1, n).map_err(array_failure)?;
        let d_ulf: f64 = (ul - uf) / f64::from(n - 1);
        for i in 1..=n {
            let mpc = mls.value(i)?;
            mults[i] = if i == 1 || i == n { 2 } else { 1 };
            (p[i], p1[i], p2[i]) = (mpc.point(1)?, mpc.point2d(2)?, mpc.point2d(3)?);
            knots[i] = if i == n {
                ul
            } else {
                uf + f64::from(i - 1) * d_ulf
            };
        }
        self.set_curves((p, p1, p2), (&knots, &mults, 1))
    }

    /// The B-splines of the poles on the knots.
    #[allow(clippy::type_complexity)] // The three pole arrays.
    fn set_curves(
        &mut self,
        (poles, poles2d1, poles2d2): (
            Array1<'static, Point3>,
            Array1<'static, Point2>,
            Array1<'static, Point2>,
        ),
        (knots, mults, degree): (&Array1<'static, f64>, &Array1<'static, i32>, i32),
    ) -> Result<(), Failure> {
        self.my_curve =
            Handle::new(GeomBSplineCurve::new(&poles, knots, mults, degree, false)?).cast();
        self.my_p_curve1 = Handle::new(Geom2dBSplineCurve::new(
            &poles2d1, knots, mults, degree, false,
        )?)
        .cast();
        self.my_p_curve2 = Handle::new(Geom2dBSplineCurve::new(
            &poles2d2, knots, mults, degree, false,
        )?)
        .cast();
        self.my_is_done = true;
        Ok(())
    }

    /// `IsDone()`; `app.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// The StdFail_NotDone of the results before a successful Perform.
    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if self.my_is_done {
            return Ok(());
        }
        Err(Failure::with_type(
            "StdFail_NotDone",
            &format!("BRepFill_ApproxSeewing::{member}"),
            "BRepFill_ApproxSeewing",
        ))
    }

    /// `Curve()`: the 3d curve; StdFail_NotDone before Perform; `app.curve()?`.
    pub fn curve(&self) -> Result<&Handle<dyn GeomCurve>, Failure> {
        self.check_done("Curve")?;
        Ok(&self.my_curve)
    }

    /// `CurveOnF1()`: the p-curve on the first face; `app.curve_on_f1()?`.
    pub fn curve_on_f1(&self) -> Result<&Handle<dyn Geom2dCurve>, Failure> {
        self.check_done("CurveOnF1")?;
        Ok(&self.my_p_curve1)
    }

    /// `CurveOnF2()`: the p-curve on the second face; `app.curve_on_f2()?`.
    pub fn curve_on_f2(&self) -> Result<&Handle<dyn Geom2dCurve>, Failure> {
        self.check_done("CurveOnF2")?;
        Ok(&self.my_p_curve2)
    }
}
