// Rust translation of IntPatch_Intersection::CheckSingularPoints, DefineUVMaxStep,
// PrepareSurfaces and the static helper splitCone
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static members of `IntPatch_Intersection` about singular points: the distance
//! from a degenerated boundary of a surface to the other surface, the parametric step
//! it asks for, and the split of a flat cone at its apex.
use super::IntPatchIntersection;
use crate::int_start::{TheArc, TheTopolTool};
use crate::intf::{std_max, std_min};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag, ExtPS};
use rustycad_tkmath::gp::{Point2, Point3, Xyz};
use std::f64::consts::PI;

/// The number of points sampled on a boundary.
const NB_BND_PNTS: i32 = 5;

/// The middle of the points of a boundary where the surface degenerates (the derivative
/// along the boundary stays null), or `None`.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn degenerated_point(s1: &Adaptor3dSurfaceHandle, bnd: &TheArc) -> Result<Option<Point3>, Failure> {
    let (pinf, psup) = (bnd.first_parameter()?, bnd.last_parameter()?);
    if precision::is_negative_infinite(pinf) || precision::is_positive_infinite(psup) {
        return Ok(None);
    }
    let tol: f64 = precision::confusion();
    let dt: f64 = (psup - pinf) / f64::from(NB_BND_PNTS - 1);
    let (_, dir) = bnd.d1((pinf + psup) / 2.0)?;
    let is_u: bool = dir.x().abs() > dir.y().abs();
    let mut d1_norm_max: f64 = 0.0;
    let mut pmid = Xyz::new(0.0, 0.0, 0.0);
    let mut nb: i32 = 0;
    let mut t: f64 = pinf;
    while t <= psup {
        let p1: Point2 = bnd.value(t)?;
        let d1 = s1.d1(p1.x(), p1.y())?;
        let magnitude: f64 = if is_u {
            d1.d1u.magnitude()
        } else {
            d1.d1v.magnitude()
        };
        d1_norm_max = std_max(d1_norm_max, magnitude);
        pmid += d1.point.xyz();
        nb += 1;
        if d1_norm_max > tol {
            break;
        }
        t += dt;
    }
    // A NaN norm is not degenerated, as natively.
    if !(d1_norm_max <= tol) {
        return Ok(None);
    }
    pmid /= f64::from(nb);
    Ok(Some(Point3::from(pmid)))
}

impl IntPatchIntersection {
    /// `CheckSingularPoints(theS1, theD1, theS2, theDist)`: whether a boundary of the
    /// domain of `s1` is degenerated (a pole); `dist` is the distance from the middle of
    /// such boundaries to `s2` (infinite when there is none). Two same surfaces have no
    /// singular point.
    ///
    /// ```ignore
    /// let (singular, dist) = IntPatchIntersection::check_singular_points(&s1, &*d1, &s2)?;
    /// ```
    pub fn check_singular_points(
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
    ) -> Result<(bool, f64), Failure> {
        let mut dist: f64 = precision::infinite();
        if s1 == s2 {
            return Ok((false, dist));
        }
        d1.init();
        while d1.more() {
            if let Some(pp1) = degenerated_point(s1, &d1.value()?)? {
                let tols: (f64, f64) = (precision::p_confusion(), precision::p_confusion());
                let proj =
                    ExtPS::with_surface(pp1, &**s2, tols.0, tols.1, ExtFlag::Min, ExtAlgo::Grad)?;
                if proj.is_done() {
                    for i in 1..=proj.nb_ext()? {
                        dist = std_min(dist, proj.square_distance(i)?);
                    }
                }
            }
            d1.next();
        }
        if !precision::is_infinite(dist) {
            return Ok((true, dist.sqrt()));
        }
        Ok((false, dist))
    }

    /// `DefineUVMaxStep(theS1, theD1, theS2, theD2)`: 0.0001 when a singular point of a
    /// surface is very close (between Confusion and 1e-5) to the other one, else 0.001.
    ///
    /// ```ignore
    /// let step = IntPatchIntersection::define_uv_max_step(&s1, &*d1, &s2, &*d2)?;
    /// ```
    pub fn define_uv_max_step(
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
    ) -> Result<f64, Failure> {
        let mut uv_max_step: f64 = 0.001;
        let (tol_min, tol_max) = (precision::confusion(), 1.0e-5);
        if s1 == s2 {
            return Ok(uv_max_step);
        }
        let (mut is_sing1, dist1) = Self::check_singular_points(s1, d1, s2)?;
        if is_sing1 {
            if dist1 > tol_min && dist1 < tol_max {
                uv_max_step = 0.0001;
            } else {
                is_sing1 = false;
            }
        }
        if !is_sing1 {
            let (is_sing2, dist2) = Self::check_singular_points(s2, d2, s1)?;
            if is_sing2 && dist2 > tol_min && dist2 < tol_max {
                uv_max_step = 0.0001;
            }
        }
        Ok(uv_max_step)
    }

    /// `PrepareSurfaces(theS1, theD1, theS2, theD2, Tol, theSeqHS1, theSeqHS2)`: each
    /// surface, or the two halves of a flat cone (a semi-angle within `tol` of Pi/2)
    /// split at its apex when the apex is in the domain, appended to `seq_hs1` and
    /// `seq_hs2`.
    ///
    /// ```ignore
    /// IntPatchIntersection::prepare_surfaces(&s1, &*d1, &s2, &*d2, 1e-7, &mut v1, &mut v2)?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_surfaces(
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        s2: &Adaptor3dSurfaceHandle,
        d2: &(dyn TheTopolTool + 'static),
        tol: f64,
        seq_hs1: &mut Vec<Adaptor3dSurfaceHandle>,
        seq_hs2: &mut Vec<Adaptor3dSurfaceHandle>,
    ) -> Result<(), Failure> {
        prepare_surface(s1, d1, tol, seq_hs1)?;
        prepare_surface(s2, d2, tol, seq_hs2)
    }
}

/// The surface, or the halves of a flat cone.
fn prepare_surface(
    s: &Adaptor3dSurfaceHandle,
    d: &(dyn TheTopolTool + 'static),
    tol: f64,
    seq: &mut Vec<Adaptor3dSurfaceHandle>,
) -> Result<(), Failure> {
    use rustycad_tkmath::geom_abs::SurfaceType;
    if s.get_type()? == SurfaceType::Cone && (PI / 2.0 - s.cone()?.semi_angle().abs()).abs() < tol {
        return split_cone(s, d, tol, seq);
    }
    seq.push(s.clone());
    Ok(())
}

/// `splitCone`: the two halves of the cone below and above its apex when the apex is in
/// the domain, else the cone.
fn split_cone(
    s: &Adaptor3dSurfaceHandle,
    d: &(dyn TheTopolTool + 'static),
    tol: f64,
    seq: &mut Vec<Adaptor3dSurfaceHandle>,
) -> Result<(), Failure> {
    use rustycad_tkmath::geom_abs::SurfaceType;
    if s.get_type()? != SurfaceType::Cone {
        return Err(Failure::with_message(
            "Standard_NoSuchObject",
            "IntPatch_Intersection : Surface is not Cone",
        ));
    }
    let (u0, v0) = Adaptor3dTopolTool::get_cone_apex_param(&s.cone()?);
    let state: State = d.classify(Point2::new(u0, v0), tol, true)?;
    if state == State::In || state == State::On {
        let dn: Handle<_> = s.v_trim(s.first_v_parameter()?, v0, precision::p_confusion())?;
        let up: Handle<_> = s.v_trim(v0, s.last_v_parameter()?, precision::p_confusion())?;
        seq.push(dn);
        seq.push(up);
        return Ok(());
    }
    seq.push(s.clone());
    Ok(())
}
