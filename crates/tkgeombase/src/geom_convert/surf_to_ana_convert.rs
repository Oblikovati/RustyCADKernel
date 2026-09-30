// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_SurfToAnaSurf.cxx
// (ConvertToAnalytical(InitialToler, Umin, Umax, Vmin, Vmax))
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The bounded conversion of `GeomConvert_SurfToAnaSurf`: the plane test, the
//! recognition from the middle isolines, and the final choice by gap.
use super::surf_to_ana_gauss::try_cylinder_by_gauss_field;
use super::surf_to_ana_try::{compute_gap, try_cyliner_cone, try_torus_sphere};
use super::{AnaCurveFit, ConvType, CurveToAnaCurve, SurfToAnaSurf};
use crate::geom_lib::{IsPlanarSurface, live};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomCircle, GeomConicalSurface, GeomCurve,
    GeomCylindricalSurface, GeomLine, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface, SurfaceBounds,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};

// RealLast(): the initial gap of every candidate.
const REAL_LAST: f64 = f64::MAX;

// The candidate types, in the native order of newSurf and dd.
const S_TYPES: [SurfaceType; 5] = [
    SurfaceType::Plane,
    SurfaceType::Cylinder,
    SurfaceType::Cone,
    SurfaceType::Sphere,
    SurfaceType::Torus,
];

/// The candidate surfaces (plane, cylinder, cone, sphere, torus) and their gaps.
struct SurfCandidates {
    surfaces: [Handle<dyn GeomSurface>; 5],
    gaps: [f64; 5],
}

// A decision to return (Some, possibly a null surface) or to go on (None).
type Decision = Option<Handle<dyn GeomSurface>>;

/// The middle isolines recognized as analytic curves.
struct MidIsos {
    umidiso: Handle<dyn GeomCurve>,
    vmidiso: Handle<dyn GeomCurve>,
    line_iso: bool,
}

impl SurfToAnaSurf {
    /// `ConvertToAnalytical(InitialToler, Umin, Umax, Vmin, Vmax)`: an elementary
    /// surface is copied with gap 0; otherwise the plane, then (from the middle
    /// isolines) a sphere or torus, a cylinder or cone, or a cylinder from the Gauss
    /// field, the choice following the conversion type; null when none is within
    /// the tolerance or the range exceeds the bounds;
    /// `conv.convert_to_analytical_in(1e-7, (0.0, 1.0), (0.0, 1.0))?`.
    pub fn convert_to_analytical_in(
        &mut self,
        initial_toler: f64,
        (umin, umax): (f64, f64),
        (vmin, vmax): (f64, f64),
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let adaptor = GeomAdaptorSurface::with_surface(&self.my_surf)?;
        let s_type: SurfaceType = adaptor.get_type()?;
        if let Some(copy) = elementary_copy(&adaptor, s_type)? {
            self.my_gap = 0.0;
            return Ok(copy);
        }
        let surf: Handle<dyn GeomSurface> = self.my_surf.clone();
        let surface: &dyn GeomSurface =
            live(&surf, "GeomConvert_SurfToAnaSurf::ConvertToAnalytical")?;
        let Some((bounds, do_segment)) = working_bounds(surface.bounds(), [umin, umax, vmin, vmax])
        else {
            return Ok(Handle::default());
        };
        let temp_s = working_surface(&surf, s_type, bounds, do_segment)?;
        let mut candidates = SurfCandidates {
            surfaces: std::array::from_fn(|_| Handle::default()),
            gaps: [REAL_LAST; 5],
        };
        if let Some(decided) = self.plane_stage(&temp_s, bounds, initial_toler, &mut candidates)? {
            return Ok(decided);
        }
        let toler: f64 = limited_tolerance(surface, bounds, initial_toler)?;
        if let Some(decided) = self.iso_stage(&temp_s, bounds, toler, &mut candidates)? {
            return Ok(decided);
        }
        self.verification(&temp_s, bounds, toler, &mut candidates)
    }

    // The planarity test (on the tolerance given): the plane is returned at once by
    // Simplest and Target plane; Target plane returns null otherwise.
    fn plane_stage(
        &mut self,
        temp_s: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        toler: f64,
        candidates: &mut SurfCandidates,
    ) -> Result<Decision, Failure> {
        let target_plane: bool =
            self.my_conv_type == ConvType::Target && self.my_target == SurfaceType::Plane;
        let planar = IsPlanarSurface::new(temp_s, toler)?;
        if !planar.is_planar() {
            if target_plane {
                self.my_gap = candidates.gaps[0];
                return Ok(Some(Handle::default()));
            }
            return Ok(None);
        }
        let plane: Handle<dyn GeomSurface> =
            Handle::new(GeomPlane::from_pln(planar.plan()?)).cast();
        candidates.gaps[0] = compute_gap(temp_s, bounds, &plane, REAL_LAST)?;
        candidates.surfaces[0] = plane;
        if self.my_conv_type == ConvType::Simplest || target_plane {
            self.my_gap = candidates.gaps[0];
            return Ok(Some(candidates.surfaces[0].clone()));
        }
        Ok(None)
    }

    // The recognition from the middle isolines, then the Gauss field cylinder when
    // an isoline is a line and nothing else was found.
    fn iso_stage(
        &mut self,
        temp_s: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        toler: f64,
        candidates: &mut SurfCandidates,
    ) -> Result<Decision, Failure> {
        let isos: MidIsos = mid_isos(temp_s, bounds, toler)?;
        let (mut cylinder_conus, mut toroid_sphere) = (false, false);
        if isos.umidiso.is_some() && isos.vmidiso.is_some() {
            let u_circle: bool = isos.umidiso.cast::<GeomCircle>().is_some();
            let v_circle: bool = isos.vmidiso.cast::<GeomCircle>().is_some();
            let u_line: bool = isos.umidiso.cast::<GeomLine>().is_some();
            let v_line: bool = isos.vmidiso.cast::<GeomLine>().is_some();
            let target: bool = self.my_conv_type == ConvType::Target;
            if u_circle && v_circle {
                toroid_sphere = true;
                if target && matches!(self.my_target, SurfaceType::Cylinder | SurfaceType::Cone) {
                    self.my_gap = candidates.gaps[1];
                    return Ok(Some(candidates.surfaces[1].clone()));
                }
            } else if (u_line && v_circle) || (u_circle && v_line) {
                cylinder_conus = true;
                if target && matches!(self.my_target, SurfaceType::Sphere | SurfaceType::Torus) {
                    self.my_gap = candidates.gaps[3];
                    return Ok(Some(candidates.surfaces[3].clone()));
                }
            }
            if toroid_sphere {
                if let Some(decided) = self.torus_sphere_stage(&isos, bounds, toler, candidates)? {
                    return Ok(Some(decided));
                }
            } else if cylinder_conus {
                let v_case: bool = u_line && v_circle;
                let (decided, found) =
                    self.cylinder_cone_stage(temp_s, &isos, v_case, (bounds, toler), candidates)?;
                if decided.is_some() {
                    return Ok(decided);
                }
                cylinder_conus = found;
            }
        }
        // Additional checking for the case of a cylinder: try the Gauss field.
        if !cylinder_conus && !toroid_sphere && isos.line_iso {
            let object = try_cylinder_by_gauss_field(temp_s, bounds, toler, (7, 7), true)?;
            if object.is_some() {
                candidates.surfaces[1] = object;
            }
        }
        Ok(None)
    }

    // The sphere or torus of the circular middle isolines: V circles of the same
    // radius first, then U circles. Target of another type returns null.
    fn torus_sphere_stage(
        &mut self,
        isos: &MidIsos,
        (u1, u2, v1, v2): SurfaceBounds,
        toler: f64,
        candidates: &mut SurfCandidates,
    ) -> Result<Decision, Failure> {
        let mut isurf: usize = 3; // set spherical surface
        let ucircle: Handle<GeomCircle> = isos.umidiso.cast::<GeomCircle>();
        let vcircle: Handle<GeomCircle> = isos.vmidiso.cast::<GeomCircle>();
        // Try when the V isolines have the same radius, then the U isolines.
        let mut object = try_torus_sphere(
            &self.my_surf,
            (&vcircle, &ucircle),
            (v1, v2),
            (u1, u2),
            toler,
            true,
        )?;
        if object.is_null() {
            object = try_torus_sphere(
                &self.my_surf,
                (&ucircle, &vcircle),
                (u1, u2),
                (v1, v2),
                toler,
                false,
            )?;
        }
        if object.is_null() {
            self.my_gap = candidates.gaps[isurf];
            return Ok(None);
        }
        if object.cast::<GeomToroidalSurface>().is_some() {
            isurf = 4; // set torus
        }
        candidates.surfaces[isurf] = object;
        if self.my_conv_type == ConvType::Target && self.my_target != S_TYPES[isurf] {
            self.my_gap = REAL_LAST;
            return Ok(Some(Handle::default()));
        }
        Ok(None)
    }

    // The cylinder or cone of the middle isolines (a line and a circle), and
    // whether one was found. Target of another type returns null; without result
    // the gap is reset to dd[1].
    fn cylinder_cone_stage(
        &mut self,
        temp_s: &Handle<dyn GeomSurface>,
        isos: &MidIsos,
        v_case: bool,
        (bounds, toler): (SurfaceBounds, f64),
        candidates: &mut SurfCandidates,
    ) -> Result<(Decision, bool), Failure> {
        let mut isurf: usize = 1; // set cylindrical surface
        let object = try_cyliner_cone(
            temp_s,
            v_case,
            (&isos.umidiso, &isos.vmidiso),
            bounds,
            toler,
        )?;
        if object.is_null() {
            self.my_gap = candidates.gaps[isurf];
            return Ok((None, false));
        }
        if object.cast::<GeomConicalSurface>().is_some() {
            isurf = 2; // set conical surface
        }
        if self.my_conv_type == ConvType::Target && self.my_target != S_TYPES[isurf] {
            self.my_gap = REAL_LAST;
            return Ok((Some(Handle::default()), true));
        }
        candidates.surfaces[isurf] = object;
        Ok((None, true))
    }

    // The final choice: the gaps within the tolerance recomputed in candidate order;
    // Simplest and Target return the first fitting one, MinGap the smallest.
    fn verification(
        &mut self,
        temp_s: &Handle<dyn GeomSurface>,
        bounds: SurfaceBounds,
        toler: f64,
        candidates: &mut SurfCandidates,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let (mut imin, mut dmin): (Option<usize>, f64) = (None, REAL_LAST);
        for (isurf, s_type) in S_TYPES.iter().enumerate() {
            if candidates.surfaces[isurf].is_null() {
                continue;
            }
            candidates.gaps[isurf] =
                compute_gap(temp_s, bounds, &candidates.surfaces[isurf], toler)?;
            if candidates.gaps[isurf] > toler {
                continue;
            }
            if self.my_conv_type == ConvType::Simplest
                || (self.my_conv_type == ConvType::Target && self.my_target == *s_type)
            {
                self.my_gap = candidates.gaps[isurf];
                return Ok(candidates.surfaces[isurf].clone());
            }
            if self.my_conv_type == ConvType::MinGap && candidates.gaps[isurf] < dmin {
                (dmin, imin) = (candidates.gaps[isurf], Some(isurf));
            }
        }
        let Some(imin) = imin else {
            return Ok(Handle::default());
        };
        self.my_gap = candidates.gaps[imin];
        Ok(candidates.surfaces[imin].clone())
    }
}

// A new plane, cylinder, cone, sphere or torus of an elementary adaptor type.
fn elementary_copy(
    adaptor: &GeomAdaptorSurface,
    s_type: SurfaceType,
) -> Result<Option<Handle<dyn GeomSurface>>, Failure> {
    let copy: Handle<dyn GeomSurface> = match s_type {
        SurfaceType::Plane => Handle::new(GeomPlane::from_pln(adaptor.plane()?)).cast(),
        SurfaceType::Cylinder => {
            Handle::new(GeomCylindricalSurface::from_cylinder(adaptor.cylinder()?)).cast()
        }
        SurfaceType::Cone => Handle::new(GeomConicalSurface::from_cone(adaptor.cone()?)).cast(),
        SurfaceType::Sphere => {
            Handle::new(GeomSphericalSurface::from_sphere(adaptor.sphere()?)).cast()
        }
        SurfaceType::Torus => Handle::new(GeomToroidalSurface::from_torus(adaptor.torus()?)).cast(),
        _ => return Ok(None),
    };
    Ok(Some(copy))
}

// The surface bounds narrowed to the requested range (a side moved when it differs
// by more than PConfusion) and whether a narrowing happened; None when the range
// exceeds the bounds.
fn working_bounds(
    (mut u1, mut u2, mut v1, mut v2): SurfaceBounds,
    [umin, umax, vmin, vmax]: [f64; 4],
) -> Option<(SurfaceBounds, bool)> {
    if umin < u1 || umax > u2 || vmin < v1 || vmax > v2 {
        return None;
    }
    let tol_bnd: f64 = precision::p_confusion();
    let mut do_segment: bool = false;
    if umin - u1 > tol_bnd {
        (u1, do_segment) = (umin, true);
    }
    if u2 - umax > tol_bnd {
        (u2, do_segment) = (umax, true);
    }
    if vmin - v1 > tol_bnd {
        (v1, do_segment) = (vmin, true);
    }
    if v2 - vmax > tol_bnd {
        (v2, do_segment) = (vmax, true);
    }
    Some(((u1, u2, v1, v2), do_segment))
}

// The surface worked on: a (segmented) copy of a B-spline or Bezier surface (null
// when the adaptor type comes from a trimmed basis), a trimmed surface otherwise.
fn working_surface(
    surf: &Handle<dyn GeomSurface>,
    s_type: SurfaceType,
    (u1, u2, v1, v2): SurfaceBounds,
    do_segment: bool,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let member = "GeomConvert_SurfToAnaSurf::ConvertToAnalytical";
    if s_type == SurfaceType::BSplineSurface {
        let bs: Handle<GeomBSplineSurface> =
            live(surf, member)?.copy()?.cast::<GeomBSplineSurface>();
        if do_segment {
            let tolerances = (precision::p_confusion(), precision::p_confusion());
            live(&bs, member)?.segment((u1, u2), (v1, v2), tolerances)?;
        }
        return Ok(bs.cast());
    }
    if s_type == SurfaceType::BezierSurface {
        let bz: Handle<GeomBezierSurface> = live(surf, member)?.copy()?.cast::<GeomBezierSurface>();
        if do_segment {
            live(&bz, member)?.segment(u1, u2, v1, v2)?;
        }
        return Ok(bz.cast());
    }
    let trimmed = GeomRectangularTrimmedSurface::new(surf, (u1, u2), (v1, v2), (true, true))?;
    Ok(Handle::new(trimmed).cast())
}

// The tolerance limited to a thousandth of the "diagonal" from (U1, V1) to
// (U1 + U2, (V1 + V2) / 2) (the native U is not halved).
fn limited_tolerance(
    surface: &dyn GeomSurface,
    (u1, u2, v1, v2): SurfaceBounds,
    toler: f64,
) -> Result<f64, Failure> {
    let diagonal: f64 = surface
        .value(u1, v1)?
        .distance(&surface.value(u1 + u2, (v1 + v2) / 2.0)?);
    let twist: f64 = 1000.0;
    if toler > diagonal / twist {
        return Ok(diagonal / twist);
    }
    Ok(toler)
}

// The middle U and V isolines converted with GeomConvert_Simplest, and whether one
// of them is a line.
fn mid_isos(
    temp_s: &Handle<dyn GeomSurface>,
    (u1, u2, v1, v2): SurfaceBounds,
    toler: f64,
) -> Result<MidIsos, Failure> {
    let surface: &dyn GeomSurface = live(temp_s, "GeomConvert_SurfToAnaSurf::ConvertToAnalytical")?;
    let u_iso = surface.u_iso(0.5 * (u1 + u2))?;
    let v_iso = surface.v_iso(0.5 * (v1 + v2))?;
    let mut fit = AnaCurveFit::new(0.0, 0.0);
    let (simplest, line) = (ConvType::Simplest, CurveType::Line);
    let umidiso =
        CurveToAnaCurve::compute_curve(&u_iso, toler, (v1, v2), &mut fit, simplest, line)?;
    let mut line_iso: bool = umidiso.cast::<GeomLine>().is_some();
    let vmidiso =
        CurveToAnaCurve::compute_curve(&v_iso, toler, (u1, u2), &mut fit, simplest, line)?;
    if vmidiso.is_some() && !line_iso {
        line_iso = vmidiso.cast::<GeomLine>().is_some();
    }
    Ok(MidIsos {
        umidiso,
        vmidiso,
        line_iso,
    })
}
