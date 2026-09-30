// Rust translation of the EstLimFor* functions of IntCurveSurface_InterUtils.pxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The finite parameter box a line can meet on an infinite swept surface
//! (`EstLimForInfExtr`, `EstLimForInfRevl`, `EstLimForInfSurf`).
use super::{InfFlags, THE_TOLERANCE_ANGULAIRE, UvLimits};
use crate::int_curve_surface::tools::InterSurfaceTool;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::extrema::ExtElC;
use rustycad_tkgeombase::int_ana2d::{AnaIntersection, Conic};
use rustycad_tkgeombase::proj_lib::{Plane as ProjLibPlane, Project};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis1, Axis3, Direction2, Direction3, Line2, Line3, Plane3, Point2, Point3, Vector2, Vector3,
};
use std::f64::consts::PI;

/// `EstLimForInfSurf`: every bound clamped to `[-1e10, 1e10]`;
/// `est_lim_for_inf_surf(&mut limits)`.
pub fn est_lim_for_inf_surf(limits: &mut UvLimits) {
    limits.u1 = std_max(limits.u1, -1.0e10);
    limits.u2 = std_min(limits.u2, 1.0e10);
    limits.v1 = std_max(limits.v1, -1.0e10);
    limits.v2 = std_min(limits.v2, 1.0e10);
}

/// `vmin - |vmin| - 10` and `vmax + |vmax| + 10`: the widened range of the meeting
/// parameters.
fn widened(min: f64, max: f64) -> (f64, f64) {
    (min - min.abs() - 10.0, max + max.abs() + 10.0)
}

/// `EstLimForInfExtr`: bounds for an extrusion (or, with `is_off_surf`, the offset
/// of one) that the line can meet; returns NoIntersection. An infinite V range is
/// bounded by the meeting points of the line with the rulings at `nbsu + 1` values
/// of u; an infinite U range by the meeting points of the projections of the line
/// and the basis curve on a plane normal to the extrusion, then V as before;
/// `est_lim_for_inf_extr::<HSurfaceTool>(line, &s, false, 10, inf, &mut limits)?`.
pub fn est_lim_for_inf_extr<S: InterSurfaceTool>(
    line: Line3,
    surface: &S::Surface,
    is_off_surf: bool,
    nbsu: i32,
    inf: InfFlags,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    let bas_surf: Option<Adaptor3dSurfaceHandle> = if is_off_surf {
        Some(S::basis_surface(surface)?)
    } else {
        None
    };
    let dir_of_ext: Direction3 = match &bas_surf {
        Some(basis) => basis.direction()?,
        None => S::direction(surface)?,
    };
    if dir_of_ext.is_parallel(line.direction(), THE_TOLERANCE_ANGULAIRE) {
        return Ok(true);
    }
    let (u_inf, v_inf) = (inf.u1 || inf.u2, inf.v1 || inf.v2);
    if v_inf && !u_inf {
        return extr_v_limits::<S>(line, surface, dir_of_ext, nbsu, limits);
    }
    if !u_inf {
        return Ok(false);
    }
    let Some(no_intersection) =
        extr_u_limits::<S>(line, surface, bas_surf.as_ref(), dir_of_ext, limits)?
    else {
        return Ok(false);
    };
    if no_intersection {
        return Ok(true);
    }
    if v_inf {
        let v_only = InfFlags {
            u1: false,
            u2: false,
            ..inf
        };
        return est_lim_for_inf_extr::<S>(line, surface, is_off_surf, nbsu, v_only, limits);
    }
    Ok(false)
}

/// The V bounds of an extrusion: the line parameters... of the rulings nearest to the
/// line at `nbsu + 1` values of u; true (NoIntersection) for a ruling parallel to it.
fn extr_v_limits<S: InterSurfaceTool>(
    line: Line3,
    surface: &S::Surface,
    dir_of_ext: Direction3,
    nbsu: i32,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    let (mut vmin, mut vmax) = (real_last(), -real_last());
    let step: f64 = (limits.u2 - limits.u1) / f64::from(nbsu);
    let mut u: f64 = limits.u1;
    for _ in 0..=nbsu {
        let p: Point3 = S::d0(surface, u, 0.0)?;
        let extr = ExtElC::lines(Line3::new(p, dir_of_ext), line, THE_TOLERANCE_ANGULAIRE);
        if !extr.is_done() {
            return Ok(false);
        }
        if extr.is_parallel()? {
            return Ok(true);
        }
        let v: f64 = extr.points(1)?.0.parameter();
        vmin = std_min(vmin, v);
        vmax = std_max(vmax, v);
        u += step;
    }
    let (vmin, vmax) = widened(vmin, vmax);
    limits.v1 = std_max(limits.v1, vmin);
    limits.v2 = std_min(limits.v2, vmax);
    Ok(false)
}

/// The U bounds of an extrusion from the plane projections of the line and of the
/// basis curve: `None` when the native code returns early, else NoIntersection.
fn extr_u_limits<S: InterSurfaceTool>(
    line: Line3,
    surface: &S::Surface,
    bas_surf: Option<&Adaptor3dSurfaceHandle>,
    dir_of_ext: Direction3,
    limits: &mut UvLimits,
) -> Result<Option<bool>, Failure> {
    let u0: f64 = std_min(std_max(0.0, limits.u1), limits.u2);
    let v0: f64 = std_min(std_max(0.0, limits.v1), limits.v2);
    let p: Point3 = S::d0(surface, u0, v0)?;
    let bas_curv: Handle<dyn Adaptor3dCurve> = match bas_surf {
        Some(basis) => basis.basis_curve()?,
        None => S::basis_curve(surface)?,
    };
    let mut projector = ProjLibPlane::with_plane(&Plane3::new(p, dir_of_ext)?);
    projector.project_line(&line)?;
    if !projector.is_done() {
        return Ok(None);
    }
    let line2d: Line2 = projector.line()?;
    let range: Option<(f64, f64)> = match bas_curv.get_type()? {
        CurveType::Line => match basis_line_meeting(&mut projector, &bas_curv, line2d)? {
            Meeting::Early => return Ok(None),
            Meeting::None => return Ok(Some(true)),
            Meeting::Range(range) => Some(range),
        },
        kind @ (CurveType::Parabola | CurveType::Hyperbola) => {
            match basis_conic_meeting(&mut projector, &bas_curv, kind, line2d)? {
                Meeting::Early => return Ok(None),
                Meeting::None => return Ok(Some(true)),
                Meeting::Range(range) => Some(range),
            }
        }
        _ => None,
    };
    let Some((umin, umax)) = range else {
        return Ok(None);
    };
    let (umin, umax) = widened(umin, umax);
    limits.u1 = std_max(limits.u1, umin);
    limits.u2 = std_min(limits.u2, umax);
    Ok(Some(false))
}

/// The outcome of a projected intersection: an early return, no meeting point, or
/// the range of the meeting parameters.
enum Meeting {
    Early,
    None,
    Range((f64, f64)),
}

/// The meeting of the projected line with the projection of a line basis curve,
/// read on the basis curve.
fn basis_line_meeting(
    projector: &mut ProjLibPlane,
    bas_curv: &Handle<dyn Adaptor3dCurve>,
    line2d: Line2,
) -> Result<Meeting, Failure> {
    projector.project_line(&bas_curv.line()?)?;
    if !projector.is_done() {
        return Ok(Meeting::Early);
    }
    let inter = AnaIntersection::lines(line2d, projector.line()?);
    if !inter.is_done() {
        return Ok(Meeting::Early);
    }
    if inter.is_empty()? || inter.identical_elements()? || inter.parallel_elements()? {
        return Ok(Meeting::None);
    }
    let u: f64 = inter.point(1)?.param_on_second()?;
    Ok(Meeting::Range((u, u)))
}

/// The meetings of the projected line with the projection of a parabola or
/// hyperbola basis curve, read on the conic.
fn basis_conic_meeting(
    projector: &mut ProjLibPlane,
    bas_curv: &Handle<dyn Adaptor3dCurve>,
    kind: CurveType,
    line2d: Line2,
) -> Result<Meeting, Failure> {
    let con = Conic::from_line(line2d);
    let inter: AnaIntersection = if kind == CurveType::Parabola {
        projector.project_parabola(&bas_curv.parabola()?)?;
        if !projector.is_done() {
            return Ok(Meeting::Early);
        }
        AnaIntersection::parabola_conic(projector.parabola()?, &con)?
    } else {
        projector.project_hyperbola(&bas_curv.hyperbola()?)?;
        if !projector.is_done() {
            return Ok(Meeting::Early);
        }
        AnaIntersection::hyperbola_conic(projector.hyperbola()?, &con)?
    };
    if !inter.is_done() {
        return Ok(Meeting::Early);
    }
    if inter.is_empty()? {
        return Ok(Meeting::None);
    }
    let (mut umin, mut umax) = (real_last(), -real_last());
    for i in 1..=inter.nb_points()? {
        let param: f64 = inter.point(i)?.param_on_first();
        umin = std_min(param, umin);
        umax = std_max(param, umax);
    }
    Ok(Meeting::Range((umin, umax)))
}

/// `EstLimForInfRevl`: bounds for a surface of revolution the line can meet;
/// returns NoIntersection. An infinite U bound becomes 0 or 2 pi (the other one
/// untouched); infinite V bounds come from the meetings of the basis curve, in a
/// plane through the axis, with the projections of the line (and of its mirror) on
/// two perpendicular planes through the axis;
/// `est_lim_for_inf_revl::<HSurfaceTool>(line, &s, inf, &mut limits)?`.
pub fn est_lim_for_inf_revl<S: InterSurfaceTool>(
    line: Line3,
    surface: &S::Surface,
    inf: InfFlags,
    limits: &mut UvLimits,
) -> Result<bool, Failure> {
    if inf.u1 || inf.u2 {
        if inf.u1 {
            limits.u1 = std_max(0.0, limits.u1);
        } else {
            limits.u2 = std_min(2.0 * PI, limits.u2);
        }
        if !inf.v1 && !inf.v2 {
            return Ok(false);
        }
    }
    let basis_curve: Handle<dyn Adaptor3dCurve> = S::basis_curve(surface)?;
    let rev_ax: Axis1 = S::axe_of_revolution(surface)?;
    let Some((bas_curv_proj, o, x_vec, u_used)) = project_basis_curve(&basis_curve, rev_ax)? else {
        return Ok(false);
    };
    let pln: Plane3 = line_plane(line, o, x_vec, u_used)?;
    let pln_prp: Plane3 = pln.rotated(Axis1::new(o, Direction3::from_vector(x_vec)?), PI / 2.0)?;
    let (mut vmin, mut vmax) = (real_last(), -real_last());
    let no_int1: bool =
        project_intersect_and_est_lim(line, &pln, &bas_curv_proj, &mut vmin, &mut vmax)?;
    let no_int2: bool =
        project_intersect_and_est_lim(line, &pln_prp, &bas_curv_proj, &mut vmin, &mut vmax)?;
    if no_int1 && no_int2 {
        return Ok(true);
    }
    let (vmin, vmax) = widened(vmin, vmax);
    if inf.v1 {
        limits.v1 = vmin;
    }
    if inf.v2 {
        limits.v2 = vmax;
    }
    Ok(false)
}

/// The projection of the basis curve on a plane through the axis and a point of the
/// curve off the axis (at u = 0, 1, 2..., or an arbitrary point after u = 3), with
/// the axis origin and direction and the last u tried; `None` for a basis curve
/// that is not a line, parabola or hyperbola, or a projection not done.
fn project_basis_curve(
    basis_curve: &Handle<dyn Adaptor3dCurve>,
    rev_ax: Axis1,
) -> Result<Option<(ProjLibPlane, Point3, Vector3, f64)>, Failure> {
    let x_vec: Vector3 = Vector3::from(rev_ax.direction());
    let ang_tol: f64 = precision::angular();
    let o: Point3 = rev_ax.location();
    let mut u: f64 = 0.0;
    let mut p: Point3 = basis_curve.value(u)?;
    while o.square_distance(&p) <= precision::p_confusion()
        || x_vec.is_parallel(Vector3::between(o, p), ang_tol)?
    {
        u += 1.0;
        p = basis_curve.value(u)?;
        if u > 3.0 {
            // The basis curve is a line along the axis: any point off it will do.
            p = Point3::new(u, u + 1.0, u + 2.0);
        }
    }
    let n_vec: Vector3 = x_vec.crossed(Vector3::between(o, p));
    let pln = Plane3::from_position(Axis3::new(
        o,
        Direction3::from_vector(n_vec)?,
        Direction3::from_vector(x_vec)?,
    )?);
    let mut proj = ProjLibPlane::with_plane(&pln);
    match basis_curve.get_type()? {
        CurveType::Line => proj.project_line(&basis_curve.line()?)?,
        CurveType::Hyperbola => proj.project_hyperbola(&basis_curve.hyperbola()?)?,
        CurveType::Parabola => proj.project_parabola(&basis_curve.parabola()?)?,
        _ => return Ok(None),
    }
    if !proj.is_done() {
        return Ok(None);
    }
    Ok(Some((proj, o, x_vec, u)))
}

/// The plane through the axis that holds the line direction (or, for a line along
/// the axis, a point off the axis), with the axis as X direction.
fn line_plane(line: Line3, o: Point3, x_vec: Vector3, mut u: f64) -> Result<Plane3, Failure> {
    let n_vec: Vector3 =
        if x_vec.is_parallel(Vector3::from(line.direction()), precision::angular())? {
            let mut p: Point3 = line.location();
            while o.square_distance(&p) <= precision::p_confusion() {
                u += 1.0;
                p = Point3::new(u, u + 1.0, u + 2.0);
            }
            x_vec.crossed(Vector3::between(o, p))
        } else {
            x_vec.crossed(Vector3::from(line.direction()))
        };
    Ok(Plane3::from_position(Axis3::new(
        o,
        Direction3::from_vector(n_vec)?,
        Direction3::from_vector(x_vec)?,
    )?))
}

/// `ProjectIntersectAndEstLim`: projects the line and its mirror about the X axis
/// of `pln` on it, intersects both with the projected basis curve and widens
/// `[vmin, vmax]` by the parameters on that curve; returns NoIntersection (neither
/// projection meets it); `project_intersect_and_est_lim(l, &pln, &proj, &mut a, &mut b)?`.
pub fn project_intersect_and_est_lim(
    line: Line3,
    pln: &Plane3,
    bas_curv_proj: &ProjLibPlane,
    vmin: &mut f64,
    vmax: &mut f64,
) -> Result<bool, Failure> {
    let line_proj = ProjLibPlane::with_line(pln, &line)?;
    if !line_proj.is_done() {
        return Ok(false);
    }
    let lin2d: Line2 = line_proj.line()?;
    let p1: Point2 = lin2d.location();
    let p2 = Point2::from(p1.xy() + lin2d.direction().xy());
    let (p1sym, p2sym) = (Point2::new(p1.x(), -p1.y()), Point2::new(p2.x(), -p2.y()));
    let lin2d_sym = Line2::new(
        p1sym,
        Direction2::from_vector(Vector2::between(p1sym, p2sym))?,
    );
    let (con, con_sym) = (Conic::from_line(lin2d), Conic::from_line(lin2d_sym));
    let (inter, inter_sym) = match bas_curv_proj.get_type() {
        CurveType::Line => (
            AnaIntersection::line_conic(bas_curv_proj.line()?, &con)?,
            AnaIntersection::line_conic(bas_curv_proj.line()?, &con_sym)?,
        ),
        CurveType::Hyperbola => (
            AnaIntersection::hyperbola_conic(bas_curv_proj.hyperbola()?, &con)?,
            AnaIntersection::hyperbola_conic(bas_curv_proj.hyperbola()?, &con_sym)?,
        ),
        CurveType::Parabola => (
            AnaIntersection::parabola_conic(bas_curv_proj.parabola()?, &con)?,
            AnaIntersection::parabola_conic(bas_curv_proj.parabola()?, &con_sym)?,
        ),
        _ => return Ok(false),
    };
    widen_by_points(&inter, &inter_sym, vmin, vmax)
}

/// Widens `[vmin, vmax]` by the first parameters of both intersections, point by
/// point in turn; true when neither has a point.
fn widen_by_points(
    inter: &AnaIntersection,
    inter_sym: &AnaIntersection,
    vmin: &mut f64,
    vmax: &mut f64,
) -> Result<bool, Failure> {
    let nb = |i: &AnaIntersection| -> Result<i32, Failure> {
        if i.is_done() { i.nb_points() } else { Ok(0) }
    };
    let (nb_int, nb_int_sym) = (nb(inter)?, nb(inter_sym)?);
    let nb_pnt: i32 = nb_int.max(nb_int_sym);
    if nb_pnt == 0 {
        return Ok(true);
    }
    for i in 1..=nb_pnt {
        for (source, count) in [(inter, nb_int), (inter_sym, nb_int_sym)] {
            if i <= count {
                let param: f64 = source.point(i)?.param_on_first();
                *vmin = std_min(*vmin, param);
                *vmax = std_max(*vmax, param);
            }
        }
    }
    Ok(false)
}
