// Rust translation of ModelingData/TKGeomBase/Geom2dConvert/Geom2dConvert.cxx
// (C0BSplineToC1BSplineCurve, C0BSplineToArrayOfC1BSplineCurve)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reduction of the C0 knots of a B-spline by C1 concatenation of its arcs.
use super::comp_curve::CompCurveToBSplineCurve;
use super::concat::{ConcatOutputs, concat_c1_with_angular};
use super::concat_support::{cloned_at, store, zero_based};
use super::reparametrize::new_curve_table;
use super::{HArrayOfBSplineCurves, copy_bspline, live_bspline};
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::{Point2, Vector2};

/// `Geom2dConvert::C0BSplineToC1BSplineCurve(BS, Tolerance)`: replaces `bs` by the
/// same geometry with its knots of multiplicity Degree reduced as far as a C1
/// concatenation of its arcs allows (the result can still be C0). The curve is
/// treated as closed when its ends meet within `tolerance` with parallel tangents
/// (1e-7 rad). A curve without such a knot is left as it is;
/// `geom2d_convert::c0_bspline_to_c1_bspline_curve(&mut curve, 1e-7)?`.
pub fn c0_bspline_to_c1_bspline_curve(
    bs: &mut Handle<Geom2dBSplineCurve>,
    tolerance: f64,
) -> Result<(), Failure> {
    let curve: &Geom2dBSplineCurve = live_bspline(bs)?;
    let nbcurve_c1: i32 = count_c1_arcs(curve, curve.first_u_knot_index()? + 1)?;
    if nbcurve_c1 <= 1 {
        return Ok(());
    }
    let mut array_of_curves = c1_arcs(curve, nbcurve_c1)?;
    let array_of_toler: Array1<'static, f64> = tolerances(nbcurve_c1, tolerance)?;
    let an_angular_toler: f64 = 1.0e-7;
    let (point1, v1, point2, v2) = end_tangents(curve)?;
    let mut closed_flag: bool = point1.square_distance(point2) < tolerance * tolerance
        && v1.is_parallel(v2, an_angular_toler)?;
    let mut array_of_indices: Handle<HArray1<i32>> = Handle::default();
    let mut array_of_concatenated: HArrayOfBSplineCurves = Handle::default();
    let outputs = ConcatOutputs {
        indices: &mut array_of_indices,
        concatenated: &mut array_of_concatenated,
    };
    concat_c1_with_angular(
        &mut array_of_curves,
        &array_of_toler,
        outputs,
        &mut closed_flag,
        tolerance,
        precision::angular(),
    )?;
    *bs = join_all(&array_of_concatenated, tolerance)?;
    Ok(())
}

/// `Geom2dConvert::C0BSplineToArrayOfC1BSplineCurve(BS, tabBS, Tolerance)`:
/// [`c0_bspline_to_array_of_c1_bspline_curve_with_angular`] with Precision::Angular();
/// `geom2d_convert::c0_bspline_to_array_of_c1_bspline_curve(&curve, &mut table, 1e-7)?`.
pub fn c0_bspline_to_array_of_c1_bspline_curve(
    bs: &Handle<Geom2dBSplineCurve>,
    tab_bs: &mut HArrayOfBSplineCurves,
    tolerance: f64,
) -> Result<(), Failure> {
    c0_bspline_to_array_of_c1_bspline_curve_with_angular(
        bs,
        tab_bs,
        precision::angular(),
        tolerance,
    )
}

/// `Geom2dConvert::C0BSplineToArrayOfC1BSplineCurve(BS, tabBS, AngularTolerance,
/// Tolerance)`: the C1 pieces of `bs` from ConcatC1 of its arcs, or `bs` alone.
///
/// Unlike [`c0_bspline_to_c1_bspline_curve`], the arc count includes the first knot,
/// and the closure test compares the square distance of the ends with `tolerance`
/// itself, as natively;
/// `geom2d_convert::c0_bspline_to_array_of_c1_bspline_curve_with_angular(&c, &mut t, 1e-9, 1e-7)?`.
pub fn c0_bspline_to_array_of_c1_bspline_curve_with_angular(
    bs: &Handle<Geom2dBSplineCurve>,
    tab_bs: &mut HArrayOfBSplineCurves,
    angular_tolerance: f64,
    tolerance: f64,
) -> Result<(), Failure> {
    let curve: &Geom2dBSplineCurve = live_bspline(bs)?;
    let nbcurve_c1: i32 = count_c1_arcs(curve, curve.first_u_knot_index()?)?;
    if nbcurve_c1 <= 1 {
        *tab_bs = new_curve_table(1)?;
        return put_single(tab_bs, bs.clone());
    }
    let mut array_of_curves = c1_arcs(curve, nbcurve_c1)?;
    let array_of_toler: Array1<'static, f64> = tolerances(nbcurve_c1, tolerance)?;
    let (point1, v1, point2, v2) = end_tangents(curve)?;
    let mut closed_flag: bool =
        point1.square_distance(point2) < tolerance && v1.is_parallel(v2, angular_tolerance)?;
    let mut array_of_indices: Handle<HArray1<i32>> = Handle::default();
    let outputs = ConcatOutputs {
        indices: &mut array_of_indices,
        concatenated: tab_bs,
    };
    concat_c1_with_angular(
        &mut array_of_curves,
        &array_of_toler,
        outputs,
        &mut closed_flag,
        tolerance,
        angular_tolerance,
    )
}

// 1 + the number of knots of multiplicity Degree from `from` to LastUKnotIndex - 1,
// capped at NbKnots - 1.
fn count_c1_arcs(bs: &Geom2dBSplineCurve, from: i32) -> Result<i32, Failure> {
    let mut nbcurve_c1: i32 = 1;
    for i in from..=(bs.last_u_knot_index()? - 1) {
        if bs.multiplicity(i)? == bs.degree() {
            nbcurve_c1 += 1;
        }
    }
    Ok(nbcurve_c1.min(bs.nb_knots() - 1))
}

// The arcs of `bs` between its knots of multiplicity Degree (and its bounds).
fn c1_arcs(
    bs: &Geom2dBSplineCurve,
    nbcurve_c1: i32,
) -> Result<Array1<'static, Handle<Geom2dBSplineCurve>>, Failure> {
    let mut array_of_curves: Array1<'static, Handle<Geom2dBSplineCurve>> = zero_based(nbcurve_c1)?;
    let mut u2: f64 = bs.first_parameter();
    let mut j: i32 = bs.first_u_knot_index()? + 1;
    for i in 0..nbcurve_c1 {
        let u1: f64 = u2;
        while j < bs.last_u_knot_index()? && bs.multiplicity(j)? < bs.degree() {
            j += 1;
        }
        u2 = bs.knot(j)?;
        j += 1;
        let arc: Handle<Geom2dBSplineCurve> = copy_bspline(bs)?;
        arc.segment(u1, u2, precision::p_confusion())?;
        store(&mut array_of_curves, i, arc)?;
    }
    Ok(array_of_curves)
}

// The junction tolerances, all equal to `tolerance`.
fn tolerances(nbcurve_c1: i32, tolerance: f64) -> Result<Array1<'static, f64>, Failure> {
    let mut array_of_toler: Array1<'static, f64> = zero_based(nbcurve_c1 - 1)?;
    for i in 0..=(nbcurve_c1 - 2) {
        store(&mut array_of_toler, i, tolerance)?;
    }
    Ok(array_of_toler)
}

// The points and first derivatives at both ends of the curve.
fn end_tangents(bs: &Geom2dBSplineCurve) -> Result<(Point2, Vector2, Point2, Vector2), Failure> {
    let (point1, v1): (Point2, Vector2) = bs.d1(bs.first_parameter())?;
    let (point2, v2): (Point2, Vector2) = bs.d1(bs.last_parameter())?;
    Ok((point1, v1, point2, v2))
}

// The concatenated pieces joined in turn (After = true); Standard_ConstructionError
// when one does not join.
fn join_all(
    pieces: &HArrayOfBSplineCurves,
    tolerance: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let table = pieces.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "Geom2dConvert: ConcatC1 left no table; expected curves",
        )
    })?;
    let pieces: Array1<'static, Handle<Geom2dBSplineCurve>> = table.array1().clone();
    let first: Handle<Geom2dBSplineCurve> = cloned_at(&pieces, 0)?;
    let mut comp = CompCurveToBSplineCurve::with_curve(
        &first.cast::<dyn Geom2dBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )?;
    for i in 1..pieces.length() {
        let next: Handle<Geom2dBSplineCurve> = cloned_at(&pieces, i)?;
        if !comp.add(&next.cast::<dyn Geom2dBoundedCurve>(), tolerance, true)? {
            return Err(failure(
                "Standard_ConstructionError",
                "Geom2dConvert Concatenation Error",
            ));
        }
    }
    Ok(comp.bspline_curve())
}

// tabBS(0) = BS.
fn put_single(
    tab_bs: &HArrayOfBSplineCurves,
    bs: Handle<Geom2dBSplineCurve>,
) -> Result<(), Failure> {
    let table = tab_bs.get().ok_or_else(|| {
        failure(
            "Standard_NullObject",
            "Geom2dConvert: null curve table; expected an array",
        )
    })?;
    table
        .change_array1()
        .set_value(0, bs)
        .map_err(|error| crate::native_failure::array_failure(error, "Geom2dConvert"))
}
