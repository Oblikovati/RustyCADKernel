// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor_Surface.hxx/.cxx (surface data, Load, load, ShallowCopy) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The locked state of `GeomAdaptor_Surface`: the loaded surface, its bounds and
//! tolerances, its kind and the kind-specific evaluation data (`SurfaceDataVariant`).
//! The swept adaptors (`GeomAdaptor_SurfaceOfLinearExtrusion`,
//! `GeomAdaptor_SurfaceOfRevolution`) derive from `GeomAdaptor_Surface` natively and
//! write this protected state directly, hence the crate visibility.
use super::curve::GeomAdaptorCurve;
use super::surface::GeomAdaptorSurface;
use crate::adaptor3d::Adaptor3dCurve;
use crate::geom::errors::{construction, null_object};
use crate::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface, GeomToroidalSurface,
};
use crate::geom_eval::RepSurfaceDesc;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::bsplslib::Cache;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis1, Cone3, Cylinder3, Plane3, Sphere3, Torus3, Xyz};

/// Extrusion data (`ExtrusionData`): the basis curve adaptor and the direction.
pub(crate) struct ExtrusionData {
    pub(crate) basis_curve: Handle<dyn Adaptor3dCurve>,
    pub(crate) direction: Xyz,
    pub(crate) eval_rep: Handle<dyn RepSurfaceDesc>,
}
/// Revolution data (`RevolutionData`): the basis curve adaptor and the axis.
pub(crate) struct RevolutionData {
    pub(crate) basis_curve: Handle<dyn Adaptor3dCurve>,
    pub(crate) axis: Axis1,
    pub(crate) eval_rep: Handle<dyn RepSurfaceDesc>,
}
/// Offset data (`OffsetData`): adaptors on the basis and on the equivalent surface
/// (null when there is none), the offset surface for the osculating queries.
pub(crate) struct OffsetData {
    pub(crate) basis_adaptor: Handle<GeomAdaptorSurface>,
    pub(crate) equivalent_adaptor: Handle<GeomAdaptorSurface>,
    pub(crate) offset_surface: Handle<GeomOffsetSurface>,
    pub(crate) offset: f64,
    pub(crate) eval_rep: Handle<dyn RepSurfaceDesc>,
}
/// Bezier data (`BezierData`): the surface, its lazily built cache and the
/// evaluation representation it had when it was loaded.
pub(crate) struct BezierData {
    pub(crate) surface: Handle<GeomBezierSurface>,
    pub(crate) cache: Option<Cache>,
    pub(crate) eval_rep: Handle<dyn RepSurfaceDesc>,
}
/// B-spline data (`BSplineData`), as `BezierData`.
pub(crate) struct BSplineData {
    pub(crate) surface: Handle<GeomBSplineSurface>,
    pub(crate) cache: Option<Cache>,
    pub(crate) eval_rep: Handle<dyn RepSurfaceDesc>,
}
/// Kind-specific evaluation data (`SurfaceDataVariant`); `None` is `std::monostate`.
pub(crate) enum SurfaceData {
    None,
    Plane(Plane3),
    Cylinder(Cylinder3),
    Cone(Cone3),
    Sphere(Sphere3),
    Torus(Torus3),
    Extrusion(ExtrusionData),
    Revolution(RevolutionData),
    Offset(OffsetData),
    Bezier(BezierData),
    BSpline(BSplineData),
}
impl SurfaceData {
    // The native file-local hasEvalRep: a representation recorded at load time.
    pub(super) fn has_eval_rep(&self) -> bool {
        match self {
            SurfaceData::Bezier(data) => data.eval_rep.is_some(),
            SurfaceData::BSpline(data) => data.eval_rep.is_some(),
            SurfaceData::Extrusion(data) => data.eval_rep.is_some(),
            SurfaceData::Revolution(data) => data.eval_rep.is_some(),
            SurfaceData::Offset(data) => data.eval_rep.is_some(),
            _ => false,
        }
    }
}

/// The protected fields of one `GeomAdaptor_Surface`.
pub(crate) struct SurfaceState {
    pub(crate) surface: Handle<dyn GeomSurface>,
    pub(crate) u_first: f64,
    pub(crate) u_last: f64,
    pub(crate) v_first: f64,
    pub(crate) v_last: f64,
    pub(crate) tol_u: f64,
    pub(crate) tol_v: f64,
    pub(crate) surface_type: SurfaceType,
    pub(crate) data: SurfaceData,
}
impl Default for SurfaceState {
    fn default() -> Self {
        Self {
            surface: Handle::default(),
            u_first: 0.0,
            u_last: 0.0,
            v_first: 0.0,
            v_last: 0.0,
            tol_u: 0.0,
            tol_v: 0.0,
            surface_type: SurfaceType::OtherSurface,
            data: SurfaceData::None,
        }
    }
}
impl SurfaceState {
    /// The loaded surface; natively a null one is dereferenced; `state.surface()?`.
    pub(crate) fn surface(&self) -> Result<&dyn GeomSurface, Failure> {
        self.surface.get().ok_or_else(|| {
            null_object("GeomAdaptor_Surface: no surface is loaded; expected Load() first")
        })
    }
}

/// The bounds `(UFirst, ULast, VFirst, VLast)` and tolerances `(TolU, TolV)` of a load.
pub type LoadRange = ((f64, f64, f64, f64), (f64, f64));

/// The public `Load(S)`: the surface's own bounds, zero tolerances;
/// Standard_NullObject for a null surface; `load_whole(&mut state, &surface)?`.
pub(super) fn load_whole(
    state: &mut SurfaceState,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(), Failure> {
    let Some(source) = surface.get() else {
        return Err(null_object(
            "GeomAdaptor_Surface::Load: null surface; expected a surface",
        ));
    };
    load(state, surface, (source.bounds(), (0.0, 0.0)))
}

/// The public `Load(S, U1, U2, V1, V2, TolU, TolV)`: Standard_NullObject for a null
/// surface, Standard_ConstructionError when `U1 > U2` or `V1 > V2`;
/// `load_range(&mut state, &surface, ((0.0, 1.0, 0.0, 1.0), (0.0, 0.0)))?`.
pub(super) fn load_range(
    state: &mut SurfaceState,
    surface: &Handle<dyn GeomSurface>,
    range: LoadRange,
) -> Result<(), Failure> {
    if surface.is_null() {
        return Err(null_object(
            "GeomAdaptor_Surface::Load: null surface; expected a surface",
        ));
    }
    let (u1, u2, v1, v2) = range.0;
    if u1 > u2 || v1 > v2 {
        return Err(construction(&format!(
            "GeomAdaptor_Surface::Load: bounds [{u1}, {u2}] x [{v1}, {v2}]; \
             expected UFirst <= ULast and VFirst <= VLast"
        )));
    }
    load(state, surface, range)
}

// The private `load`: bounds and tolerances always change; a new surface is
// classified, while the same surface keeps its data (and caches) untouched.
fn load(
    state: &mut SurfaceState,
    surface: &Handle<dyn GeomSurface>,
    range: LoadRange,
) -> Result<(), Failure> {
    (
        (state.u_first, state.u_last, state.v_first, state.v_last),
        (state.tol_u, state.tol_v),
    ) = range;
    if state.surface == *surface {
        return Ok(());
    }
    state.surface = surface.clone();
    state.data = SurfaceData::None;
    classify(state, surface)
}

fn is_class<T: RuntimeTyped>(surface: &Handle<dyn GeomSurface>) -> bool {
    surface.is_instance(T::static_type())
}

// Kind and data from the exact dynamic type, as the native `DynamicType() ==` chain.
// A trimmed surface reloads its basis through the public Load on the same bounds,
// which resets the tolerances to 0 (the defaults of that call).
fn classify(state: &mut SurfaceState, surface: &Handle<dyn GeomSurface>) -> Result<(), Failure> {
    if is_class::<GeomRectangularTrimmedSurface>(surface) {
        let basis = surface
            .cast::<GeomRectangularTrimmedSurface>()
            .basis_surface();
        let bounds = (state.u_first, state.u_last, state.v_first, state.v_last);
        return load_range(state, &basis, (bounds, (0.0, 0.0)));
    }
    let (kind, data): (SurfaceType, SurfaceData) = match elementary_data(surface)? {
        Some(elementary) => elementary,
        None => swept_or_spline_data(state, surface)?,
    };
    state.surface_type = kind;
    state.data = data;
    Ok(())
}

fn elementary_data(
    surface: &Handle<dyn GeomSurface>,
) -> Result<Option<(SurfaceType, SurfaceData)>, Failure> {
    Ok(Some(if is_class::<GeomPlane>(surface) {
        (
            SurfaceType::Plane,
            SurfaceData::Plane(surface.cast::<GeomPlane>().pln()),
        )
    } else if is_class::<GeomCylindricalSurface>(surface) {
        let cylinder: Cylinder3 = surface.cast::<GeomCylindricalSurface>().cylinder()?;
        (SurfaceType::Cylinder, SurfaceData::Cylinder(cylinder))
    } else if is_class::<GeomConicalSurface>(surface) {
        let cone: Cone3 = surface.cast::<GeomConicalSurface>().cone()?;
        (SurfaceType::Cone, SurfaceData::Cone(cone))
    } else if is_class::<GeomSphericalSurface>(surface) {
        let sphere: Sphere3 = surface.cast::<GeomSphericalSurface>().sphere()?;
        (SurfaceType::Sphere, SurfaceData::Sphere(sphere))
    } else if is_class::<GeomToroidalSurface>(surface) {
        let torus: Torus3 = surface.cast::<GeomToroidalSurface>().torus()?;
        (SurfaceType::Torus, SurfaceData::Torus(torus))
    } else {
        return Ok(None);
    }))
}

fn swept_or_spline_data(
    state: &SurfaceState,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(SurfaceType, SurfaceData), Failure> {
    Ok(if is_class::<GeomSurfaceOfRevolution>(surface) {
        (
            SurfaceType::SurfaceOfRevolution,
            revolution_data(state, surface)?,
        )
    } else if is_class::<GeomSurfaceOfLinearExtrusion>(surface) {
        (
            SurfaceType::SurfaceOfExtrusion,
            extrusion_data(state, surface)?,
        )
    } else if is_class::<GeomBezierSurface>(surface) {
        (SurfaceType::BezierSurface, bezier_data(surface))
    } else if is_class::<GeomBSplineSurface>(surface) {
        (SurfaceType::BSplineSurface, bspline_data(surface))
    } else if is_class::<GeomOffsetSurface>(surface) {
        (SurfaceType::OffsetSurface, offset_data(state, surface)?)
    } else {
        (SurfaceType::OtherSurface, SurfaceData::None)
    })
}

// The basis curve adaptor spans the V range of the adaptor.
fn revolution_data(
    state: &SurfaceState,
    surface: &Handle<dyn GeomSurface>,
) -> Result<SurfaceData, Failure> {
    let revolution: Handle<GeomSurfaceOfRevolution> = surface.cast();
    let curve =
        GeomAdaptorCurve::with_range(&revolution.basis_curve(), state.v_first, state.v_last)?;
    Ok(SurfaceData::Revolution(RevolutionData {
        basis_curve: Handle::new(curve).cast(),
        axis: revolution.axis(),
        eval_rep: revolution.eval_representation(),
    }))
}

// The basis curve adaptor spans the U range of the adaptor.
fn extrusion_data(
    state: &SurfaceState,
    surface: &Handle<dyn GeomSurface>,
) -> Result<SurfaceData, Failure> {
    let extrusion: Handle<GeomSurfaceOfLinearExtrusion> = surface.cast();
    let curve =
        GeomAdaptorCurve::with_range(&extrusion.basis_curve(), state.u_first, state.u_last)?;
    Ok(SurfaceData::Extrusion(ExtrusionData {
        basis_curve: Handle::new(curve).cast(),
        direction: extrusion.direction().xyz(),
        eval_rep: extrusion.eval_representation(),
    }))
}

fn bezier_data(surface: &Handle<dyn GeomSurface>) -> SurfaceData {
    let bezier: Handle<GeomBezierSurface> = surface.cast();
    let eval_rep = bezier.eval_representation();
    SurfaceData::Bezier(BezierData {
        surface: bezier,
        cache: None,
        eval_rep,
    })
}

fn bspline_data(surface: &Handle<dyn GeomSurface>) -> SurfaceData {
    let bspline: Handle<GeomBSplineSurface> = surface.cast();
    let eval_rep = bspline.eval_representation();
    SurfaceData::BSpline(BSplineData {
        surface: bspline,
        cache: None,
        eval_rep,
    })
}

// Adaptors on the basis and on the equivalent canonical surface, if any, both over
// the adaptor bounds and tolerances.
fn offset_data(
    state: &SurfaceState,
    surface: &Handle<dyn GeomSurface>,
) -> Result<SurfaceData, Failure> {
    let offset_surface: Handle<GeomOffsetSurface> = surface.cast();
    let range: LoadRange = (
        (state.u_first, state.u_last, state.v_first, state.v_last),
        (state.tol_u, state.tol_v),
    );
    let basis = GeomAdaptorSurface::with_range(&offset_surface.basis_surface(), range)?;
    let equivalent: Handle<dyn GeomSurface> = offset_surface.surface()?;
    let equivalent_adaptor: Handle<GeomAdaptorSurface> = match equivalent.is_null() {
        true => Handle::default(),
        false => Handle::new(GeomAdaptorSurface::with_range(&equivalent, range)?),
    };
    Ok(SurfaceData::Offset(OffsetData {
        basis_adaptor: Handle::new(basis),
        equivalent_adaptor,
        offset: offset_surface.offset(),
        eval_rep: offset_surface.eval_representation(),
        offset_surface,
    }))
}

/// `ShallowCopy` of the whole state: the surface is shared, the data shallow-copied;
/// `shallow_state(&state)?`.
pub(crate) fn shallow_state(state: &SurfaceState) -> Result<SurfaceState, Failure> {
    Ok(SurfaceState {
        surface: state.surface.clone(),
        u_first: state.u_first,
        u_last: state.u_last,
        v_first: state.v_first,
        v_last: state.v_last,
        tol_u: state.tol_u,
        tol_v: state.tol_v,
        surface_type: state.surface_type,
        data: shallow_data(&state.data)?,
    })
}

/// `ShallowCopy` of the data: nested adaptors are shallow-copied, the Geom objects
/// shared, and the caches left to be rebuilt; `shallow_data(&state.data)?`.
pub(crate) fn shallow_data(data: &SurfaceData) -> Result<SurfaceData, Failure> {
    Ok(match data {
        SurfaceData::Extrusion(extrusion) => SurfaceData::Extrusion(ExtrusionData {
            basis_curve: extrusion.basis_curve.shallow_copy()?,
            direction: extrusion.direction,
            eval_rep: extrusion.eval_rep.clone(),
        }),
        SurfaceData::Revolution(revolution) => SurfaceData::Revolution(RevolutionData {
            basis_curve: revolution.basis_curve.shallow_copy()?,
            axis: revolution.axis,
            eval_rep: revolution.eval_rep.clone(),
        }),
        SurfaceData::Offset(offset) => SurfaceData::Offset(shallow_offset(offset)?),
        SurfaceData::BSpline(bspline) => SurfaceData::BSpline(BSplineData {
            surface: bspline.surface.clone(),
            cache: None,
            eval_rep: bspline.eval_rep.clone(),
        }),
        SurfaceData::Bezier(bezier) => SurfaceData::Bezier(BezierData {
            surface: bezier.surface.clone(),
            cache: None,
            eval_rep: bezier.eval_rep.clone(),
        }),
        other => elementary_copy(other),
    })
}

fn shallow_offset(offset: &OffsetData) -> Result<OffsetData, Failure> {
    let equivalent_adaptor: Handle<GeomAdaptorSurface> = match offset.equivalent_adaptor.get() {
        Some(equivalent) => Handle::new(equivalent.shallow_adaptor()?),
        None => Handle::default(),
    };
    Ok(OffsetData {
        basis_adaptor: Handle::new(offset.basis_adaptor.shallow_adaptor()?),
        equivalent_adaptor,
        offset_surface: offset.offset_surface.clone(),
        offset: offset.offset,
        eval_rep: offset.eval_rep.clone(),
    })
}

// The value types, copied directly ("Elementary surface types ... direct copy").
fn elementary_copy(data: &SurfaceData) -> SurfaceData {
    match data {
        SurfaceData::Plane(plane) => SurfaceData::Plane(*plane),
        SurfaceData::Cylinder(cylinder) => SurfaceData::Cylinder(*cylinder),
        SurfaceData::Cone(cone) => SurfaceData::Cone(*cone),
        SurfaceData::Sphere(sphere) => SurfaceData::Sphere(*sphere),
        SurfaceData::Torus(torus) => SurfaceData::Torus(*torus),
        _ => SurfaceData::None,
    }
}
