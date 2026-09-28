// Created on: 1998-11-06
// Created by: Igor FEOKTISTOV
// Copyright (c) 1998-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/FEmTool/FEmTool_LinearFlexion.cxx (constructor statics)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reference matrices of the linear criteria: the Gauss integrals over [-1, 1]
//! of the products of basis derivatives, cached process-wide as natively.
use super::elements_of_ref_matrix::ElementsOfRefMatrix;
use super::linear_criterion::LinearKind;
use crate::native_failure::failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::{GaussSetIntegration, IntegerVector, Matrix, Vector, gauss_points_max};
use rustycad_tkmath::plib::HermitJacobi;
use std::sync::PoisonError;

/// The degree the reference integrals are computed for (`WDeg`).
const REFERENCE_DEGREE: i32 = 14;
/// The number of upper-triangle products for [`REFERENCE_DEGREE`].
const REFERENCE_SIZE: usize = ((REFERENCE_DEGREE + 2) * (REFERENCE_DEGREE + 1) / 2) as usize;

/// The function-local statics of each native criterion constructor: the constraint
/// order of the last computation (`Order`, -333 initially) and the integrals
/// (`MatrixElemts`). They live for the process, so a later criterion reuses the
/// integrals of an earlier one with the same order, whatever its WorkDegree.
pub struct ReferenceCache {
    order: i32,
    elements: [f64; REFERENCE_SIZE],
}

impl ReferenceCache {
    /// The initial statics; `ReferenceCache::new()`.
    pub const fn new() -> Self {
        Self {
            order: -333,
            elements: [0.; REFERENCE_SIZE],
        }
    }
}

impl Default for ReferenceCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Fills `ref_matrix` (0..=WorkDegree square) from the cached integrals of kind `K`,
/// recomputing them when `order` differs from the cached one.
pub(super) fn reference_matrix<K: LinearKind>(
    order: i32,
    constraint_order: Shape,
    work_degree: i32,
    ref_matrix: &mut Matrix<'_>,
) -> Result<(), Failure> {
    let mut cache = K::cache().lock().unwrap_or_else(PoisonError::into_inner);
    if order != cache.order {
        if work_degree > REFERENCE_DEGREE {
            return Err(failure(
                "Standard_ConstructionError",
                &format!(
                    "{}: Degree too high ({work_degree}; expected <= {REFERENCE_DEGREE})",
                    K::NAME
                ),
            ));
        }
        // The native code stores the order before integrating.
        cache.order = order;
        cache.elements = integrate::<K>(constraint_order)?;
    }
    fill(&cache.elements, work_degree, ref_matrix)
}

// The Gauss integrals of the basis-derivative products on [-1, 1].
fn integrate<K: LinearKind>(constraint_order: Shape) -> Result<[f64; REFERENCE_SIZE], Failure> {
    let base: HermitJacobi = HermitJacobi::new(REFERENCE_DEGREE, constraint_order)?;
    let mut elements: ElementsOfRefMatrix = ElementsOfRefMatrix::new(&base, K::DER_ORDER)?;
    let max_degree: i32 = REFERENCE_DEGREE + 1;
    let order: IntegerVector<'static> =
        IntegerVector::filled(1, 1, (4 * (max_degree / 2 + 1)).min(gauss_points_max()))?;
    let (lower, upper) = (Vector::filled(1, 1, -1.)?, Vector::filled(1, 1, 1.)?);
    let integral: GaussSetIntegration =
        GaussSetIntegration::new(&mut elements, &lower, &upper, &order)?;
    let values: &Vector<'static> = integral.value()?;
    let mut result: [f64; REFERENCE_SIZE] = [0.; REFERENCE_SIZE];
    for (slot, index) in result.iter_mut().zip(values.lower()..) {
        *slot = values[index];
    }
    Ok(result)
}

// RefMatrix(i, j) = MatrixElemts(ii + j - i), row i starting at the WDeg-based offset ii;
// a WorkDegree above WDeg reads past the integrals (Standard_OutOfRange natively too).
fn fill(
    elements: &[f64; REFERENCE_SIZE],
    work_degree: i32,
    ref_matrix: &mut Matrix<'_>,
) -> Result<(), Failure> {
    let element = |index: i32| -> Result<f64, Failure> {
        usize::try_from(index)
            .ok()
            .and_then(|slot| elements.get(slot).copied())
            .ok_or_else(|| {
                Failure::out_of_range(&format!(
                    "math_Vector::Value: index {index}; expected 0..={}",
                    REFERENCE_SIZE - 1
                ))
            })
    };
    let mut ii: i32 = 0;
    for i in 0..=work_degree {
        *ref_matrix.change_value(i, i)? = element(ii)?;
        for (j, jj) in (i + 1..=work_degree).zip(ii + 1..) {
            let value: f64 = element(jj)?;
            *ref_matrix.change_value(i, j)? = value;
            *ref_matrix.change_value(j, i)? = value;
        }
        ii += REFERENCE_DEGREE + 1 - i;
    }
    Ok(())
}
