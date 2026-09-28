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
// Rust translation of Perform of ModelingData/TKGeomBase/ExtremaPC/
// ExtremaPC_GridEvaluator.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ExtremaPC_GridEvaluator::Perform`: unlike the 2D evaluator, a failing curve
//! evaluation during the classification escapes it (only the stationarity signs are
//! caught), so the Rust member returns `Result`.
use super::{Domain1D, ExtremaResult, GridEvaluator, SearchMode};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::Point3;

impl GridEvaluator {
    /// The extrema of `point` against `curve` over `domain` with the cached partition
    /// (`Perform`); InvalidInput when the partition does not cover the domain;
    /// `evaluator.perform(&adaptor, &point, &domain, 1e-9, SearchMode::MinMax)?`.
    pub fn perform(
        &mut self,
        curve: &dyn Adaptor3dCurve,
        point: &Point3,
        domain: &Domain1D,
        tol: f64,
        mode: SearchMode,
    ) -> Result<&ExtremaResult, Failure> {
        self.perform_core(curve, point, domain, (tol, mode))?;
        Ok(self.result())
    }
}
