// Rust translation of ModelingData/TKG3d/Geom/Geom_OffsetSurface.cxx (SetBasisSurface) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The basis resolution of SetBasisSurface: copy, unwrapping of trimmed and offset
//! surfaces, and the C0 check through the iso curves.
use super::GeomOffsetSurface;
use crate::geom::errors;
use crate::geom::{
    GeomBSplineCurve, GeomCurve, GeomOffsetCurve, GeomRectangularTrimmedSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface, GeomTrimmedCurve,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;

/// The offset and continuity that SetBasisSurface writes into the object, kept even
/// when it fails part-way, as the native member leaves them.
pub(super) struct BasisResolution {
    pub(super) offset: f64,
    pub(super) continuity: Option<Shape>,
}

/// The basis to store and the unwrapped surface.
pub(super) type ResolvedBasis = (Handle<dyn GeomSurface>, Handle<dyn GeomSurface>);

/// SetBasisSurface without the object's lock: returns the basis to store (re-trimmed
/// when the input was trimmed) and the unwrapped surface.
pub(super) fn resolve_basis(
    surface: &Handle<dyn GeomSurface>,
    not_check_c0: bool,
    resolved: &mut BasisResolution,
) -> Result<ResolvedBasis, Failure> {
    let Some(source) = surface.get() else {
        return Err(errors::construction(
            "Geom_OffsetSurface: the basis surface is null; expected a surface",
        ));
    };
    let bounds = source.bounds();
    let mut checking: Handle<dyn GeomSurface> = source.copy()?.cast();
    let mut is_trimmed: bool = false;
    loop {
        let trimmed: Handle<GeomRectangularTrimmedSurface> = checking.cast();
        let offset: Handle<GeomOffsetSurface> = checking.cast();
        if trimmed.is_null() && offset.is_null() {
            break;
        }
        if let Some(trimmed) = trimmed.get() {
            checking = trimmed.basis_surface();
            is_trimmed = true;
        }
        let offset: Handle<GeomOffsetSurface> = checking.cast();
        if let Some(offset) = offset.get() {
            checking = offset.basis_surface();
            resolved.offset += offset.offset();
        }
    }
    let continuity: Shape = checking.continuity();
    resolved.continuity = Some(continuity);
    if !not_check_c0 && continuity == Shape::C0 {
        check_g1(&checking, bounds)?;
        resolved.continuity = Some(Shape::G1);
    }
    if !is_trimmed {
        return Ok((checking.clone(), checking));
    }
    let (u1, u2, v1, v2) = bounds;
    let trimmed = GeomRectangularTrimmedSurface::new(&checking, (u1, u2), (v1, v2), (true, true))?;
    Ok((Handle::new(trimmed).cast(), checking))
}

// The basis curve of a swept surface with its trimmed and offset wrappers removed,
// null for other surfaces.
fn swept_profile(surface: &Handle<dyn GeomSurface>) -> Handle<dyn GeomCurve> {
    let mut curve: Handle<dyn GeomCurve> =
        if let Some(rev) = surface.cast::<GeomSurfaceOfRevolution>().get() {
            rev.basis_curve()
        } else if let Some(ext) = surface.cast::<GeomSurfaceOfLinearExtrusion>().get() {
            ext.basis_curve()
        } else {
            return Handle::default();
        };
    loop {
        if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
            curve = trimmed.basis_curve();
        } else if let Some(offset) = curve.cast::<GeomOffsetCurve>().get() {
            curve = offset.basis_curve();
        } else {
            return curve;
        }
    }
}

// Whether an iso curve is G1: not a B-spline, or a B-spline that IsG1 on the range.
fn iso_is_g1(curve: &Handle<dyn GeomCurve>, first: f64, last: f64) -> Result<bool, Failure> {
    match curve.cast::<GeomBSplineCurve>().get() {
        Some(bspline) => bspline.is_g1(first, last, precision::angular()),
        None => Ok(true),
    }
}

// The C0 check: a C0 basis is accepted as G1 when the profile (or the middle U iso)
// and the middle V iso are G1; Standard_ConstructionError otherwise.
fn check_g1(
    checking: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<(), Failure> {
    let (uf, ul, vf, vl) = bounds;
    let (u_iso_par, v_iso_par) = ((uf + ul) / 2.0, (vf + vl) / 2.0);
    let profile: Handle<dyn GeomCurve> = swept_profile(checking);
    let curve1: Handle<dyn GeomCurve> = if profile.is_null() {
        checking.u_iso(u_iso_par)?
    } else {
        profile
    };
    let curve2: Handle<dyn GeomCurve> = checking.v_iso(v_iso_par)?;
    let is_ug1: bool = iso_is_g1(&curve1, vf, vl)?;
    let is_vg1: bool = iso_is_g1(&curve2, uf, ul)?;
    if is_ug1 && is_vg1 {
        return Ok(());
    }
    Err(errors::construction(&format!(
        "Offset with no C1 Surface: the iso curves at U = {u_iso_par} (G1: {is_ug1}) and V = {v_iso_par} (G1: {is_vg1}) are not both G1"
    )))
}
