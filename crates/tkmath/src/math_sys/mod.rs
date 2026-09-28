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

//! Nonlinear equation systems using native rank-aware linear policies.
mod types;
pub use types::SystemResult;
pub mod linear;
mod newton;
pub use newton::{newton, newton_bounded, newton_uniform};

mod lm;
pub use lm::{LMConfig, levenberg_marquardt, levenberg_marquardt_bounded};

mod small;
pub use small::{
    CurveD2, CurveEvaluatorD2, NewtonBoundsN, NewtonOptions, NewtonResultN, SmallSystem, SurfaceD2,
    SurfaceEvaluatorD2, SymmetricFunction2D, solve_2d, solve_2d_symmetric, solve_3d, solve_4d,
    solve_curve_surface_extrema_3d, solve_surface_surface_extrema_4d,
};

pub mod utils;
