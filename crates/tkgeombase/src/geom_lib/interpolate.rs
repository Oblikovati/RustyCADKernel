// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib_Interpolate.hxx/.cxx/.lxx
// and GeomLib_InterpolationErrors.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib_Interpolate`: the B-spline through points at given parameters, with
//! null derivatives of every order up to half the degree at both ends.
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::gp::Point3;

/// `GeomLib_InterpolationErrors`, in the native enumerator order.
///
/// `assert_eq!(InterpolationErrors::InversionProblem as i32, 3);`
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InterpolationErrors {
    /// `GeomLib_NoError`.
    NoError,
    /// `GeomLib_NotEnoughtPoints`.
    NotEnoughtPoints,
    /// `GeomLib_DegreeSmallerThan3`.
    DegreeSmallerThan3,
    /// `GeomLib_InversionProblem`.
    InversionProblem,
}

/// The interpolation result: the curve and whether it was built, or the error.
///
/// Natively `myError` is not assigned on success (it is indeterminate there); the
/// port reads `NoError`.
///
/// `let interp = Interpolate::new(3, 5, &points, &parameters)?; if interp.is_done() { let c = interp.curve(); }`
#[derive(Clone)]
pub struct Interpolate {
    my_curve: Handle<GeomBSplineCurve>,
    my_is_done: bool,
    my_error: InterpolationErrors,
}

impl Interpolate {
    /// Interpolates the first `num_points` points at the first `num_points`
    /// parameters with a B-spline of degree `Degree - 1` when `Degree` is even (the
    /// order is the even number at most `Degree + 1`). NotEnoughtPoints when
    /// `num_points < degree` or the arrays do not start at 1 or are shorter;
    /// DegreeSmallerThan3 for `degree < 3`; InversionProblem when BSplCLib::Interpolate
    /// fails. The curve's knots are the whole `parameters` array, as natively
    /// (`GeomLib_Interpolate(Degree, NumPoints, Points, Parameters)`);
    /// `Interpolate::new(3, 5, &points, &parameters)?`.
    pub fn new(
        degree: i32,
        num_points: i32,
        points_array: &Array1<'_, Point3>,
        parameters_array: &Array1<'_, f64>,
    ) -> Result<Self, Failure> {
        let mut result = Self {
            my_curve: Handle::default(),
            my_is_done: false,
            my_error: InterpolationErrors::NoError,
        };
        if num_points < degree
            || points_array.lower() != 1
            || points_array.upper() < num_points
            || parameters_array.lower() != 1
            || parameters_array.upper() < num_points
        {
            result.my_error = InterpolationErrors::NotEnoughtPoints;
        } else if degree < 3 {
            result.my_error = InterpolationErrors::DegreeSmallerThan3;
        } else {
            result.interpolate(degree, num_points, points_array, parameters_array)?;
        }
        Ok(result)
    }

    // The interpolation problem: flat knots with the end parameters of multiplicity
    // `order`, contact orders 0 at the ends and inside, half_order + i - 1 for the
    // null derivatives at both ends.
    fn interpolate(
        &mut self,
        degree: i32,
        num_points: i32,
        points_array: &Array1<'_, Point3>,
        parameters_array: &Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let param = |i: i32| -> Result<f64, Failure> {
            parameters_array
                .value(i)
                .copied()
                .map_err(|e| array_failure(e, "Parameters"))
        };
        let point = |i: i32| -> Result<Point3, Failure> {
            points_array
                .value(i)
                .copied()
                .map_err(|e| array_failure(e, "Points"))
        };
        let order: i32 = if (degree + 1) % 2 != 0 {
            degree
        } else {
            degree + 1
        };
        let half_order: i32 = order / 2;
        let num_knots: i32 = num_points + 2 * order - 2;
        let num_controls: i32 = num_knots - order;
        let mut flat_knots: Array1<'static, f64> = one_based(num_knots)?;
        let mut multiplicities: Array1<'static, i32> = one_based(num_points)?;
        multiplicities.init(1);
        multiplicities[1] = order;
        multiplicities[num_points] = order;
        for (ii, jj) in (1..=order).zip(num_controls + 1..) {
            flat_knots[ii] = param(1)?;
            flat_knots[jj] = param(num_points)?;
        }
        for (ii, jj) in (2..num_points).zip(order + 1..) {
            flat_knots[jj] = param(ii)?;
        }
        let mut contacts: Array1<'static, i32> = one_based(num_controls)?;
        let mut parameters: Array1<'static, f64> = one_based(num_controls)?;
        let mut poles: Array1<'static, Point3> = one_based(num_controls)?;
        contacts.init(0);
        for (ii, jj) in (1..=half_order).zip((1..=num_controls).rev()) {
            contacts[ii] = half_order + ii - 1;
            parameters[ii] = param(1)?;
            poles[ii] = Point3::new(0.0, 0.0, 0.0);
            contacts[jj] = half_order + ii - 1;
            parameters[jj] = param(num_points)?;
            poles[jj] = Point3::new(0.0, 0.0, 0.0);
        }
        for (ii, jj) in (2..num_points).zip(half_order + 1..) {
            parameters[jj] = param(ii)?;
            poles[jj] = point(ii)?;
        }
        (contacts[1], contacts[num_controls]) = (0, 0);
        (poles[1], poles[num_controls]) = (point(1)?, point(num_points)?);
        let mut inversion_problem: i32 = 0;
        bsplclib::interpolate(
            order - 1,
            &flat_knots,
            &parameters,
            &contacts,
            &mut poles,
            &mut inversion_problem,
        )?;
        if inversion_problem != 0 {
            self.my_error = InterpolationErrors::InversionProblem;
            return Ok(());
        }
        self.my_curve = Handle::new(GeomBSplineCurve::new(
            &poles,
            parameters_array,
            &multiplicities,
            order - 1,
            false,
        )?);
        self.my_is_done = true;
        Ok(())
    }

    /// Whether the curve was built (`IsDone()`); `interp.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// The error of a failed interpolation (`Error()`); `interp.error()`.
    pub fn error(&self) -> InterpolationErrors {
        self.my_error
    }

    /// The interpolating curve, null when not done (`Curve()`); `interp.curve()`.
    pub fn curve(&self) -> Handle<GeomBSplineCurve> {
        self.my_curve.clone()
    }
}
