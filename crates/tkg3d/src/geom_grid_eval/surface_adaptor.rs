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
// Rust translation of GeomGridEval_Surface::initialization(const Adaptor3d_Surface&) and the anonymous namespace of ModelingData/TKG3d/GeomGridEval/GeomGridEval_Surface.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! How `GeomGridEval_Surface` turns an adaptor into a Geom surface: the transformed
//! adaptor gives its original surface and transformation; the revolution and
//! extrusion adaptors try an elementary surface, their stored surface, then a rebuilt
//! swept surface; a plain `GeomAdaptor_Surface` its surface or an elementary one; any
//! other adaptor an elementary surface; what remains goes to the fallback.
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomPlane, GeomSphericalSurface,
    GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use crate::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorSurfaceOfLinearExtrusion,
    GeomAdaptorSurfaceOfRevolution, GeomAdaptorTransformedSurface,
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Transform3, TransformForm};

/// The outcome of the adaptor initialization.
pub(super) enum AdaptorResolution<'a> {
    /// Initialize from this Geom surface (`initialization(occ::handle<Geom_Surface>)`),
    /// with the non-identity transformation of a transformed adaptor.
    Geometry(Handle<dyn GeomSurface>, Option<Transform3>),
    /// No Geom surface: the adaptor fallback with the adaptor's `GetType`.
    Fallback(SurfaceType, &'a dyn Adaptor3dSurface),
}

/// `initialization(const Adaptor3d_Surface&)` up to the evaluator choice.
pub(super) fn resolve_adaptor(
    surface: &dyn Adaptor3dSurface,
) -> Result<AdaptorResolution<'_>, Failure> {
    if let Some(transformed) = GeomAdaptorTransformedSurface::project_from(surface) {
        let trsf: Transform3 = transformed.trsf();
        let kept: Option<Transform3> = (trsf.form() != TransformForm::Identity).then_some(trsf);
        let original = transformed.geom_surface_original();
        return Ok(AdaptorResolution::Geometry(original, kept));
    }
    if let Some(revolution) = GeomAdaptorSurfaceOfRevolution::project_from(surface) {
        let stored = revolution.surface();
        return swept_resolution(surface, stored, || revolution_surface(revolution));
    }
    if let Some(extrusion) = GeomAdaptorSurfaceOfLinearExtrusion::project_from(surface) {
        let stored = extrusion.surface();
        return swept_resolution(surface, stored, || extrusion_surface(extrusion));
    }
    if let Some(geom_adaptor) = GeomAdaptorSurface::project_from(surface) {
        let mut geom = geom_adaptor.surface();
        if geom.is_null() {
            geom = elementary_surface(surface)?;
        }
        return resolved_or_fallback(surface, geom);
    }
    resolved_or_fallback(surface, elementary_surface(surface)?)
}

// The revolution and extrusion order: elementary surface, stored surface, rebuilt
// swept surface, then the fallback.
fn swept_resolution<'a>(
    surface: &'a dyn Adaptor3dSurface,
    stored: Handle<dyn GeomSurface>,
    rebuild: impl FnOnce() -> Result<Handle<dyn GeomSurface>, Failure>,
) -> Result<AdaptorResolution<'a>, Failure> {
    let elementary = elementary_surface(surface)?;
    if !elementary.is_null() {
        return Ok(AdaptorResolution::Geometry(elementary, None));
    }
    if !stored.is_null() {
        return Ok(AdaptorResolution::Geometry(stored, None));
    }
    resolved_or_fallback(surface, rebuild()?)
}

// The Geom surface when not null, the fallback with `GetType` otherwise.
fn resolved_or_fallback(
    surface: &dyn Adaptor3dSurface,
    geom: Handle<dyn GeomSurface>,
) -> Result<AdaptorResolution<'_>, Failure> {
    if !geom.is_null() {
        return Ok(AdaptorResolution::Geometry(geom, None));
    }
    Ok(AdaptorResolution::Fallback(surface.get_type()?, surface))
}

/// `CreateGeomSurfaceFromAdaptor`: a new elementary Geom surface from the adaptor's gp
/// surface, null for any other type.
fn elementary_surface(surface: &dyn Adaptor3dSurface) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match surface.get_type()? {
        SurfaceType::Plane => Handle::new(GeomPlane::from_pln(surface.plane()?)).cast(),
        SurfaceType::Cylinder => {
            Handle::new(GeomCylindricalSurface::from_cylinder(surface.cylinder()?)).cast()
        }
        SurfaceType::Cone => Handle::new(GeomConicalSurface::from_cone(surface.cone()?)).cast(),
        SurfaceType::Sphere => {
            Handle::new(GeomSphericalSurface::from_sphere(surface.sphere()?)).cast()
        }
        SurfaceType::Torus => Handle::new(GeomToroidalSurface::from_torus(surface.torus()?)).cast(),
        _ => Handle::default(),
    })
}

/// `ExtractGeomCurve`: the curve of a `GeomAdaptor_Curve`, null for any other adaptor.
fn extract_geom_curve(curve: &Handle<dyn Adaptor3dCurve>) -> Handle<dyn GeomCurve> {
    match curve.cast::<GeomAdaptorCurve>().get() {
        Some(geom_adaptor) => geom_adaptor.curve(),
        None => Handle::default(),
    }
}

/// `CreateRevolutionSurface`: a new surface of revolution on the adaptor's basis curve
/// and axis, null without a Geom basis curve.
fn revolution_surface(
    adaptor: &GeomAdaptorSurfaceOfRevolution,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let curve: Handle<dyn GeomCurve> = extract_geom_curve(&adaptor.basis_curve()?);
    if curve.is_null() {
        return Ok(Handle::default());
    }
    let axis = adaptor.axe_of_revolution()?;
    Ok(Handle::new(GeomSurfaceOfRevolution::new(&curve, axis)?).cast())
}

/// `CreateExtrusionSurface`: a new extrusion on the adaptor's basis curve and
/// direction, null without a Geom basis curve.
fn extrusion_surface(
    adaptor: &GeomAdaptorSurfaceOfLinearExtrusion,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let curve: Handle<dyn GeomCurve> = extract_geom_curve(&adaptor.basis_curve()?);
    if curve.is_null() {
        return Ok(Handle::default());
    }
    let direction = adaptor.direction()?;
    Ok(Handle::new(GeomSurfaceOfLinearExtrusion::new(&curve, direction)?).cast())
}
