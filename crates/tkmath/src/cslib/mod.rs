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

//! CSLib surface-normal computation and 2D point-in-polygon classification.
//!
//! The package-level static functions of OCCT's `CSLib` class are free functions
//! here; C++ output references become the returned [`NormalResult`] and
//! [`SingularNormal`] values, and C++ exceptions become [`Failure`] errors.
//!
//! [`Failure`]: rustycad_tkernel::standard::Failure
mod binomial;
mod class2d;
mod derivatives;
mod normal;
mod normal_poly;
mod precision;
mod singular;
mod status;

pub use class2d::{Class2d, Class2dResult};
pub use derivatives::{dn_normal, dnnuv, dnnuv_osculating};
pub use normal::{normal, normal_from_second_derivatives, normal_with_magnitude};
pub use normal_poly::NormalPolyDef;
pub use singular::{ParametricPosition, SingularNormal, normal_at_singularity};
pub use status::{DerivativeStatus, NormalResult, NormalStatus};
