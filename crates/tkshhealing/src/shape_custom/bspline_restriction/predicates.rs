//! The static predicates of ShapeCustom_BSplineRestriction.cxx: which curves and surfaces
//! need a conversion, and the continuity/integer mappings of the approximation loops.
use crate::shape_custom::ShapeCustomRestrictionParameters;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomConicalSurface,
    GeomCurve, GeomCylindricalSurface, GeomOffsetCurve, GeomOffsetSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, GeomSweptSurface, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The limits the predicates compare with.
#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub degree: i32,
    pub segments: i32,
    pub rational: bool,
}

/// `IntegerToGeomAbsShape(i)`: 0 C0, 1 C1, 2 C2, 3 C3, any other CN.
pub(super) fn integer_to_continuity(i: i32) -> Continuity {
    match i {
        0 => Continuity::C0,
        1 => Continuity::C1,
        2 => Continuity::C2,
        3 => Continuity::C3,
        _ => Continuity::CN,
    }
}

/// `ContToInteger(Cont)`: C0/G1 0, C1/G2 1, C2 2, C3 3, CN 4.
pub(super) fn continuity_to_integer(continuity: Continuity) -> i32 {
    match continuity {
        Continuity::C0 | Continuity::G1 => 0,
        Continuity::C1 | Continuity::G2 => 1,
        Continuity::C2 => 2,
        Continuity::C3 => 3,
        Continuity::CN => 4,
    }
}

/// `IsConvertCurve3d`: a null curve never; any curve when ConvertCurve3d; trimmed and
/// offset curves by their basis (offsets always when ConvertOffsetCurv3d); B-splines above
/// the degree or with at least `segments` spans, Beziers above the degree, and rational
/// ones when `rational`.
pub(super) fn is_convert_curve_3d(
    curve: &Handle<dyn GeomCurve>,
    limits: Limits,
    p: &ShapeCustomRestrictionParameters,
) -> bool {
    if curve.is_null() {
        return false;
    }
    if p.convert_curve3d() {
        return true;
    }
    if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
        return is_convert_curve_3d(&trimmed.basis_curve(), limits, p);
    }
    if let Some(offset) = curve.cast::<GeomOffsetCurve>().get() {
        return p.convert_offset_curv3d() || is_convert_curve_3d(&offset.basis_curve(), limits, p);
    }
    if let Some(spline) = curve.cast::<GeomBSplineCurve>().get() {
        // Native `(NbKnots() - 1) >= NbSeg`: at least NbSeg spans.
        if spline.degree() > limits.degree || spline.nb_knots() > limits.segments {
            return true;
        }
        return limits.rational && spline.is_rational();
    }
    if let Some(bezier) = curve.cast::<GeomBezierCurve>().get() {
        return bezier.degree() > limits.degree || limits.rational && bezier.is_rational();
    }
    false
}

/// `IsConvertCurve2d`: as the 3D predicate with the 2D modes; a B-spline needs more than
/// `segments` spans here (the native 3D test uses at least).
pub(super) fn is_convert_curve_2d(
    curve: &Handle<dyn Geom2dCurve>,
    limits: Limits,
    p: &ShapeCustomRestrictionParameters,
) -> bool {
    if curve.is_null() {
        return false;
    }
    if p.convert_curve2d() {
        return true;
    }
    if let Some(trimmed) = curve.cast::<Geom2dTrimmedCurve>().get() {
        return is_convert_curve_2d(&trimmed.basis_curve(), limits, p);
    }
    if let Some(offset) = curve.cast::<Geom2dOffsetCurve>().get() {
        return p.convert_offset_curv2d() || is_convert_curve_2d(&offset.basis_curve(), limits, p);
    }
    if let Some(spline) = curve.cast::<Geom2dBSplineCurve>().get() {
        return spline.degree() > limits.degree
            || spline.nb_knots() - 1 > limits.segments
            || limits.rational && spline.is_rational();
    }
    if let Some(bezier) = curve.cast::<Geom2dBezierCurve>().get() {
        return bezier.degree() > limits.degree || limits.rational && bezier.is_rational();
    }
    false
}

/// `IsConvertSurface`: elementary surfaces by their modes, swept surfaces by their mode or
/// basis curve, trimmed and offset surfaces by their basis (offsets always with
/// ConvertOffsetSurf), B-splines above the limits, Beziers with ConvertBezierSurf or above
/// the degree, and rational ones when `rational`.
pub(super) fn is_convert_surface(
    surface: &Handle<dyn GeomSurface>,
    limits: Limits,
    p: &ShapeCustomRestrictionParameters,
) -> bool {
    if surface.is_null() {
        return false;
    }
    if let Some(mode) = elementary_mode(surface, p) {
        return mode;
    }
    if let Some(swept) = surface.cast::<dyn GeomSweptSurface>().get() {
        return swept_mode(surface, p) || is_convert_curve_3d(&swept.basis_curve(), limits, p);
    }
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        return is_convert_surface(&trimmed.basis_surface(), limits, p);
    }
    if let Some(offset) = surface.cast::<GeomOffsetSurface>().get() {
        return p.convert_offset_surf() || is_convert_surface(&offset.basis_surface(), limits, p);
    }
    spline_surface_needs_conversion(surface, limits, p)
}

fn elementary_mode(
    surface: &Handle<dyn GeomSurface>,
    p: &ShapeCustomRestrictionParameters,
) -> Option<bool> {
    if !surface.cast::<GeomPlane>().is_null() {
        return Some(p.convert_plane());
    }
    if !surface.cast::<GeomConicalSurface>().is_null() {
        return Some(p.convert_conical_surf());
    }
    if !surface.cast::<GeomSphericalSurface>().is_null() {
        return Some(p.convert_spherical_surf());
    }
    if !surface.cast::<GeomCylindricalSurface>().is_null() {
        return Some(p.convert_cylindrical_surf());
    }
    if !surface.cast::<GeomToroidalSurface>().is_null() {
        return Some(p.convert_toroidal_surf());
    }
    None
}

fn swept_mode(surface: &Handle<dyn GeomSurface>, p: &ShapeCustomRestrictionParameters) -> bool {
    !surface.cast::<GeomSurfaceOfRevolution>().is_null() && p.convert_revolution_surf()
        || !surface.cast::<GeomSurfaceOfLinearExtrusion>().is_null() && p.convert_extrusion_surf()
}

fn spline_surface_needs_conversion(
    surface: &Handle<dyn GeomSurface>,
    limits: Limits,
    p: &ShapeCustomRestrictionParameters,
) -> bool {
    if let Some(s) = surface.cast::<GeomBSplineSurface>().get() {
        return s.u_degree() > limits.degree
            || s.v_degree() > limits.degree
            || (s.nb_u_knots() - 1) * (s.nb_v_knots() - 1) > limits.segments
            || limits.rational && (s.is_u_rational() || s.is_v_rational());
    }
    if let Some(s) = surface.cast::<GeomBezierSurface>().get() {
        return p.convert_bezier_surf()
            || s.u_degree() > limits.degree
            || s.v_degree() > limits.degree
            || limits.rational && (s.is_u_rational() || s.is_v_rational());
    }
    false
}
