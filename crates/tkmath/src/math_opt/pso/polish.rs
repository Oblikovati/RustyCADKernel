// Copyright (c) 2026 OPEN CASCADE SAS
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

//! MathOpt_PSO.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::{MultipleVarFunction, Vector};
use crate::math_utils::brent_along_coordinate;
/// Native two-pass coordinate polishing with shared budget; `polish_coordinate_wise(&mut f,&mut x,&mut value,&lower,&upper,1e-8,100,&mut count)`.
#[allow(clippy::too_many_arguments)]
pub fn polish_coordinate_wise(
    function: &mut impl MultipleVarFunction,
    position: &mut Vector<'_>,
    value: &mut f64,
    lower: &Vector<'_>,
    upper: &Vector<'_>,
    tolerance: f64,
    budget: usize,
    evaluations: &mut usize,
) {
    *evaluations = 0;
    let dimensions: usize = position.length() as usize;
    let passes: usize = if dimensions > 1 { 2 } else { 1 };
    let per_coordinate: u32 = (budget / (dimensions * passes)).max(1) as u32;
    for _ in 0..passes {
        for dimension in 0..dimensions {
            if *evaluations >= budget {
                return;
            }
            let limit: u32 = (per_coordinate as usize).min(budget - *evaluations) as u32;
            brent_along_coordinate(
                function,
                position,
                dimension,
                value,
                [
                    *lower.at(dimension).unwrap(),
                    *upper.at(dimension).unwrap(),
                    tolerance,
                ],
                limit,
                evaluations,
            );
        }
    }
}
