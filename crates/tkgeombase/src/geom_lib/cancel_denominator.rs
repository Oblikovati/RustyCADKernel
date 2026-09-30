// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx (CanBeTreated,
// law_evaluator, CheckIfKnotExists, AddAKnot, BuildFlatKnot, FunctionMultiply,
// CancelDenominatorDerivative1D, CancelDenominatorDerivative)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::CancelDenominatorDerivative`: the multiplication of a rational surface
//! by a law that sets its denominator to 1 and cancels its derivative on the
//! boundaries.
use super::denominator_multiplier::DenominatorMultiplier;
use super::live;
use crate::array_access::grid;
use crate::hermit;
use crate::native_failure::failure;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomSurface};
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::bsplslib::{
    self, EvaluatorFunction, InterpolationKnots, MultiplySource, SurfaceSpan,
};
use rustycad_tkmath::gp::Point3;
use std::cell::RefCell;

/// `GeomLib::CancelDenominatorDerivative(BSurf, UDirection, VDirection)`: cancels,
/// on the boundaries, the first derivative of the denominator in the directions
/// asked and sets its value to 1; `bsurf` is replaced by the product surface.
///
/// Both directions are treated in the order that treats the lower degree first; a
/// V treatment exchanges U and V around the U one. A direction is treated only when
/// the surface is rational and not periodic in it, its U boundary weights are
/// proportional, and its first or last two weights differ; Standard_ConstructionError
/// ("GeomLib Multiplication Error") when the product fails;
/// `geom_lib::cancel_denominator_derivative(&mut surface, true, false)?`.
pub fn cancel_denominator_derivative(
    bsurf: &mut Handle<GeomBSplineSurface>,
    udirection: bool,
    vdirection: bool,
) -> Result<(), Failure> {
    if udirection && !vdirection {
        return cancel_1d(bsurf);
    }
    if !udirection && vdirection {
        return exchanged(bsurf);
    }
    if !(udirection && vdirection) {
        return Ok(());
    }
    // Optimize the treatment.
    let surface: &GeomBSplineSurface = live(bsurf, "GeomLib::CancelDenominatorDerivative")?;
    if surface.u_degree() <= surface.v_degree() {
        cancel_1d(bsurf)?;
        exchanged(bsurf)
    } else {
        exchanged(bsurf)?;
        cancel_1d(bsurf)
    }
}

// ExchangeUV, the U treatment, ExchangeUV (on the possibly new surface).
fn exchanged(bsurf: &mut Handle<GeomBSplineSurface>) -> Result<(), Failure> {
    live(bsurf, "GeomLib::CancelDenominatorDerivative")?.exchange_uv()?;
    cancel_1d(bsurf)?;
    live(bsurf, "GeomLib::CancelDenominatorDerivative")?.exchange_uv()
}

// `CanBeTreated`: rational in U and not periodic, the first and last weight rows
// proportional (within Precision::Confusion()), and at least one row whose first or
// last two weights differ.
fn can_be_treated(bsurf: &GeomBSplineSurface) -> Result<bool, Failure> {
    if !bsurf.is_u_rational() || bsurf.is_u_periodic() {
        return Ok(false);
    }
    let weights: Array2<'static, f64> = bsurf.weights_array(|w| w.clone());
    let at = |u: i32, v: i32| -> Result<f64, Failure> {
        crate::array_access::at(&weights, u, v, "Weights")
    };
    let (last_u, cols): (i32, i32) = (weights.col_length(), weights.row_length());
    let lambda: f64 = at(1, 1)? / at(last_u, 1)?;
    for v in 1..=cols {
        let boundary_ratio: f64 = at(1, v)? / (lambda * at(last_u, v)?);
        if (boundary_ratio - 1.0).abs() > precision::confusion() {
            return Ok(false);
        }
    }
    // The first and last weight pairs differ only before this treatment is applied.
    for v in 1..=cols {
        let first_ratio: f64 = at(1, v)? / at(2, v)?;
        let last_ratio: f64 = at(last_u - 1, v)? / at(last_u, v)?;
        if (first_ratio - 1.0).abs() > precision::confusion()
            || (last_ratio - 1.0).abs() > precision::confusion()
        {
            return Ok(true);
        }
    }
    Ok(false)
}

// `CancelDenominatorDerivative1D`: the U knots reparametrized onto [0, 1], the knots
// where the Hermit laws of the weight columns change (their extremes), the product
// by the denominator multiplier, and the original U range restored.
fn cancel_1d(bsurf: &mut Handle<GeomBSplineSurface>) -> Result<(), Failure> {
    let surface: &GeomBSplineSurface = live(bsurf, "GeomLib::CancelDenominatorDerivative")?;
    if !can_be_treated(surface)? {
        return Ok(());
    }
    let startu_value: f64 = surface.u_knot(1)?;
    let endu_value: f64 = surface.u_knot(surface.nb_u_knots())?;
    let mut bsurf_u_knots: Array1<'static, f64> = surface.u_knots();
    bsplclib::reparametrize(0.0, 1.0, &mut bsurf_u_knots)?;
    // Reparametrisation of the surface.
    surface.set_u_knots(&bsurf_u_knots)?;
    let (uknotmin, uknotmax) = extremal_knots(surface)?;
    // Multiplication.
    *bsurf = function_multiply(bsurf, uknotmin, uknotmax)?;
    let surface: &GeomBSplineSurface = live(bsurf, "GeomLib::CancelDenominatorDerivative")?;
    let mut bsurf_u_knots: Array1<'static, f64> = surface.u_knots();
    bsplclib::reparametrize(startu_value, endu_value, &mut bsurf_u_knots)?;
    surface.set_u_knots(&bsurf_u_knots)
}

// uknotmin, uknotmax: the extremal knots of the Hermit laws (Hermit::Solutionbis) of
// the rational curves of each weight column.
fn extremal_knots(surface: &GeomBSplineSurface) -> Result<(f64, f64), Failure> {
    let (mut uknotmin, mut uknotmax): (f64, f64) = (1.0, 0.0);
    let knots: Array1<'static, f64> = surface.u_knots();
    let mults: Array1<'static, i32> = surface.u_multiplicities();
    let weights: Array2<'static, f64> = surface.weights_array(|w| w.clone());
    let poles: Array2<'static, Point3> = surface.poles();
    let nb_u: i32 = surface.nb_u_poles();
    let mut curve_weights: Array1<'static, f64> = one_based(nb_u)?;
    let mut curve_poles: Array1<'static, Point3> = one_based(nb_u)?;
    for v in 1..=poles.row_length() {
        // Build each scalar weight function in the reusable pole and weight buffers.
        let (mut x, mut y): (f64, f64) = (1.0, 0.0);
        for u in 1..=nb_u {
            curve_weights[u] = crate::array_access::at(&weights, u, v, "Weights")?;
            curve_poles[u] = crate::array_access::at(&poles, u, v, "Poles")?;
        }
        let curve = GeomBSplineCurve::with_weights(
            &curve_poles,
            &curve_weights,
            &knots,
            &mults,
            surface.u_degree(),
            false,
        )?;
        hermit::solutionbis(
            &Handle::new(curve),
            &mut x,
            &mut y,
            precision::confusion(),
            precision::confusion(),
        )?;
        if x < uknotmin {
            uknotmin = x; // uknotmin, uknotmax: extremal knots
        }
        if x != 1.0 && x > uknotmax {
            uknotmax = x;
        }
        if y != 0.0 && y < uknotmin {
            uknotmin = y;
        }
        if y > uknotmax {
            uknotmax = y;
        }
    }
    Ok((uknotmin, uknotmax))
}

/// `law_evaluator`: the value of the denominator multiplier (error code 1 for a
/// derivative request).
struct LawEvaluator<'d> {
    my_denominator: &'d DenominatorMultiplier,
    // A failed Value would unwind natively; it is carried to the end of the product.
    failure: RefCell<Option<Failure>>,
}

impl EvaluatorFunction for LawEvaluator<'_> {
    fn evaluate(
        &self,
        derivative_request: i32,
        u: f64,
        v: f64,
        result: &mut f64,
        error_code: &mut i32,
    ) {
        if derivative_request != 0 {
            *error_code = 1;
            return;
        }
        match self.my_denominator.value(u, v) {
            Ok(value) => {
                *result = value;
                *error_code = 0;
            }
            Err(error) => {
                self.failure.borrow_mut().get_or_insert(error);
                *error_code = 1;
            }
        }
    }
}

// `FunctionMultiply`: the product of the surface by the denominator multiplier on
// the flat knots of a cubic with the extremal knots inserted, of degrees
// (UDegree + 3, 2 VDegree).
fn function_multiply(
    bsurf: &Handle<GeomBSplineSurface>,
    knotmin: f64,
    knotmax: f64,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let surface: &GeomBSplineSurface = live(bsurf, "GeomLib::CancelDenominatorDerivative")?;
    let (u_deg, v_deg): (i32, i32) = (surface.u_degree(), surface.v_degree());
    let law_flat_knots: Array1<'static, f64> = law_knots(knotmin, knotmax)?;
    let denominator = DenominatorMultiplier::new(bsurf, &law_flat_knots)?;
    let (newuknots, newumults) = build_flat_knot(
        &surface.u_knots(),
        &surface.u_multiplicities(),
        [3, u_deg + 3],
        [knotmin, knotmax],
    )?;
    let (newvknots, newvmults) = build_flat_knot(
        &surface.v_knots(),
        &surface.v_multiplicities(),
        [v_deg, 2 * v_deg],
        [1.0, 0.0],
    )?;
    let newuflatknots: Array1<'static, f64> = flat_knots(&newuknots, &newumults)?;
    let newvflatknots: Array1<'static, f64> = flat_knots(&newvknots, &newvmults)?;
    let new_num_u_poles: i32 = newuflatknots.length() - u_deg - 4;
    let new_num_v_poles: i32 = newvflatknots.length() - 2 * v_deg - 1;
    let mut numerator: Array2<'static, Point3> = grid(new_num_u_poles, new_num_v_poles)?;
    let mut denominator_poles: Array2<'static, f64> = grid(new_num_u_poles, new_num_v_poles)?;
    let ev = LawEvaluator {
        my_denominator: &denominator,
        failure: RefCell::default(),
    };
    multiply(
        surface,
        &ev,
        (&newuflatknots, &newvflatknots),
        (&mut numerator, &mut denominator_poles),
    )?;
    for u in 1..=new_num_u_poles {
        for v in 1..=new_num_v_poles {
            let w: f64 = crate::array_access::at(&denominator_poles, u, v, "NewDenominator")?;
            let mut p: Point3 = crate::array_access::at(&numerator, u, v, "NewNumerator")?;
            p.set_xyz(p.xyz() / w);
            crate::array_access::put(&mut numerator, u, v, p, "NewNumerator")?;
        }
    }
    Ok(Handle::new(GeomBSplineSurface::with_weights(
        &numerator,
        &denominator_poles,
        (&newuknots, &newumults),
        (&newvknots, &newvmults),
        (u_deg + 3, 2 * v_deg),
        (false, false),
    )?))
}

// BSplSLib::FunctionMultiply of the surface by the law; a non-null status (or a
// failure of the law) is Standard_ConstructionError ("GeomLib Multiplication Error").
fn multiply(
    surface: &GeomBSplineSurface,
    ev: &LawEvaluator<'_>,
    (u_flat, v_flat): (&Array1<'static, f64>, &Array1<'static, f64>),
    (numerator, denominator): (&mut Array2<'static, Point3>, &mut Array2<'static, f64>),
) -> Result<(), Failure> {
    let (u_knots, v_knots) = (surface.u_knots(), surface.v_knots());
    let (u_mults, v_mults) = (surface.u_multiplicities(), surface.v_multiplicities());
    let poles: Array2<'static, Point3> = surface.poles();
    let weights: Option<Array2<'static, f64>> = surface.weights();
    let span = SurfaceSpan {
        u: 0.0,
        v: 0.0,
        u_index: 0,
        v_index: 0,
        u_knots: &u_knots,
        v_knots: &v_knots,
        u_mults: Some(&u_mults),
        v_mults: Some(&v_mults),
        u_degree: surface.u_degree(),
        v_degree: surface.v_degree(),
        u_rat: true,
        v_rat: true,
        u_per: false,
        v_per: false,
    };
    let source = MultiplySource {
        surface: span,
        poles: &poles,
        weights: weights.as_ref(),
    };
    let knots = InterpolationKnots {
        u_flat_knots: u_flat,
        v_flat_knots: v_flat,
        u_new_degree: surface.u_degree() + 3,
        v_new_degree: 2 * surface.v_degree(),
    };
    let mut status: i32 = 0;
    let result =
        bsplslib::function_multiply(ev, &source, &knots, numerator, denominator, &mut status);
    if let Some(error) = ev.failure.borrow_mut().take() {
        return Err(error);
    }
    result?;
    if status != 0 {
        return Err(failure(
            "Standard_ConstructionError",
            "GeomLib Multiplication Error",
        ));
    }
    Ok(())
}

// The flat knots of the cubic law: knots 0 and 1 of multiplicity 4 with the
// extremal knots inserted.
fn law_knots(knotmin: f64, knotmax: f64) -> Result<Array1<'static, f64>, Failure> {
    let mut knots: Array1<'static, f64> = one_based(2)?;
    (knots[1], knots[2]) = (0.0, 1.0);
    let mut mults: Array1<'static, i32> = one_based(2)?;
    (mults[1], mults[2]) = (4, 4);
    let (new_knots, new_mults) = build_flat_knot(&knots, &mults, [0, 3], [knotmin, knotmax])?;
    flat_knots(&new_knots, &new_mults)
}

// BSplCLib::KnotSequence into an array of the multiplicity sum.
fn flat_knots(
    knots: &Array1<'static, f64>,
    mults: &Array1<'static, i32>,
) -> Result<Array1<'static, f64>, Failure> {
    let length: i32 = mults.iter().sum();
    let mut flat: Array1<'static, f64> = one_based(length)?;
    bsplclib::knot_sequence(knots, mults, &mut flat, false)?;
    Ok(flat)
}

// `CheckIfKnotExists`: a knot within Precision::Confusion().
fn check_if_knot_exists(knots: &Array1<'static, f64>, knot: f64) -> bool {
    knots
        .iter()
        .any(|k| (k - knot).abs() <= precision::confusion())
}

// `AddAKnot`: `knot_to_insert` inserted before the first greater-or-equal knot with
// multiplicity FinalDegree - 2, every other multiplicity raised by DegreeDelta.
fn add_a_knot(
    knots: &Array1<'static, f64>,
    mults: &Array1<'static, i32>,
    knot_to_insert: f64,
    [degree_delta, final_degree]: [i32; 2],
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let (k, m): (&[f64], &[i32]) = (knots.as_slice(), mults.as_slice());
    let result_size: usize = k.len() + 1;
    let mut new_knots: Vec<f64> = Vec::with_capacity(result_size);
    let mut new_mults: Vec<i32> = Vec::with_capacity(result_size);
    let mut input_index: usize = 0;
    while input_index < k.len() && k[input_index] < knot_to_insert {
        new_knots.push(k[input_index]);
        new_mults.push(m[input_index] + degree_delta);
        input_index += 1;
    }
    new_knots.push(knot_to_insert);
    new_mults.push(final_degree - 2);
    for output_index in input_index + 1..result_size {
        new_knots.push(k[output_index - 1]);
        new_mults.push(m[output_index - 1] + degree_delta);
    }
    Ok((one_based_from(&new_knots)?, one_based_from(&new_mults)?))
}

fn one_based_from<T: Default + Copy>(values: &[T]) -> Result<Array1<'static, T>, Failure> {
    let mut array: Array1<'static, T> = one_based(values.len() as i32)?;
    array.as_mut_slice().copy_from_slice(values);
    Ok(array)
}

// `BuildFlatKnot`: the knots and multiplicities (raised by DegreeDelta) with
// `knot_min` and `knot_max` inserted when missing: nothing when both exist, the
// missing one (the minimum when they are equal) when one exists, both otherwise
// (the maximum without a second degree raise).
fn build_flat_knot(
    surface_knots: &Array1<'static, f64>,
    surface_mults: &Array1<'static, i32>,
    [degree_delta, final_degree]: [i32; 2],
    [knot_min, knot_max]: [f64; 2],
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let has_min_knot: bool = check_if_knot_exists(surface_knots, knot_min);
    let has_max_knot: bool = check_if_knot_exists(surface_knots, knot_max);
    if has_min_knot && has_max_knot {
        let mults: Vec<i32> = surface_mults.iter().map(|m| m + degree_delta).collect();
        return Ok((surface_knots.clone(), one_based_from(&mults)?));
    }
    if has_min_knot || has_max_knot || knot_min == knot_max {
        let knot_to_insert: f64 = if has_min_knot { knot_max } else { knot_min };
        return add_a_knot(
            surface_knots,
            surface_mults,
            knot_to_insert,
            [degree_delta, final_degree],
        );
    }
    let (knots, mults) = add_a_knot(
        surface_knots,
        surface_mults,
        knot_min,
        [degree_delta, final_degree],
    )?;
    add_a_knot(&knots, &mults, knot_max, [0, final_degree])
}
