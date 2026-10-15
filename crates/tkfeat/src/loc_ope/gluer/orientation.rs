// Rust translation of the static GetOrientation of
// ModelingAlgorithms/TKFeat/LocOpe/LocOpe_Gluer.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The relative orientation of a glued face and its base face: REVERSED when their
//! (oriented) normals agree at a sample point of the boundary of the glued face, so the
//! new shape lies inside the basis.
use crate::errors::null_object;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::extrema::{ExtFlag, ExtPS};
use rustycad_tkmath::gp::{Point3, Vector3};

/// The unit normal of `s` at `(u, v)`, reversed for a reversed face; None when the
/// derivatives are parallel.
fn oriented_normal(
    s: &dyn GeomSurface,
    u: f64,
    v: f64,
    reversed: bool,
) -> Result<Option<(Point3, Vector3)>, Failure> {
    let d1 = s.d1(u, v)?;
    let n = d1.d1u.crossed(d1.d1v);
    if n.magnitude() <= precision::confusion() {
        return Ok(None);
    }
    let n = n.normalized()?;
    Ok(Some((d1.point, if reversed { n.reversed() } else { n })))
}

/// The p-curve range with infinite bounds replaced: [-100, 100], or 200 units from the
/// finite bound.
fn finite_range(mut f: f64, mut l: f64) -> (f64, f64) {
    if precision::is_negative_infinite(f) && precision::is_positive_infinite(l) {
        f = -100.;
        l = 100.;
    } else if precision::is_negative_infinite(f) {
        f = l - 200.;
    } else if precision::is_positive_infinite(l) {
        l = f + 200.;
    }
    (f, l)
}

/// The nearest extremum of the projection of `p` on the base surface, as `(u, v)`.
fn nearest_on_base(ext_ps: &mut ExtPS<'_>, p: Point3) -> Result<Option<(f64, f64)>, Failure> {
    ext_ps.perform(p)?;
    if !ext_ps.is_done() {
        return Ok(None);
    }
    let (mut dist2min, mut jmin) = (f64::MAX, 0);
    for j in 1..=ext_ps.nb_ext()? {
        let d = ext_ps.square_distance(j)?;
        if d < dist2min {
            jmin = j;
            dist2min = d;
        }
    }
    if jmin == 0 {
        return Ok(None);
    }
    Ok(Some(ext_ps.point(jmin)?.parameter()))
}

/// `GetOrientation(Fn, Fb)`: REVERSED when the normals of `fn_` (at a point of its
/// boundary, sampled 21 times per edge) and of `fb` (at the projection of that point)
/// agree, FORWARD when they are opposed, INTERNAL when no point gives both normals.
pub(super) fn get_orientation(fn_: &Face, fb: &Face) -> Result<Orientation, Failure> {
    let sn = BRepTool::surface(fn_)?;
    let sb = BRepTool::surface(fb)?;
    let null = || null_object("LocOpe_Gluer::GetOrientation", "face without surface");
    let (sn_ref, sb_ref) = (sn.get().ok_or_else(null)?, sb.get().ok_or_else(null)?);
    // Initialize extrema projector once for the target surface
    let gas = GeomAdaptorSurface::with_surface(&sb)?;
    let tol_u = gas.u_resolution(precision::confusion())?;
    let tol_v = gas.v_resolution(precision::confusion())?;
    let mut ext_ps = ExtPS::new();
    ext_ps.initialize(
        &gas,
        (gas.first_u_parameter()?, gas.last_u_parameter()?),
        (gas.first_v_parameter()?, gas.last_v_parameter()?),
        (tol_u, tol_v),
    )?;
    ext_ps.set_flag(ExtFlag::Min);
    for e in Explorer::new(fn_, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let pc = BRepTool::curve_on_face(&topods::edge(&e)?, fn_)?;
        let c2d: &dyn Geom2dCurve = pc
            .pcurve
            .get()
            .ok_or_else(|| null_object("LocOpe_Gluer::GetOrientation", "edge without p-curve"))?;
        let (f, l) = finite_range(pc.first, pc.last);
        let deltau = (l - f) / 20.;
        for i in 1..=21 {
            let ptvtx = c2d.d0(f + f64::from(i - 1) * deltau)?;
            let Some((pvt, n1)) = oriented_normal(
                sn_ref,
                ptvtx.x(),
                ptvtx.y(),
                fn_.orientation() == Orientation::Reversed,
            )?
            else {
                continue;
            };
            // Projection sur Sb
            let Some((uu, vv)) = nearest_on_base(&mut ext_ps, pvt)? else {
                continue;
            };
            let Some((_, n2)) =
                oriented_normal(sb_ref, uu, vv, fb.orientation() == Orientation::Reversed)?
            else {
                continue;
            };
            return Ok(if n1.dot(n2) > 0. {
                Orientation::Reversed
            } else {
                Orientation::Forward
            });
        }
    }
    Ok(Orientation::Internal)
}
