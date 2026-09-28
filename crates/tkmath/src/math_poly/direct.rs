// Copyright (c) 2025 OPEN CASCADE SAS
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

//! `MathPoly_Quadratic.hxx`, `MathPoly_Cubic.hxx` and `MathPoly_Quartic.hxx`: thin wrappers
//! that pass ascending coefficients to `Utils::SolveDirect`.
use super::PolyResult;
use super::utils::solve_direct;
/// Solves a linear equation with native degenerate statuses; `linear(2.0,-4.0)`.
pub fn linear(a: f64, b: f64) -> PolyResult {
    solve_direct(&[b, a], 1)
}
/// Uses the compensated native discriminant; `quadratic(1.0,-5.0,6.0)`.
pub fn quadratic(a: f64, b: f64, c: f64) -> PolyResult {
    solve_direct(&[c, b, a], 2)
}
/// Isolates real roots recursively with x87 precision; `cubic(1.0,-6.0,11.0,-6.0)`.
pub fn cubic(a: f64, b: f64, c: f64, d: f64) -> PolyResult {
    solve_direct(&[d, c, b, a], 3)
}
/// Preserves native real-root multiplicities; `quartic(1.0,-10.0,35.0,-50.0,24.0)`.
pub fn quartic(a: f64, b: f64, c: f64, d: f64, e: f64) -> PolyResult {
    solve_direct(&[e, d, c, b, a], 4)
}
