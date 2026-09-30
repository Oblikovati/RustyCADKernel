// Rust translation of FoundationClasses/TKMath/BSplCLib/BSplCLib.cxx (TangExtendToConstraint) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Extension of an n-dimensional B-spline to a point with Hermite continuity.
//!
//! The extension is a Hermite polynomial matching the curve derivatives at the
//! connection (scaled by C1Coefficient) and the constraint point; it is raised to
//! the curve degree, concatenated, and the connection knot multiplicity is reduced
//! while the removal stays within 1e-6.
use super::degree::increase_degree_flat;
use super::extrapolate::eval_extrapolated;
use super::knots::knot_sequence;
use super::remove::remove_knot_flat;
use super::views::one_based;
use crate::math::{Matrix, Vector};
use crate::plib::access::{array_cell, array_value, buffer_value, require_length};
use crate::plib::{coefficients_poles_flat, hermite_coefficients};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

const EXTEND: &str = "BSplCLib::TangExtendToConstraint";

/// Extends the curve (flat knots, `num_poles` rows of `dimension` values) after its
/// end (`after`) or before its start to `constraint_point` with `continuity` 0..=3
/// (`BSplCLib::TangExtendToConstraint`).
///
/// Returns the pole and flat-knot counts written to `poles_result` and `knots_result`;
/// `bsplclib::tang_extend_to_constraint(&flat, 1.0, 4, &p, 3, 3, &point, 1, true, &mut k, &mut p2)?`.
pub fn tang_extend_to_constraint(
    flat_knots: &Array1<'_, f64>,
    c1_coefficient: f64,
    num_poles: i32,
    poles: &[f64],
    dimension: i32,
    degree: i32,
    constraint_point: &Array1<'_, f64>,
    continuity: i32,
    after: bool,
    knots_result: &mut [f64],
    poles_result: &mut [f64],
) -> Result<(i32, i32), Failure> {
    let setup = Extension {
        c1_coefficient,
        dimension,
        degree,
        continuity,
        after,
        csize: continuity + 2,
    };
    let (eval_bs, l1) = setup.connection_derivatives(flat_knots, poles)?;
    let extra_poles: Array1<'static, f64> = setup.extension_poles(&eval_bs, constraint_point)?;
    let l2: f64 = setup.extension_tangent_norm(&extra_poles)?;
    let (new_p2, new_fk2) = setup.harmonized_extension(extra_poles)?;
    let concatenation =
        setup.concatenate(flat_knots, num_poles, poles, &new_p2, &new_fk2, [l1, l2])?;
    setup.reduce(concatenation, knots_result, poles_result)
}

struct Extension {
    c1_coefficient: f64,
    dimension: i32,
    degree: i32,
    continuity: i32,
    after: bool,
    csize: i32,
}

// Poles and flat knots of the C0 concatenation.
struct Concatenation {
    poles: Array1<'static, f64>,
    flats: Array1<'static, f64>,
    nb_poles: i32,
}

impl Extension {
    // Derivatives of the curve at the connection knot, and the tangent norm L1.
    fn connection_derivatives(
        &self,
        flat_knots: &Array1<'_, f64>,
        poles: &[f64],
    ) -> Result<(Vector<'static>, f64), Failure> {
        let tbord: f64 = if self.after {
            array_value(flat_knots, flat_knots.upper() - self.degree)?
        } else {
            array_value(flat_knots, flat_knots.lower() + self.degree)?
        };
        let request: i32 = self.continuity.max(1);
        let mut eval_bs: Vector<'static> = Vector::with_bounds(1, self.dimension * (request + 1))?;
        let mode: [i32; 2] = [self.degree, self.degree];
        let mut values: Vec<f64> = vec![0.0; (self.dimension * (request + 1)).max(0) as usize];
        eval_extrapolated(
            tbord,
            false,
            request,
            &mode,
            self.degree,
            flat_knots,
            self.dimension,
            poles,
            &mut values,
        )?;
        for (i, value) in values.iter().enumerate() {
            *eval_bs.change_value(i as i32 + 1)? = *value;
        }
        let l1: f64 = self.tangent_norm(&eval_bs)?;
        Ok((eval_bs, l1))
    }

    fn tangent_norm(&self, eval_bs: &Vector<'_>) -> Result<f64, Failure> {
        let mut tangent: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        for ipos in 1..=self.dimension {
            *tangent.change_value(ipos)? = *eval_bs.value(ipos + self.dimension)?;
        }
        tangent.norm()
    }

    // The Hermite constraints, their polynomial coefficients and Bezier poles.
    fn extension_poles(
        &self,
        eval_bs: &Vector<'_>,
        constraint_point: &Array1<'_, f64>,
    ) -> Result<Array1<'static, f64>, Failure> {
        let csize: i32 = self.csize;
        let mut mat_coefs: Matrix<'static> = Matrix::filled(1, csize, 1, csize, 0.0)
            .map_err(|error| Failure::out_of_range(&error.to_string()))?;
        let (first_order, last_order) = if self.after {
            (self.continuity, 0)
        } else {
            (0, self.continuity)
        };
        hermite_coefficients(0.0, 1.0, first_order, last_order, &mut mat_coefs)?;
        let constraints: Matrix<'static> = self.constraints(eval_bs, constraint_point)?;
        let mut extra_coeffs: Array1<'static, f64> = one_based(EXTEND, csize * self.dimension)?;
        for ii in 1..=csize {
            for jj in 1..=csize {
                for kk in 1..=self.dimension {
                    let term: f64 = *mat_coefs.value(ii, jj)? * *constraints.value(ii, kk)?;
                    *array_cell(&mut extra_coeffs, kk + (jj - 1) * self.dimension)? += term;
                }
            }
        }
        let mut extrap_poles: Array1<'static, f64> = one_based(EXTEND, csize * self.dimension)?;
        coefficients_poles_flat(self.dimension, &extra_coeffs, None, &mut extrap_poles, None)?;
        Ok(extrap_poles)
    }

    fn constraints(
        &self,
        eval_bs: &Vector<'_>,
        constraint_point: &Array1<'_, f64>,
    ) -> Result<Matrix<'static>, Failure> {
        let dim: i32 = self.dimension;
        let mut constraints: Matrix<'static> = Matrix::filled(1, self.csize, 1, dim, 0.0)
            .map_err(|error| Failure::out_of_range(&error.to_string()))?;
        let c1: f64 = self.c1_coefficient;
        let (c1_2, c1_3): (f64, f64) = (c1 * c1, c1 * c1 * c1);
        // Rows: value, C1*first, second*C1^2, third*C1^3 (shifted by one before).
        let shift: i32 = if self.after { 0 } else { 1 };
        for ipos in 1..=dim {
            let derivative = |order: i32| eval_bs.value(ipos + order * dim).copied();
            if !self.after {
                *constraints.change_value(1, ipos)? = array_value(constraint_point, ipos)?;
            }
            *constraints.change_value(1 + shift, ipos)? = derivative(0)?;
            if self.after || self.continuity >= 1 {
                *constraints.change_value(2 + shift, ipos)? = c1 * derivative(1)?;
            }
            if self.continuity >= 2 {
                *constraints.change_value(3 + shift, ipos)? = derivative(2)? * c1_2;
            }
            if self.continuity >= 3 {
                *constraints.change_value(4 + shift, ipos)? = derivative(3)? * c1_3;
            }
            if self.after {
                *constraints.change_value(self.continuity + 2, ipos)? =
                    array_value(constraint_point, ipos)?;
            }
        }
        Ok(constraints)
    }

    // L2: the tangent norm of the extension at the connection point.
    fn extension_tangent_norm(&self, extrap_poles: &Array1<'_, f64>) -> Result<f64, Failure> {
        let (knots, mults) = self.extension_knots()?;
        let mut fk2: Array1<'static, f64> = one_based(EXTEND, self.csize * 2)?;
        knot_sequence(&knots, &mults, &mut fk2, false)?;
        let parameter: f64 = if self.after { 0. } else { 1. };
        let mode: [i32; 2] = [self.degree, self.degree];
        let mut values: Vec<f64> = vec![0.0; 2 * self.dimension.max(0) as usize];
        eval_extrapolated(
            parameter,
            false,
            1,
            &mode,
            self.csize - 1,
            &fk2,
            self.dimension,
            extrap_poles.as_slice(),
            &mut values,
        )?;
        let mut tangent: Vector<'static> = Vector::with_bounds(1, self.dimension)?;
        for ipos in 1..=self.dimension {
            *tangent.change_value(ipos)? = values[(ipos + self.dimension - 1) as usize];
        }
        tangent.norm()
    }

    fn extension_knots(&self) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
        let mut knots: Array1<'static, f64> = one_based(EXTEND, 2)?;
        knots.as_mut_slice().copy_from_slice(&[0., 1.]);
        let mut mults: Array1<'static, i32> = one_based(EXTEND, 2)?;
        mults.init(self.csize);
        Ok((knots, mults))
    }

    // The extension raised to the curve degree, and its flat knots.
    fn harmonized_extension(
        &self,
        extrap_poles: Array1<'static, f64>,
    ) -> Result<(Array1<'static, f64>, Array1<'static, f64>), Failure> {
        let (knots, mults) = self.extension_knots()?;
        let mut new_p2: Array1<'static, f64> =
            one_based(EXTEND, (self.degree + 1) * self.dimension)?;
        let mut new_k2: Array1<'static, f64> = one_based(EXTEND, 2)?;
        let mut new_m2: Array1<'static, i32> = one_based(EXTEND, 2)?;
        if self.csize - 1 < self.degree {
            increase_degree_flat(
                self.csize - 1,
                self.degree,
                false,
                self.dimension,
                &extrap_poles,
                &knots,
                &mults,
                &mut new_p2,
                &mut new_k2,
                &mut new_m2,
            )?;
        } else {
            new_p2.assign(&extrap_poles);
            new_k2.assign(&knots);
            new_m2.assign(&mults);
        }
        let mut new_fk2: Array1<'static, f64> = one_based(EXTEND, (self.degree + 1) * 2)?;
        knot_sequence(&new_k2, &new_m2, &mut new_fk2, false)?;
        Ok((new_p2, new_fk2))
    }
}

impl Extension {
    fn concatenate(
        &self,
        flat_knots: &Array1<'_, f64>,
        num_poles: i32,
        poles: &[f64],
        new_p2: &Array1<'_, f64>,
        new_fk2: &Array1<'_, f64>,
        [l1, l2]: [f64; 2],
    ) -> Result<Concatenation, Failure> {
        let mut ratio: f64 = 1.;
        if l1 > precision::confusion() && l2 > precision::confusion() {
            ratio = l2 / l1;
        }
        // Explicit comparisons keep a NaN ratio, as natively.
        if ratio < 1.0e-5 || ratio > 1.0e5 {
            ratio = 1.;
        }
        let (nb_p1, nb_p2): (i32, i32) = (num_poles, self.degree + 1);
        let (nb_k1, nb_k2): (i32, i32) = (flat_knots.len() as i32, 2 * (self.degree + 1));
        let mut new_poles: Array1<'static, f64> =
            one_based(EXTEND, (nb_p1 + nb_p2 - 1) * self.dimension)?;
        self.concatenate_poles([nb_p1, nb_p2], poles, new_p2, &mut new_poles)?;
        let mut flats: Array1<'static, f64> = one_based(EXTEND, nb_k1 + nb_k2 - self.degree - 2)?;
        self.concatenate_knots([nb_k1, nb_k2], ratio, flat_knots, new_fk2, &mut flats)?;
        Ok(Concatenation {
            poles: new_poles,
            flats,
            nb_poles: nb_p1 + nb_p2 - 1,
        })
    }

    fn concatenate_poles(
        &self,
        [nb_p1, nb_p2]: [i32; 2],
        poles: &[f64],
        new_p2: &Array1<'_, f64>,
        new_poles: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let dim: i32 = self.dimension;
        for ii in 1..=nb_p1 + nb_p2 - 1 {
            for jj in 1..=dim {
                let ind_np: i32 = (ii - 1) * dim + jj;
                let value: f64 = if self.after {
                    if ii < nb_p1 {
                        buffer_value(EXTEND, poles, i64::from((ii - 1) * dim + jj - 1))?
                    } else {
                        array_value(new_p2, (ii - nb_p1) * dim + jj)?
                    }
                } else if ii < nb_p2 {
                    array_value(new_p2, (ii - 1) * dim + jj)?
                } else {
                    buffer_value(EXTEND, poles, i64::from((ii - nb_p2) * dim + jj - 1))?
                };
                *array_cell(new_poles, ind_np)? = value;
            }
        }
        Ok(())
    }

    fn concatenate_knots(
        &self,
        [nb_k1, nb_k2]: [i32; 2],
        ratio: f64,
        flat_knots: &Array1<'_, f64>,
        new_fk2: &Array1<'_, f64>,
        flats: &mut Array1<'_, f64>,
    ) -> Result<(), Failure> {
        let degree: i32 = self.degree;
        if self.after {
            // Do not touch the first B-spline.
            let delta: f64 = ratio * array_value(new_fk2, new_fk2.lower())?
                - array_value(flat_knots, flat_knots.upper())?;
            for ii in 1..nb_k1 {
                *array_cell(flats, ii)? = array_value(flat_knots, flat_knots.lower() + ii - 1)?;
            }
            for ii in 1..=nb_k2 - degree - 1 {
                let knot: f64 =
                    ratio * array_value(new_fk2, new_fk2.lower() + ii + degree)? - delta;
                *array_cell(flats, nb_k1 + ii - 1)? = knot;
            }
            return Ok(());
        }
        // Do not touch the second B-spline.
        let delta: f64 = ratio * array_value(new_fk2, new_fk2.upper())?
            - array_value(flat_knots, flat_knots.lower())?;
        for ii in 1..nb_k2 - degree {
            *array_cell(flats, ii)? =
                ratio * array_value(new_fk2, new_fk2.lower() + ii - 1)? - delta;
        }
        for ii in 2..=nb_k1 {
            *array_cell(flats, nb_k2 + ii - degree - 2)? =
                array_value(flat_knots, flat_knots.lower() + ii - 1)?;
        }
        Ok(())
    }

    // Reduces the multiplicity of the connection knot while removal stays within 1e-6.
    fn reduce(
        &self,
        concatenation: Concatenation,
        knots_result: &mut [f64],
        poles_result: &mut [f64],
    ) -> Result<(i32, i32), Failure> {
        let (new_knots, new_mults) = flat_to_knots(&concatenation.flats)?;
        let klength: i32 = new_knots.len() as i32;
        let index: i32 = if self.after { klength - 1 } else { 2 };
        let dim: i32 = self.dimension;
        let mut result_poles: Array1<'static, f64> =
            one_based(EXTEND, concatenation.nb_poles * dim)?;
        let mut result_knots: Array1<'static, f64> = one_based(EXTEND, klength)?;
        let mut result_mults: Array1<'static, i32> = one_based(EXTEND, klength)?;
        let mut m: i32 = self.degree;
        let mut ok: bool = true;
        while m > self.degree - self.continuity && ok {
            ok = remove_knot_flat(
                index,
                m - 1,
                self.degree,
                false,
                dim,
                &concatenation.poles,
                &new_knots,
                &new_mults,
                &mut result_poles,
                &mut result_knots,
                &mut result_mults,
                1.0e-6,
            )?;
            if ok {
                m -= 1;
            }
        }
        if m == self.degree {
            return write_result(
                &concatenation.poles,
                concatenation.nb_poles * dim,
                &new_knots,
                &new_mults,
                knots_result,
                poles_result,
            )
            .map(|nb_knots| (concatenation.nb_poles, nb_knots));
        }
        let nb_poles: i32 = concatenation.nb_poles - self.degree + m;
        let nb_knots: i32 = write_result(
            &result_poles,
            nb_poles * dim,
            &result_knots,
            &result_mults,
            knots_result,
            poles_result,
        )?;
        Ok((nb_poles, nb_knots))
    }
}

// Flat knots to distinct knots and multiplicities, with exact comparisons.
fn flat_to_knots(
    flats: &Array1<'_, f64>,
) -> Result<(Array1<'static, f64>, Array1<'static, i32>), Failure> {
    let count: i32 = flats.len() as i32;
    let mut klength: i32 = 1;
    for ii in 2..=count {
        if array_value(flats, ii)? != array_value(flats, ii - 1)? {
            klength += 1;
        }
    }
    let mut knots: Array1<'static, f64> = one_based(EXTEND, klength)?;
    let mut mults: Array1<'static, i32> = one_based(EXTEND, klength)?;
    mults.init(1);
    let mut jj: i32 = 1;
    *array_cell(&mut knots, jj)? = array_value(flats, 1)?;
    for ii in 2..=count {
        if array_value(flats, ii)? == array_value(flats, ii - 1)? {
            *array_cell(&mut mults, jj)? += 1;
        } else {
            jj += 1;
            *array_cell(&mut knots, jj)? = array_value(flats, ii)?;
        }
    }
    Ok((knots, mults))
}

// Copies PLength pole values and expands the knots into flat knots; returns their count.
fn write_result(
    poles: &Array1<'_, f64>,
    p_length: i32,
    knots: &Array1<'_, f64>,
    mults: &Array1<'_, i32>,
    knots_result: &mut [f64],
    poles_result: &mut [f64],
) -> Result<i32, Failure> {
    require_length(
        EXTEND,
        "PolesResult",
        poles_result.len(),
        p_length.max(0) as usize,
    )?;
    for jj in 0..p_length {
        poles_result[jj as usize] = array_value(poles, jj + 1)?;
    }
    let mut ideb: usize = 0;
    for jj in 0..knots.len() as i32 {
        let mult: i32 = array_value(mults, jj + 1)?;
        let knot: f64 = array_value(knots, jj + 1)?;
        require_length(
            EXTEND,
            "KnotsResult",
            knots_result.len(),
            ideb + mult.max(0) as usize,
        )?;
        for ii in 0..mult.max(0) as usize {
            knots_result[ideb + ii] = knot;
        }
        ideb += mult.max(0) as usize;
    }
    Ok(ideb as i32)
}
