// Rust translation of GeomAPI_PointsToBSplineSurface.cxx (Init(Points, ParType, ...,
// thePeriodic) and Init(Points, Weight1, Weight2, Weight3, ...)) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The two passes of GeomAPI_PointsToBSplineSurface over a grid of points.
use super::points_to_bspline_surface::{PointsToBSplineSurface, SurfaceFitSettings};
use super::points_to_bspline_surface_tangent::{build_parameters, build_periodic_tangent};
use crate::geom2d_api::points_to_bspline::continuity_code;
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomBSplineSurface;
use rustycad_tkgeombase::app_def::{
    BSplineCompute, MultiLine, MultiPointConstraint, Variational, VariationalOptions,
};
use rustycad_tkgeombase::app_par_curves::{
    Constraint, ConstraintCouple, MultiBSpCurve, MultiCurveEval,
};
use rustycad_tkgeombase::approx::{ComputeLineOptions, ParametrizationType};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::Vector;

const MEMBER: &str = "GeomAPI_PointsToBSplineSurface";

// A fresh native array.
pub(super) fn array<T: Default>(lower: i32, upper: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(lower, upper).map_err(|e| e.into_failure(MEMBER))
}

// `Points(i, j)` (Standard_OutOfRange outside the grid).
fn grid_point(points: &Array2<'_, Point3>, i: i32, j: i32) -> Result<Point3, Failure> {
    points
        .value(i, j)
        .copied()
        .map_err(|e| e.into_failure(MEMBER))
}

/// The options of both AppDef_BSplineCompute passes: two iterations, cutting, least
/// squares when `Tol3D <= 1e-3`; the 2D tolerance is the 3D one.
fn compute_options(
    (deg_min, deg_max, _, tol3d): SurfaceFitSettings,
    par_type: ParametrizationType,
) -> ComputeLineOptions {
    ComputeLineOptions {
        degree_min: deg_min,
        degree_max: deg_max,
        tolerance_3d: tol3d,
        tolerance_2d: tol3d,
        nb_iterations: 2,
        cutting: true,
        parametrization: par_type,
        squares: tol3d <= 1.0e-3,
    }
}

// The multipoints of the U isos: column `j` of the grid holds the rows `Imin..Imax`
// (and row 1 again for a periodic surface) at the native indices `i` and `Imax + 1`.
fn column_lines(points: &Array2<'_, Point3>, periodic: bool) -> Result<MultiLine, Failure> {
    let (imin, imax) = (points.lower_row(), points.upper_row());
    let (jmin, jmax) = (points.lower_col(), points.upper_col());
    let add: i32 = if periodic { 2 } else { 1 };
    let line = MultiLine::with_nb_multi_points(jmax - jmin + 1)?;
    for j in jmin..=jmax {
        let mp = MultiPointConstraint::with_counts(imax - imin + add, 0)?;
        for i in imin..=imax {
            mp.set_point(i, grid_point(points, i, j)?)?;
        }
        if periodic {
            mp.set_point(imax + 1, grid_point(points, 1, j)?)?;
        }
        line.set_value(j, &mp)?;
    }
    Ok(line)
}

// A multiline of `count` multipoints, the poles of curve `i` of `the_curve` in the
// multipoint `i` (the native `Line2` of both methods).
fn pole_lines(the_curve: &MultiBSpCurve, count: i32) -> Result<MultiLine, Failure> {
    let mut poles: Array1<'static, Point3> = array(1, the_curve.nb_poles())?;
    let line2 = MultiLine::with_nb_multi_points(count)?;
    for i in 1..=count {
        the_curve.curve_3d(i, &mut poles)?;
        let mp = MultiPointConstraint::with_counts(poles.upper(), 0)?;
        for j in 1..=poles.upper() {
            mp.set_point(j, poles[j])?;
        }
        line2.set_value(i, &mp)?;
    }
    Ok(line2)
}

/// The degree, pole count, knots and multiplicities of one pass, read in the native
/// order (Degree, NbPoles, Knots, Multiplicities) right after the pass.
pub(super) struct PassCurve {
    pub(super) degree: i32,
    pub(super) nb_poles: i32,
    pub(super) knots: Array1<'static, f64>,
    pub(super) mults: Array1<'static, i32>,
}

impl PassCurve {
    pub(super) fn of(the_curve: &MultiBSpCurve) -> Result<Self, Failure> {
        let degree: i32 = the_curve.degree()?;
        let nb_poles: i32 = the_curve.nb_poles();
        let knots: Array1<'static, f64> = the_curve.knots()?;
        Ok(Self {
            degree,
            nb_poles,
            knots,
            mults: the_curve.multiplicities()?,
        })
    }
}

// The surface whose pole column `j` holds the poles of curve `j` of `the_curve2` (the
// U direction), with the knots of both passes.
fn surface_of(
    (the_curve2, u): (&MultiBSpCurve, &PassCurve),
    v: &PassCurve,
) -> Result<GeomBSplineSurface, Failure> {
    let mut poles2: Array1<'static, Point3> = array(1, u.nb_poles)?;
    let mut the_poles: Array2<'static, Point3> =
        Array2::with_bounds(1, u.nb_poles, 1, v.nb_poles).map_err(|e| e.into_failure(MEMBER))?;
    for j in 1..=v.nb_poles {
        the_curve2.curve_3d(j, &mut poles2)?;
        for i in 1..=u.nb_poles {
            the_poles[(i, j)] = poles2[i];
        }
    }
    GeomBSplineSurface::new(
        &the_poles,
        (&u.knots, &u.mults),
        (&v.knots, &v.mults),
        (u.degree, v.degree),
        (false, false),
    )
}

impl PointsToBSplineSurface {
    /// `Init(Points, ParType, DegMin, DegMax, Continuity, Tol3D, thePeriodic)`: the
    /// columns approximated (interpolated for `Tol3D <= 0`), then the rows of their
    /// poles; a periodic surface closes on row 1 and, for more than two rows of poles
    /// and a positive tolerance, gets an averaged end tangent. The native code sets the
    /// multipoints at the grid indices, so a grid not starting at (1, 1) raises
    /// Standard_OutOfRange. `fit.init_parametrization(&grid,
    /// ParametrizationType::ChordLength, (3, 8, Shape::C2, 1e-3), false)?`.
    pub fn init_parametrization(
        &mut self,
        points: &Array2<'_, Point3>,
        par_type: ParametrizationType,
        settings: SurfaceFitSettings,
        the_periodic: bool,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let line = column_lines(points, the_periodic)?;
        let mut the_computer = BSplineCompute::new(compute_options(settings, par_type));
        the_computer.set_continuity(continuity_code(settings.2));
        run_compute(&mut the_computer, &line, settings.3)?;
        let the_curve: &MultiBSpCurve = the_computer.value();
        let v: PassCurve = PassCurve::of(the_curve)?;
        let add: i32 = if the_periodic { 2 } else { 1 };
        let nbisosu: i32 = points.upper_row() - points.lower_row() + add;
        let mut line2 = pole_lines(the_curve, nbisosu)?;
        let mut the_computer2 = BSplineCompute::new(compute_options(settings, par_type));
        let closed = the_periodic && line2.nb_multi_points()? > 2;
        if settings.3 <= 0.0 {
            if the_periodic {
                the_computer2.set_periodic(the_periodic);
            }
            the_computer2.interpol(&line2)?;
        } else {
            if closed {
                the_computer2.set_periodic(the_periodic);
                line2 = with_periodic_tangents(line2, the_curve, par_type)?;
            }
            the_computer2.perform(&line2)?;
        }
        let u: PassCurve = PassCurve::of(the_computer2.value())?;
        let surface = surface_of((the_computer2.value(), &u), &v)?;
        if the_periodic && line2.nb_multi_points()? > 2 {
            surface.set_u_periodic()?;
        }
        self.my_surface = Handle::new(surface);
        self.my_is_done = true;
        Ok(())
    }
}

// `Interpol(Line)` for a non-positive tolerance, `Perform(Line)` otherwise.
fn run_compute(compute: &mut BSplineCompute, line: &MultiLine, tol3d: f64) -> Result<(), Failure> {
    if tol3d <= 0.0 {
        compute.interpol(line)
    } else {
        compute.perform(line)
    }
}

// The first and last multipoints of `line2` rebuilt from the first and last curves of
// `the_curve` with the tangent BuildPeriodicTangent averages at both ends.
fn with_periodic_tangents(
    line2: MultiLine,
    the_curve: &MultiBSpCurve,
    par_type: ParametrizationType,
) -> Result<MultiLine, Failure> {
    let count: i32 = line2.nb_multi_points()?;
    let mut a_pars: Array1<'static, f64> = array(1, count)?;
    build_parameters(&line2, par_type, &mut a_pars)?;
    let nb_poles: i32 = the_curve.nb_poles();
    let mut a_tang: Vector<'static> = Vector::with_bounds(1, 3 * nb_poles)?;
    build_periodic_tangent(&line2, &a_pars, &mut a_tang)?;
    for ind in [1, count] {
        let mut poles: Array1<'static, Point3> = array(1, nb_poles)?;
        the_curve.curve_3d(ind, &mut poles)?;
        let mut mp = MultiPointConstraint::with_counts(nb_poles, 0)?;
        for j in 1..=nb_poles {
            mp.set_point(j, poles[j])?;
            let k: i32 = 3 * (j - 1);
            let a_t = Vector3::new(a_tang[k + 1], a_tang[k + 2], a_tang[k + 3]);
            mp.set_tang(j, a_t)?;
        }
        line2.set_value(ind, &mp)?;
    }
    Ok(line2)
}

// The AppDef_Variational pass over `line` (`count` multipoints): not done (None) when
// it is not created, over-constrained, fails (its exception is swallowed, as natively)
// or not done.
fn variational_pass(
    line: &MultiLine,
    count: i32,
    (deg_max, continuity, tol3d): (i32, Shape, f64),
    (w1, w2, w3): (f64, f64, f64),
) -> Result<Option<MultiBSpCurve>, Failure> {
    let mut couples: Array1<'static, ConstraintCouple> = array(1, count)?;
    for i in 1..=count {
        couples[i] = ConstraintCouple::new(i, Constraint::NoConstraint);
    }
    let tab_of_cc = Handle::new(HArray1::from_owned(couples));
    let mut variation =
        Variational::new(line, 1, count, &tab_of_cc, VariationalOptions::default())?;
    variation.set_max_degree(deg_max)?;
    variation.set_continuity(continuity)?;
    variation.set_max_segment(1000);
    variation.set_tolerance(tol3d)?;
    variation.set_with_min_max(false)?;
    variation.set_nb_iterations(if tol3d <= 1.0e-3 { 0 } else { 2 });
    variation.set_criterium_weight(w1, w2, w3)?;
    if !variation.is_created() || variation.is_over_constrained() {
        return Ok(None);
    }
    if variation.approximate().is_err() || !variation.is_done() {
        return Ok(None);
    }
    Ok(Some(variation.value()?))
}

impl PointsToBSplineSurface {
    /// `Init(Points, Weight1, Weight2, Weight3, DegMax, Continuity, Tol3D)`: the two
    /// passes by AppDef_Variational smoothing (1000 segments, 2 iterations, none when
    /// `Tol3D <= 1e-3`); not done when a pass fails. The multipoints are set at the
    /// grid indices, as in the other Init.
    /// `fit.init_weights(&grid, (1.0, 0.0, 0.0), 8, Shape::C2, 1e-3)?`.
    pub fn init_weights(
        &mut self,
        points: &Array2<'_, Point3>,
        weights: (f64, f64, f64),
        deg_max: i32,
        continuity: Shape,
        tol3d: f64,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        let nb_point_j: i32 = points.upper_col() - points.lower_col() + 1;
        let nb_point_i: i32 = points.upper_row() - points.lower_row() + 1;
        let line = column_lines(points, false)?;
        let options = (deg_max, continuity, tol3d);
        let Some(the_curve) = variational_pass(&line, nb_point_j, options, weights)? else {
            return Ok(());
        };
        let v: PassCurve = PassCurve::of(&the_curve)?;
        let line2 = pole_lines(&the_curve, nb_point_i)?;
        let Some(the_curve2) = variational_pass(&line2, nb_point_i, options, weights)? else {
            return Ok(());
        };
        let u: PassCurve = PassCurve::of(&the_curve2)?;
        self.my_surface = Handle::new(surface_of((&the_curve2, &u), &v)?);
        self.my_is_done = true;
        Ok(())
    }
}
