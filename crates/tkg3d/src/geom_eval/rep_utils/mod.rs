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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_RepUtils`: validation, cloning and evaluation through a descriptor.
//!
//! The `try_eval_*` functions return `Ok(None)` where the native ones return false
//! (no descriptor, derivative order above the limit, parameter outside the mapped
//! domain), so the owner falls back to its own evaluation.
mod curve;
mod surface;
pub use curve::{
    try_eval_curve_d0, try_eval_curve_d1, try_eval_curve_d2, try_eval_curve_d3, try_eval_curve_dn,
};
pub use surface::{
    try_eval_surface_d0, try_eval_surface_d1, try_eval_surface_d2, try_eval_surface_d3,
    try_eval_surface_dn,
};

use super::rep_curve_desc::{
    RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
};
use super::rep_surface_desc::{
    RepSurfaceDesc, RepSurfaceDescDerivBounded, RepSurfaceDescFull, RepSurfaceDescMapped,
};
use crate::geom::errors;
use crate::geom::{GeomCurve, GeomSurface};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_eval::RepCurveKind;

// The checks shared by ValidateCurveDesc and ValidateSurfaceDesc: a representation,
// a valid map for a mapped descriptor, and no descriptor chain back to the owner.
fn validate_parts(
    representation: Option<*const ()>,
    map_valid: bool,
    owner: *const (),
) -> Result<(), Failure> {
    let Some(address) = representation else {
        return Err(errors::program(
            "SetEvalRepresentation: null Representation",
        ));
    };
    if !map_valid {
        return Err(errors::program(
            "SetEvalRepresentation: invalid parameter map; expected scales with |Scale| > Precision::Confusion()",
        ));
    }
    if address == owner {
        return Err(errors::program(
            "SetEvalRepresentation: invalid descriptor chain; the representation is the owner itself",
        ));
    }
    Ok(())
}

/// Reject a descriptor with a null representation, an invalid map, or a
/// representation that is the owner itself (`ValidateCurveDesc`);
/// `validate_curve_desc(&desc, std::ptr::from_ref(self).cast())?`.
pub fn validate_curve_desc(
    desc: &Handle<dyn RepCurveDesc>,
    owner: *const (),
) -> Result<(), Failure> {
    let Some(desc) = desc.get() else {
        return Ok(());
    };
    let representation: Handle<dyn GeomCurve> = desc.representation();
    let address = representation
        .is_some()
        .then(|| representation.object_address());
    let map_valid: bool = match (desc.kind(), desc.mapping()) {
        (RepCurveKind::Mapped, Some((_, map))) => map.is_valid(),
        _ => true,
    };
    validate_parts(address, map_valid, owner)
}

/// Reject a descriptor with a null representation, an invalid map, or a
/// representation that is the owner itself (`ValidateSurfaceDesc`);
/// `validate_surface_desc(&desc, std::ptr::from_ref(self).cast())?`.
pub fn validate_surface_desc(
    desc: &Handle<dyn RepSurfaceDesc>,
    owner: *const (),
) -> Result<(), Failure> {
    let Some(desc) = desc.get() else {
        return Ok(());
    };
    let representation: Handle<dyn GeomSurface> = desc.representation();
    let address = representation
        .is_some()
        .then(|| representation.object_address());
    let map_valid: bool = match (desc.kind(), desc.mapping()) {
        (RepCurveKind::Mapped, Some((_, map))) => map.is_valid(),
        _ => true,
    };
    validate_parts(address, map_valid, owner)
}

/// Deep copy with a copied representation; null when there is nothing to copy
/// (`CloneCurveDesc`); `let copy = clone_curve_desc(&desc)?;`.
pub fn clone_curve_desc(
    desc: &Handle<dyn RepCurveDesc>,
) -> Result<Handle<dyn RepCurveDesc>, Failure> {
    let Some(source) = desc.get() else {
        return Ok(Handle::default());
    };
    let representation: Handle<dyn GeomCurve> = source.representation();
    let Some(curve) = representation.get() else {
        return Ok(Handle::default());
    };
    let copy: Handle<dyn GeomCurve> = curve.copy()?.cast();
    if copy.is_null() {
        return Ok(Handle::default());
    }
    Ok(
        match (source.kind(), source.max_deriv_order(), source.mapping()) {
            (RepCurveKind::DerivBounded, Some(order), _) => {
                Handle::new(RepCurveDescDerivBounded::new(copy, order)).cast()
            }
            (RepCurveKind::Mapped, Some(order), Some((domain, map))) => {
                Handle::new(RepCurveDescMapped::new(copy, order, domain, map)).cast()
            }
            _ => Handle::new(RepCurveDescFull::new(copy)).cast(),
        },
    )
}

/// Deep copy with a copied representation; null when there is nothing to copy
/// (`CloneSurfaceDesc`); `let copy = clone_surface_desc(&desc)?;`.
pub fn clone_surface_desc(
    desc: &Handle<dyn RepSurfaceDesc>,
) -> Result<Handle<dyn RepSurfaceDesc>, Failure> {
    let Some(source) = desc.get() else {
        return Ok(Handle::default());
    };
    let representation: Handle<dyn GeomSurface> = source.representation();
    let Some(surface) = representation.get() else {
        return Ok(Handle::default());
    };
    let copy: Handle<dyn GeomSurface> = surface.copy()?.cast();
    if copy.is_null() {
        return Ok(Handle::default());
    }
    Ok(
        match (source.kind(), source.max_deriv_order(), source.mapping()) {
            (RepCurveKind::DerivBounded, Some(order), _) => {
                Handle::new(RepSurfaceDescDerivBounded::new(copy, order)).cast()
            }
            (RepCurveKind::Mapped, Some(order), Some((domain, map))) => {
                Handle::new(RepSurfaceDescMapped::new(copy, order, domain, map)).cast()
            }
            _ => Handle::new(RepSurfaceDescFull::new(copy)).cast(),
        },
    )
}
