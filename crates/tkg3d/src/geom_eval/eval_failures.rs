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
// Rust translation of the Standard_NotImplemented members of ModelingData/TKG3d/GeomEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The failure every GeomEval curve and surface raises from the members that would
//! change or reparameterize it (`Reverse`, `UReverse`, `ReversedParameter`,
//! `Transform`, `UIso`...).
use crate::geom::errors;
use rustycad_tkernel::standard::Failure;

/// Standard_NotImplemented of a refused member, named as natively (`Class::Member`),
/// with the offending arguments in `detail`;
/// `unsupported("GeomEval_SineWaveCurve::ReversedParameter", "(U = 0.5)")`.
#[allow(dead_code)] // Used by every GeomEval class; the classes land one by one.
pub(crate) fn unsupported(member: &str, detail: &str) -> Failure {
    errors::not_implemented(&format!(
        "{member}{detail}: GeomEval geometry keeps its defining parameterization; expected no reversal, transformation or iso extraction"
    ))
}
