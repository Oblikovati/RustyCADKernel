//! The chamfers between a plane and a cylinder along a line of the cylinder.
use super::common::{
    frame_normal, fwd_if, line2, line3, oriented, set_interference, set_orientation, set_surf,
};
use crate::chfi_ds::{ChamfMode, SurfDataHandle};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis1, Axis3, Cylinder3, Direction2, Direction3, Line3, Plane3, Point2, Point3, Vector3,
};

/// The axis point of the cylinder nearest to the spine origin.
fn cylinder_axis_point(cyl: &Cylinder3, or_spine: Point3) -> Point3 {
    let ax = cyl.position();
    let (loc, dir) = (ax.location().xyz(), ax.direction().xyz());
    let parameter = (or_spine.xyz() - loc).dot(dir);
    Point3::from(loc + dir * parameter)
}

/// The translation on the plane from the spine toward the material of the cylinder.
fn plane_translation(
    cyl: &Cylinder3,
    (xdir, nor_pln, or2): (Direction3, Direction3, Orientation),
    or_spine: Point3,
    or_cyl: Point3,
) -> Result<Vector3, Failure> {
    let mut tmp = Vector3::between(or_spine, or_cyl);
    let direct = cyl.direct()?;
    if (or2 == Orientation::Forward && direct) || (or2 == Orientation::Reversed && !direct) {
        tmp.reverse();
    }
    let mut v = Vector3::from(xdir.crossed(nor_pln)?);
    if v.dot(tmp) <= 0. {
        v.reverse();
    }
    Ok(v)
}

/// The data of a line spine chamfer end: the spine origin and direction, the points on
/// the plane and the cylinder with their parameters.
struct LineChamfer {
    or_spine: Point3,
    xdir: Direction3,
    uv_pln: (f64, f64),
    uv_cyl: (f64, f64),
    p_on_cyl: Point3,
}

/// The chamfer plane through the points on the plane and the cylinder (the shared end of
/// the line spine chamfers), the transitions flipped by the side of the chamfer when
/// `flips` allow.
#[allow(clippy::too_many_arguments)]
fn line_chamfer(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, cyl): (&Plane3, &Cylinder3),
    lc: &LineChamfer,
    ofpl: Orientation,
    plandab: bool,
    (flip_pln, flip_cyl): (bool, bool),
) -> Result<(), Failure> {
    let (ax_pln, ax_cyl, xdir) = (pln.position(), cyl.position(), lc.xdir);
    let nor_pln = frame_normal(&ax_pln)?;
    let p_on_pln = elslib::plane_value(lc.uv_pln.0, lc.uv_pln.1, ax_pln);
    let mut ydir = Vector3::between(p_on_pln, lc.p_on_cyl);
    if !plandab {
        ydir.reverse();
    }
    let ax_ch = Axis3::new(
        p_on_pln,
        xdir.crossed(Direction3::from_vector(ydir)?)?,
        xdir,
    )?;
    set_surf(data, Handle::new(GeomPlane::new(ax_ch)).cast(), dstr);
    let pc_pln = line2(
        Point2::new(lc.uv_pln.0, lc.uv_pln.1),
        Direction2::new(
            xdir.dot(ax_pln.x_direction()),
            xdir.dot(ax_pln.y_direction()),
        )?,
    );
    let p_on_pln = elslib::value(lc.uv_pln.0, lc.uv_pln.1, pln);
    let pc_ch = |p: Point3| {
        let (u, v) = elslib::plane_parameters(ax_ch, p);
        line2(Point2::new(u, v), Direction2::X)
    };
    let nor_fil = ax_ch.direction();
    let dir_pln_cyl = Direction3::from_vector(Vector3::between(p_on_pln, lc.p_on_cyl))?;
    let pos_pln = dir_pln_cyl.dot(Direction3::from_vector(Vector3::between(
        lc.or_spine,
        p_on_pln,
    ))?) > 0.;
    let toreverse = (nor_fil.dot(nor_pln) <= 0.) != (flip_pln && pos_pln);
    set_orientation(
        data,
        if toreverse {
            top_abs::reverse(ofpl)
        } else {
            ofpl
        },
    );
    set_interference(
        data,
        if plandab { 1 } else { 2 },
        &line3(p_on_pln, xdir),
        fwd_if(toreverse == plandab),
        (&pc_pln, &pc_ch(p_on_pln)),
        dstr,
    );
    let mut vcyl = Direction2::Y;
    if xdir.dot(ax_cyl.direction()) < 0. {
        vcyl.reverse();
    }
    let p_on_cyl = elslib::value(lc.uv_cyl.0, lc.uv_cyl.1, cyl);
    let (_, deru, derv) = elslib::cylinder_d1(lc.uv_cyl.0, lc.uv_cyl.1, ax_cyl, cyl.radius());
    let nor_cyl = Direction3::from_vector(deru.crossed(derv))?;
    let pos_cyl = dir_pln_cyl.dot(Direction3::from_vector(Vector3::between(
        lc.or_spine,
        p_on_cyl,
    ))?) < 0.;
    let toreverse = (nor_fil.dot(nor_cyl) <= 0.) != (flip_cyl && pos_cyl);
    let pc_cyl = line2(Point2::new(lc.uv_cyl.0, lc.uv_cyl.1), vcyl);
    set_interference(
        data,
        if plandab { 2 } else { 1 },
        &line3(p_on_cyl, xdir),
        fwd_if(toreverse != plandab),
        (&pc_cyl, &pc_ch(p_on_cyl)),
        dstr,
    );
    Ok(())
}

/// `ChFiKPart_MakeChamfer(DStr, Data, Mode, Pln, Cyl, fu, lu, Or1, Or2, Dis1, Dis2,
/// Spine(line), First, Ofpl, plandab)`: the plane of the classic chamfer of a plane and
/// a cylinder of parallel axes (false for the other modes);
/// `make_chamfer_pln_cyl_line(&mut ds, &data, mode, (&pln, &cyl), (or1, or2), (1.0, 1.0), &spine, 0.0, ofpl, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_chamfer_pln_cyl_line(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    mode: ChamfMode,
    (pln, cyl): (&Plane3, &Cylinder3),
    (or1, or2): (Orientation, Orientation),
    (dis1, dis2): (f64, f64),
    spine: &Line3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
) -> Result<bool, Failure> {
    if mode != ChamfMode::ClassicChamfer {
        return Ok(false);
    }
    let or_spine = elclib::value(first, spine);
    let xdir = spine.direction();
    let nor_pln = frame_normal(&pln.position())?;
    let nor_f = oriented(nor_pln, or1);
    let or_cyl = cylinder_axis_point(cyl, or_spine);
    let v_pln = plane_translation(cyl, (xdir, nor_pln, or2), or_spine, or_cyl)? * dis1;
    let p_on_pln = Point3::from(or_spine.xyz() + *v_pln.xyz());
    let alpha = 2. * (dis2 * 0.5 / cyl.radius()).asin();
    let mut vct = Vector3::between(or_cyl, or_spine);
    let axis_dir = if xdir.crossed(Direction3::from_vector(vct)?)?.dot(nor_f) > 0. {
        xdir
    } else {
        xdir.reversed()
    };
    vct.rotate(Axis1::new(or_cyl, axis_dir), alpha)?;
    let uv_cyl = elslib::parameters(cyl, Point3::from(or_cyl.xyz() + *vct.xyz()));
    let p_on_cyl = elslib::cylinder_value(uv_cyl.0, uv_cyl.1, cyl.position(), cyl.radius());
    let lc = LineChamfer {
        or_spine,
        xdir,
        uv_pln: elslib::parameters(pln, p_on_pln),
        uv_cyl,
        p_on_cyl,
    };
    line_chamfer(dstr, data, (pln, cyl), &lc, ofpl, plandab, (true, true))?;
    Ok(true)
}

/// The distance on the cylinder of the line spine asymmetric chamfer with the distance
/// on the plane (None when the chamfer cannot be built).
fn asym_cylinder_distance(
    dis: f64,
    angle: f64,
    (cos_a1, sin_a1): (f64, f64),
    ray: f64,
) -> Option<f64> {
    let (sin_al, cos_al) = (angle.sin(), angle.cos());
    let h = dis * sin_al;
    let cos_ah = cos_a1 * sin_al + sin_a1 * cos_al;
    let mut sin_ah = sin_a1 * sin_al - cos_a1 * cos_al;
    if cos_a1 > 0. {
        sin_ah = -sin_ah;
    }
    let (t1, t2) = (h / ray, sin_ah * sin_ah + (h / ray) * cos_ah);
    let d = t2 + t1 * (cos_ah - t1);
    if d < -1.0e-09 {
        return None;
    }
    Some(if d < 1.0e-09 {
        ray * (2. * t2).sqrt()
    } else {
        ray * (2. * (t2 - sin_ah * d.sqrt())).sqrt()
    })
}

/// The distance on the plane of the line spine asymmetric chamfer with the distance on
/// the cylinder.
fn asym_plane_distance(
    p_on_cyl: Point3,
    or_spine: Point3,
    (du, dv, xdir): (Vector3, Vector3, Direction3),
    angle: f64,
    (cos_a1, sin_a1, dis2, ray): (f64, f64, f64, f64),
) -> Result<f64, Failure> {
    let corde = Vector3::between(p_on_cyl, or_spine).normalized()?;
    let tcyl =
        Vector3::from(xdir.crossed(Direction3::from_vector(du.crossed(dv))?)?).normalized()?;
    let cos_ct = tcyl.dot(corde);
    let tg_ct = (1. / (cos_ct * cos_ct) - 1.).sqrt();
    let mut tg_ang = angle.tan();
    tg_ang = (tg_ang + tg_ct) / (1. - tg_ang * tg_ct);
    let cos_a11 = dis2 / (2. * ray);
    let mut sin_a11 = (1. - cos_a11 * cos_a11).sqrt();
    if cos_a1 > 0. {
        sin_a11 = -sin_a11;
    }
    let mut d = (sin_a1 + cos_a1 * tg_ang) * cos_a11;
    d -= (cos_a1 - sin_a1 * tg_ang) * sin_a11;
    Ok((dis2 * tg_ang) / d)
}

/// `ChFiKPart_MakeChAsym(DStr, Data, Pln, Cyl, fu, lu, Or1, Or2, Dis, Angle, Spine(line),
/// First, Ofpl, plandab, DisOnP)`: the plane of the chamfer at a distance and an angle
/// of a plane and a cylinder of parallel axes; false when it cannot be built;
/// `make_ch_asym_pln_cyl_line(&mut ds, &data, (&pln, &cyl), (or1, or2), (1.0, 0.5), &spine, 0.0, ofpl, true, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_ch_asym_pln_cyl_line(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, cyl): (&Plane3, &Cylinder3),
    (or1, or2): (Orientation, Orientation),
    (dis, angle): (f64, f64),
    spine: &Line3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
    dis_on_p: bool,
) -> Result<bool, Failure> {
    let or_spine = elclib::value(first, spine);
    let xdir = spine.direction();
    let nor_pln = frame_normal(&pln.position())?;
    let nor_f = oriented(nor_pln, or1);
    let or_cyl = cylinder_axis_point(cyl, or_spine);
    let v_pln = plane_translation(cyl, (xdir, nor_pln, or2), or_spine, or_cyl)?;
    let vct = Vector3::between(or_spine, or_cyl);
    let cos_a1 = vct.normalized()?.dot(v_pln.normalized()?);
    let sin_a1 = (1. - cos_a1 * cos_a1).sqrt();
    let ray = cyl.radius();
    let is_dis_on_p = plandab == dis_on_p;
    let dis2 = if is_dis_on_p {
        let Some(d) = asym_cylinder_distance(dis, angle, (cos_a1, sin_a1), ray) else {
            return Ok(false);
        };
        d
    } else {
        dis
    };
    let alpha = 2. * (dis2 * 0.5 / ray).asin();
    let mut vtemp = vct.reversed();
    let axis_dir = if xdir.crossed(Direction3::from_vector(vct)?)?.dot(nor_f) < 0. {
        xdir
    } else {
        xdir.reversed()
    };
    vtemp.rotate(Axis1::new(or_cyl, axis_dir), alpha)?;
    let uv_cyl = elslib::parameters(cyl, Point3::from(or_cyl.xyz() + *vtemp.xyz()));
    let (p_on_cyl, du, dv) = elslib::cylinder_d1(uv_cyl.0, uv_cyl.1, cyl.position(), ray);
    let dis1 = if is_dis_on_p {
        dis
    } else {
        asym_plane_distance(
            p_on_cyl,
            or_spine,
            (du, dv, xdir),
            angle,
            (cos_a1, sin_a1, dis2, ray),
        )?
    };
    let p_on_pln = Point3::from(or_spine.xyz() + *(v_pln * dis1).xyz());
    let lc = LineChamfer {
        or_spine,
        xdir,
        uv_pln: elslib::parameters(pln, p_on_pln),
        uv_cyl,
        p_on_cyl,
    };
    line_chamfer(
        dstr,
        data,
        (pln, cyl),
        &lc,
        ofpl,
        plandab,
        (!is_dis_on_p, is_dis_on_p),
    )?;
    Ok(true)
}
