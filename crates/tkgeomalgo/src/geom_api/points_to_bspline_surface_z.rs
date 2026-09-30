// Rust translation of GeomAPI_PointsToBSplineSurface.cxx (Init(ZPoints, X0, dX, Y0,
// dY, ...)) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The heights overload of GeomAPI_PointsToBSplineSurface: the heights fitted along
//! the columns then along the poles, with X and Y rebuilt from the grid lines.
use super::points_to_bspline_surface::{HeightGrid, PointsToBSplineSurface, SurfaceFitSettings};
use super::points_to_bspline_surface_fit::{PassCurve, array};
use crate::geom2d_api::points_to_bspline::continuity_code;
use crate::geom2d_api::points_to_bspline_y::{LinearCoordinate, linear_coordinate};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkgeombase::app_def::{BSplineCompute, MultiLine, MultiPointConstraint};
use rustycad_tkgeombase::app_par_curves::{MultiBSpCurve, MultiCurveEval};
use rustycad_tkgeombase::approx::{ComputeLineOptions, ParametrizationType};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::Vector;

const MEMBER: &str = "GeomAPI_PointsToBSplineSurface";

// `AppDef_BSplineCompute(Parameters, DegMin, DegMax, Tol3D, Tol2D, 0, true, true)`.
fn height_computer(
    parameters: &Vector<'_>,
    (deg_min, deg_max, _, tol3d): SurfaceFitSettings,
) -> Result<BSplineCompute, Failure> {
    let options = ComputeLineOptions {
        degree_min: deg_min,
        degree_max: deg_max,
        tolerance_3d: tol3d,
        tolerance_2d: tol3d,
        nb_iterations: 0,
        cutting: true,
        parametrization: ParametrizationType::IsoParametric,
        squares: true,
    };
    BSplineCompute::from_parameters(parameters, options)
}

// `Interpol(Line)` for a non-positive tolerance, `Perform(Line)` otherwise.
fn run(compute: &mut BSplineCompute, line: &MultiLine, tol3d: f64) -> Result<(), Failure> {
    if tol3d <= 0.0 {
        compute.interpol(line)
    } else {
        compute.perform(line)
    }
}

// `(index - start) / span`, or 0 for an empty span.
fn uniform(index: i32, start: i32, span: i32) -> f64 {
    if span > 0 {
        f64::from(index - start) / f64::from(span)
    } else {
        0.0
    }
}

// The columns of heights as 2D points `(0, Z(i, j))` at the native grid indices, with
// their uniform parameters.
fn height_columns(z_points: &Array2<'_, f64>) -> Result<(MultiLine, Vector<'static>), Failure> {
    let (imin, imax) = (z_points.lower_row(), z_points.upper_row());
    let (jmin, jmax) = (z_points.lower_col(), z_points.upper_col());
    let line = MultiLine::with_nb_multi_points(jmax - jmin + 1)?;
    let mut param: Vector<'static> = Vector::with_bounds(jmin, jmax)?;
    for j in jmin..=jmax {
        let mp = MultiPointConstraint::with_counts(0, imax - imin + 1)?;
        for i in imin..=imax {
            let z: f64 = *z_points.value(i, j).map_err(|e| e.into_failure(MEMBER))?;
            mp.set_point2d(i, Point2::new(0.0, z))?;
        }
        param[j] = uniform(j, jmin, jmax - jmin);
        line.set_value(j, &mp)?;
    }
    Ok((line, param))
}

// The second line: the heights of the poles of each column curve, with uniform
// parameters.
fn pole_heights(
    the_curve: &MultiBSpCurve,
    nbisosu: i32,
) -> Result<(MultiLine, Vector<'static>), Failure> {
    let mut poles: Array1<'static, Point2> = array(1, the_curve.nb_poles())?;
    let line2 = MultiLine::with_nb_multi_points(nbisosu)?;
    let mut param2: Vector<'static> = Vector::with_bounds(1, nbisosu)?;
    for i in 1..=nbisosu {
        the_curve.curve_2d(i, &mut poles)?;
        let mp = MultiPointConstraint::with_counts(0, poles.upper())?;
        for j in 1..=poles.upper() {
            mp.set_point2d(j, Point2::new(0.0, poles[j].y()))?;
        }
        param2[i] = uniform(i, 1, nbisosu - 1);
        line2.set_value(i, &mp)?;
    }
    Ok((line2, param2))
}

// The surface of the X poles, the Y poles and the heights of the poles of the curves
// of `the_curve2`, with the knots of the rebuilt X and Y coordinates.
fn height_surface(
    the_curve2: &MultiBSpCurve,
    (x, y): (&LinearCoordinate, &LinearCoordinate),
    (u, v): (&PassCurve, &PassCurve),
) -> Result<GeomBSplineSurface, Failure> {
    let mut poles2: Array1<'static, Point2> = array(1, u.nb_poles)?;
    let mut the_poles: Array2<'static, Point3> =
        Array2::with_bounds(1, u.nb_poles, 1, v.nb_poles).map_err(|e| e.into_failure(MEMBER))?;
    for j in 1..=v.nb_poles {
        the_curve2.curve_2d(j, &mut poles2)?;
        for i in 1..=u.nb_poles {
            the_poles[(i, j)] = Point3::new(x.poles[i], y.poles[j], poles2[i].y());
        }
    }
    GeomBSplineSurface::new(
        &the_poles,
        (&x.knots, &x.mults),
        (&y.knots, &y.mults),
        (u.degree, v.degree),
        (false, false),
    )
}

impl PointsToBSplineSurface {
    /// `Init(ZPoints, X0, dX, Y0, dY, DegMin, DegMax, Continuity, Tol3D)`: the heights
    /// of each column fitted over uniform parameters (interpolated for `Tol3D <= 0`),
    /// then the heights of their poles; X and Y are the lines `X0 + dX (i - Imin)` and
    /// `Y0 + dY (j - Jmin)` raised to the degrees and refined with the fitted knots. The
    /// second pass keeps the default continuity, as natively.
    /// `fit.init_heights(&z, (0.0, 1.0, 0.0, 1.0), (3, 8, Shape::C2, 1e-3))?`.
    pub fn init_heights(
        &mut self,
        z_points: &Array2<'_, f64>,
        (x0, dx, y0, dy): HeightGrid,
        settings: SurfaceFitSettings,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let (imin, imax) = (z_points.lower_row(), z_points.upper_row());
        let (jmin, jmax) = (z_points.lower_col(), z_points.upper_col());
        let length_y: f64 = dy * f64::from(jmax - jmin);
        let (line, param) = height_columns(z_points)?;
        let mut the_computer = height_computer(&param, settings)?;
        the_computer.set_continuity(continuity_code(settings.2));
        run(&mut the_computer, &line, settings.3)?;
        let the_curve: &MultiBSpCurve = the_computer.value();
        let v: PassCurve = PassCurve::of(the_curve)?;
        let y = linear_coordinate((v.degree, v.nb_poles), (&v.knots, &v.mults), y0, length_y)?;
        let nbisosu: i32 = imax - imin + 1;
        let length_x: f64 = dx * f64::from(imax - imin);
        let (line2, param2) = pole_heights(the_curve, nbisosu)?;
        let mut the_computer2 = height_computer(&param2, settings)?;
        run(&mut the_computer2, &line2, settings.3)?;
        let the_curve2: &MultiBSpCurve = the_computer2.value();
        let u: PassCurve = PassCurve::of(the_curve2)?;
        let x = linear_coordinate((u.degree, u.nb_poles), (&u.knots, &u.mults), x0, length_x)?;
        let surface = height_surface(the_curve2, (&x, &y), (&u, &v))?;
        self.my_surface = Handle::new(surface);
        self.my_is_done = true;
        Ok(())
    }
}
