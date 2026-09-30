// Rust translation of the static helpers FUN_GetMinMaxXYZPnt, FUN_TrimInfSurf,
// FUN_NewFirstLast and FUN_TrimBothSurf of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming of infinite surfaces before their parametric intersection: an infinite
//! surface is trimmed around the points of the finite one, two infinite surfaces are
//! trimmed along their infinite isolines.
use super::pl_inter::{curve_type, geom_surface, iso_of};
use crate::intf::std_max;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::extrema::ExtPS;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Point3;

/// `FUN_GetMinMaxXYZPnt`: the sample points of the surface (in steps of a quarter of
/// its ranges) nearest to and farthest from the origin.
pub(super) fn get_min_max_xyz_pnt(s: &Adaptor3dSurfaceHandle) -> Result<(Point3, Point3), Failure> {
    let du: f64 = 0.25 * (s.last_u_parameter()? - s.first_u_parameter()?).abs();
    let dv: f64 = 0.25 * (s.last_v_parameter()? - s.first_v_parameter()?).abs();
    let mut t_min: f64 = f64::MAX;
    let mut t_max: f64 = -t_min;
    let origin = Point3::new(0.0, 0.0, 0.0);
    let (mut pt_min, mut pt_max) = (origin, origin);
    let mut u: f64 = s.first_u_parameter()?;
    while u <= s.last_u_parameter()? {
        let mut v: f64 = s.first_v_parameter()?;
        while v <= s.last_v_parameter()? {
            let puv: Point3 = s.d0(u, v)?;
            let c_xyz: f64 = puv.xyz().modulus();
            if c_xyz > t_max {
                (t_max, pt_max) = (c_xyz, puv);
            }
            if c_xyz < t_min {
                (t_min, pt_min) = (c_xyz, puv);
            }
            v += dv;
        }
        u += du;
    }
    Ok((pt_min, pt_max))
}

/// The parameter bounds of the extrema of a point on the surface, widened.
fn widen_with_extrema(ext: &ExtPS<'_>, bounds: &mut [f64; 4]) -> Result<(), Failure> {
    for i in 1..=ext.nb_ext()? {
        let (cu, cv) = ext.point(i)?.parameter();
        if cu > bounds[1] {
            bounds[1] = cu;
        }
        if cu < bounds[0] {
            bounds[0] = cu;
        }
        if cv > bounds[3] {
            bounds[3] = cv;
        }
        if cv < bounds[2] {
            bounds[2] = cv;
        }
    }
    Ok(())
}

/// The trim parameter of `FUN_TrimInfSurf`: the largest parameter of the projections
/// of the two points, or `alternative` when both projections fail.
fn trim_parameter(
    pmin: Point3,
    pmax: Point3,
    inf_surf: &Adaptor3dSurfaceHandle,
    alternative: f64,
) -> Result<f64, Failure> {
    let mut ext = ExtPS::new();
    let u_range = (inf_surf.first_u_parameter()?, inf_surf.last_u_parameter()?);
    let v_range = (inf_surf.first_v_parameter()?, inf_surf.last_v_parameter()?);
    ext.initialize(&**inf_surf, u_range, v_range, (1.0e-7, 1.0e-7))?;
    // Umin, Umax, Vmin, Vmax.
    let mut bounds: [f64; 4] = [1.0e+100, -1.0e+100, 1.0e+100, -1.0e+100];
    ext.perform(pmin)?;
    let is_done1: bool = ext.is_done();
    if is_done1 {
        widen_with_extrema(&ext, &mut bounds)?;
    }
    ext.perform(pmax)?;
    let is_done2: bool = ext.is_done();
    if is_done2 {
        widen_with_extrema(&ext, &mut bounds)?;
    }
    if !(is_done1 || is_done2) {
        return Ok(alternative);
    }
    let [umin, umax, vmin, vmax] = bounds;
    Ok(std_max(
        umin.abs(),
        std_max(umax.abs(), std_max(vmin.abs(), vmax.abs())),
    ))
}

/// The trimmed range of an infinite direction (`None` for a finite one).
fn infinite_range(first: f64, last: f64, tp: f64) -> Option<(f64, f64)> {
    let (inf, sup) = (
        precision::is_negative_infinite(first),
        precision::is_positive_infinite(last),
    );
    match (inf, sup) {
        (true, true) => Some((-tp, tp)),
        (true, false) => Some((-tp, last)),
        (false, true) => Some((first, tp)),
        (false, false) => None,
    }
}

/// `FUN_TrimInfSurf(Pmin, Pmax, InfSurf, AlternativeTrimPrm, TrimS)`: the infinite
/// surface trimmed to a thousand times the largest parameter of the projections of the
/// two points (the surface itself for a null parameter).
pub(super) fn trim_inf_surf(
    (pmin, pmax): (Point3, Point3),
    inf_surf: &Adaptor3dSurfaceHandle,
    alternative_trim_prm: f64,
) -> Result<Adaptor3dSurfaceHandle, Failure> {
    let tp: f64 = trim_parameter(pmin, pmax, inf_surf, alternative_trim_prm)?;
    if tp == 0.0 {
        return Ok(inf_surf.clone());
    }
    let tp: f64 = 1000.0 * tp;
    let v_trim = infinite_range(
        inf_surf.first_v_parameter()?,
        inf_surf.last_v_parameter()?,
        tp,
    );
    let u_trim = infinite_range(
        inf_surf.first_u_parameter()?,
        inf_surf.last_u_parameter()?,
        tp,
    );
    let mut trim_s: Adaptor3dSurfaceHandle = Adaptor3dSurfaceHandle::default();
    if let Some((v1, v2)) = v_trim {
        trim_s = inf_surf.v_trim(v1, v2, 1.0e-7)?;
    }
    let base: Adaptor3dSurfaceHandle = if v_trim.is_some() {
        trim_s.clone()
    } else {
        inf_surf.clone()
    };
    if let Some((u1, u2)) = u_trim {
        trim_s = base.u_trim(u1, u2, 1.0e-7)?;
    }
    Ok(trim_s)
}

/// `FUN_NewFirstLast(ga_ct, Fst, Lst, TrVal, NewFst, NewLst, NeedTr)`: the range of an
/// isoline of a line or a parabola cut to `tr_val`, of a hyperbola to 4 (a range beyond
/// 4 keeps its bounds, as natively); `None` when no trim is needed.
// The negated comparison sends the NaN operands where the native one does.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn new_first_last(ga_ct: CurveType, fst: f64, lst: f64, tr_val: f64) -> Option<(f64, f64)> {
    match ga_ct {
        CurveType::Line | CurveType::Parabola => {
            if !((lst - fst).abs() > tr_val) {
                return None;
            }
            // The native case of two positive bounds is followed by an `if`, not an
            // `else if`: its `else` branch overwrites both bounds, so only these two cases
            // remain.
            if fst < 0.0 && lst < 0.0 {
                let new_fst: f64 = if (lst - tr_val) > fst {
                    lst - tr_val
                } else {
                    fst
                };
                return Some((new_fst, lst));
            }
            let new_fst: f64 = if fst < -tr_val { -tr_val } else { fst };
            let new_lst: f64 = if lst > tr_val { tr_val } else { lst };
            Some((new_fst, new_lst))
        }
        CurveType::Hyperbola => hyperbola_first_last(fst, lst),
        _ => None,
    }
}

/// The hyperbola case of `FUN_NewFirstLast`; a range entirely beyond 4 returns with the
/// trim not needed.
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn hyperbola_first_last(fst: f64, lst: f64) -> Option<(f64, f64)> {
    if !((lst - fst).abs() > 10.0) {
        return None;
    }
    // As for the lines, the bounds of the positive case are overwritten; only its early
    // return remains.
    if fst >= 0.0 && lst >= 0.0 && fst > 4.0 {
        return None;
    }
    if fst < 0.0 && lst < 0.0 {
        if lst < -4.0 {
            return None;
        }
        return Some((if fst < -4.0 { -4.0 } else { fst }, lst));
    }
    Some((
        if fst < -4.0 { -4.0 } else { fst },
        if lst > 4.0 { 4.0 } else { lst },
    ))
}

/// A trimmed range of a direction, if any.
type TrimRange = Option<(f64, f64)>;
/// The trims of a surface in V and in U.
type SurfaceTrims = (TrimRange, TrimRange);

/// The trims of a surface in V (by the type of its U isoline) and in U.
fn trim_ranges(
    s: &Adaptor3dSurfaceHandle,
    typ: SurfaceType,
    tv: f64,
) -> Result<Option<SurfaceTrims>, Failure> {
    let gs = geom_surface(s)?;
    let um: f64 = 0.5 * (s.last_u_parameter()? + s.first_u_parameter()?);
    let vm: f64 = 0.5 * (s.last_v_parameter()? + s.first_v_parameter()?);
    let viso = iso_of(&gs, typ, false, vm)?;
    let uiso = iso_of(&gs, typ, true, um)?;
    if uiso.is_null() || viso.is_null() {
        return Ok(None);
    }
    let (ga_u, ga_v) = (curve_type(&uiso)?, curve_type(&viso)?);
    let v_trim = new_first_last(ga_u, s.first_v_parameter()?, s.last_v_parameter()?, tv);
    let u_trim = new_first_last(ga_v, s.first_u_parameter()?, s.last_u_parameter()?, tv);
    Ok(Some((v_trim, u_trim)))
}

/// The surface trimmed in V (`ns` is left unchanged when it is not).
fn trim_v(
    s: &Adaptor3dSurfaceHandle,
    v_trim: TrimRange,
    ns: &mut Adaptor3dSurfaceHandle,
) -> Result<(), Failure> {
    if let Some((v1, v2)) = v_trim {
        *ns = s.v_trim(v1, v2, 1.0e-7)?;
    }
    Ok(())
}

/// The surface (trimmed in V first when it was) trimmed in U.
fn trim_u(
    s: &Adaptor3dSurfaceHandle,
    (v_trim, u_trim): SurfaceTrims,
    ns: &mut Adaptor3dSurfaceHandle,
) -> Result<(), Failure> {
    if let Some((u1, u2)) = u_trim {
        let base: Adaptor3dSurfaceHandle = if v_trim.is_some() {
            ns.clone()
        } else {
            s.clone()
        };
        *ns = base.u_trim(u1, u2, 1.0e-7)?;
    }
    Ok(())
}

/// `FUN_TrimBothSurf(S1, T1, S2, T2, TV, NS1, NS2)`: both surfaces trimmed along their
/// infinite isolines of lines, parabolas and hyperbolas; `ns1`/`ns2` hold the surfaces
/// on input and are left unchanged for a surface that is not trimmed.
pub(super) fn trim_both_surf(
    (s1, t1): (&Adaptor3dSurfaceHandle, SurfaceType),
    (s2, t2): (&Adaptor3dSurfaceHandle, SurfaceType),
    tv: f64,
    (ns1, ns2): (&mut Adaptor3dSurfaceHandle, &mut Adaptor3dSurfaceHandle),
) -> Result<(), Failure> {
    let (r1, r2) = (trim_ranges(s1, t1, tv)?, trim_ranges(s2, t2, tv)?);
    let (Some(r1), Some(r2)) = (r1, r2) else {
        (*ns1, *ns2) = (s1.clone(), s2.clone());
        return Ok(());
    };
    // The native order: both V trims, then both U trims.
    trim_v(s1, r1.0, ns1)?;
    trim_v(s2, r2.0, ns2)?;
    trim_u(s1, r1, ns1)?;
    trim_u(s2, r2, ns2)
}
