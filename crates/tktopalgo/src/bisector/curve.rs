// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_Curve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Bisector_Curve`: the abstract bisector curve, a `Geom2d_Curve` with a parameter
//! of a point, extensions and intervals of continuity.
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::Point2;

/// `Bisector_Curve`: the members every bisector curve adds to `Geom2d_Curve`
/// (docs/topalgo-design.md section 3). A trimmed curve whose basis is a bisector
/// gives it back with `trimmed.basis_curve().cast::<dyn BisectorCurve>()`.
/// ```
/// use rustycad_tktopalgo::bisector::{BisecAna, BisectorCurve};
/// use rustycad_tkernel::standard::Handle;
/// let bisector: Handle<dyn BisectorCurve> = Handle::new(BisecAna::new()).cast();
/// assert!(!bisector.is_extend_at_start() && bisector.nb_intervals() == 1);
/// ```
pub trait BisectorCurve: Geom2dCurve {
    /// The parameter of the point `p` of the bisector (`Parameter`); `bis.parameter(p)?`.
    fn parameter(&self, p: Point2) -> Result<f64, Failure>;
    /// True when the start of the curve is an extension (`IsExtendAtStart`).
    fn is_extend_at_start(&self) -> bool;
    /// True when the end of the curve is an extension (`IsExtendAtEnd`).
    fn is_extend_at_end(&self) -> bool;
    /// The number of intervals of continuity C1 (`NbIntervals`); `bis.nb_intervals()`.
    fn nb_intervals(&self) -> i32;
    /// The first parameter of the interval `index` (`IntervalFirst`); `bis.interval_first(1)?`.
    fn interval_first(&self, index: i32) -> Result<f64, Failure>;
    /// The last parameter of the interval `index` (`IntervalLast`); `bis.interval_last(1)?`.
    fn interval_last(&self, index: i32) -> Result<f64, Failure>;
}
impl_transient_interface!(dyn BisectorCurve, "Bisector_Curve", dyn Geom2dCurve);
