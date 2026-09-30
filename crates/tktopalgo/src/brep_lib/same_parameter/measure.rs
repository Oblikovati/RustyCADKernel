// Rust translation of EvalTol and ComputeTol (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The deviations of `BRepLib::SameParameter` between a 3d curve and a curve on
//! surface.
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::LocateExtPC;

/// The static `EvalTol`: the largest of `tol` and the distances from the 3d curve to
/// the surface points of the p-curve at 5 inner parameters, each located on the curve
/// from the same parameter; None unless more than 2 of them are located.
pub(super) fn eval_tol(
    pc: &Handle<dyn Geom2dCurve>,
    s: &Handle<dyn GeomSurface>,
    gac: &GeomAdaptorCurve,
    tol: f64,
) -> Result<Option<f64>, Failure> {
    let (pc, s) = (live(pc)?, live(s)?);
    let (f, l) = (gac.first_parameter()?, gac.last_parameter()?);
    let mut projector = LocateExtPC::new();
    projector.initialize(gac, (f, l), tol)?;
    let mut tolbail: f64 = tol;
    let mut ok: i32 = 0;
    for i in 1..=5 {
        let t: f64 = f64::from(i) / 6.;
        let t: f64 = (1. - t) * f + t * l;
        let uv = pc.value(t)?;
        let p = s.value(uv.x(), uv.y())?;
        projector.perform(p, t)?;
        if projector.is_done() {
            let dist2: f64 = projector.square_distance()?;
            if dist2 > tolbail * tolbail {
                tolbail = dist2.sqrt();
            }
            ok += 1;
        }
    }
    Ok((ok > 2).then_some(tolbail))
}

/// The live object of a handle (Standard_NullObject, a crash natively).
fn live<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: &Handle<T>,
) -> Result<&T, Failure> {
    handle.get().ok_or_else(|| {
        crate::brep_lib::errors::null_object("BRepLib::SameParameter: null curve or surface")
    })
}

/// The samples of ComputeTol: the square distances (-1 where none was measured), the
/// largest one, and the largest distance of the parameters outside the surface
/// (`dapp`, -1 when none).
struct Samples {
    dist: Vec<f64>,
    d2: f64,
    dapp: f64,
}

/// The static `ComputeTol`: the deviation between the 3d curve and the curve on
/// surface at `nbp + 1` parameters: infinite when a surface point is, the distance of
/// a point outside the surface domain when larger, else 1.5 times the largest
/// distance (or the largest below 1 when fewer than 10% are at least 1), at least 1e-7.
pub(super) fn compute_tol(
    c3d: &Handle<dyn Adaptor3dCurve>,
    c2d: &Handle<dyn Adaptor2dCurve2d>,
    surf: &Handle<dyn Adaptor3dSurface>,
    nbp: i32,
) -> Result<f64, Failure> {
    let samples = sample(live(c3d)?, live(c2d)?, live(surf)?, nbp)?;
    if precision::is_infinite(samples.d2) {
        return Ok(samples.d2);
    }
    let d2: f64 = samples.d2.sqrt();
    if samples.dapp > d2 {
        return Ok(samples.dapp);
    }
    let d2 = match analytic_bound(&samples.dist) {
        None => 1.5 * d2,
        Some(big_d2) => 1.5 * big_d2.sqrt(),
    };
    Ok(std_max(d2, 1.0e-7))
}

/// The sample loop of ComputeTol.
fn sample(
    c3d: &dyn Adaptor3dCurve,
    c2d: &dyn Adaptor2dCurve2d,
    surf: &dyn Adaptor3dSurface,
    nbp: i32,
) -> Result<Samples, Failure> {
    let mut samples = Samples {
        dist: vec![-1.; (nbp + 10) as usize],
        d2: 0.,
        dapp: -1.,
    };
    let (uf, ul) = (surf.first_u_parameter()?, surf.last_u_parameter()?);
    let (vf, vl) = (surf.first_v_parameter()?, surf.last_v_parameter()?);
    let (du, dv) = (0.01 * (ul - uf), 0.01 * (vl - vf));
    let (u_periodic, v_periodic) = (surf.is_u_periodic()?, surf.is_v_periodic()?);
    let (dsdu, dsdv) = (1. / surf.u_resolution(1.)?, 1. / surf.v_resolution(1.)?);
    let (first, last) = (c3d.first_parameter()?, c3d.last_parameter()?);
    for i in 0..=nbp {
        let t: f64 = f64::from(i) / f64::from(nbp);
        let u: f64 = first * (1. - t) + last * t;
        let pc3d = c3d.value(u)?;
        let puv = c2d.value(u)?;
        let outside = [
            (!u_periodic, puv.x(), uf, ul, du, dsdu),
            (!v_periodic, puv.y(), vf, vl, dv, dsdv),
        ];
        if let Some(distance) = outside_distance(&outside) {
            samples.dapp = std_max(samples.dapp, distance);
            continue;
        }
        let pcons = surf.value(puv.x(), puv.y())?;
        if precision::is_infinite(pcons.x())
            || precision::is_infinite(pcons.y())
            || precision::is_infinite(pcons.z())
        {
            samples.d2 = precision::infinite();
            break;
        }
        let temp: f64 = pc3d.square_distance(&pcons);
        samples.dist[i as usize] = temp;
        samples.d2 = std_max(samples.d2, temp);
    }
    Ok(samples)
}

/// The distance of a parameter outside the domain of a non-periodic direction (by
/// more than 1% of it), checked in U then V, as `DSdu * (uf - u)` and the like.
fn outside_distance(directions: &[(bool, f64, f64, f64, f64, f64); 2]) -> Option<f64> {
    for &(check, p, lo, hi, margin, scale) in directions {
        if !check {
            continue;
        }
        if p < lo - margin {
            return Some(scale * (lo - p));
        }
        if p > hi + margin {
            return Some(scale * (p - hi));
        }
    }
    None
}

/// "ana": when some but fewer than 10% (integer percent) of the measured square
/// distances are at least 1 while more are below, the largest one below 1.
fn analytic_bound(dist: &[f64]) -> Option<f64> {
    let (mut n1, mut n2) = (0_i32, 0_i32);
    for &d in dist.iter().filter(|&&d| d > 0.) {
        if d < 1.0 {
            n1 += 1;
        } else {
            n2 += 1;
        }
    }
    let mut n3: i32 = 0;
    if n1 > n2 && n2 != 0 {
        n3 = 100 * n2 / (n1 + n2);
    }
    if !(n3 < 10 && n3 != 0) {
        return None;
    }
    let mut big_d2: f64 = 0.;
    for &d in dist.iter().filter(|&&d| d > 0. && d < 1.0) {
        big_d2 = std_max(big_d2, d);
    }
    Some(big_d2)
}
