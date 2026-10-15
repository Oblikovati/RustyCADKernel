// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart/ChFiKPart_ComputeData_FilPlnPln.cxx,
// ChFiKPart_ComputeData_ChPlnPln.cxx and ChFiKPart_ComputeData_ChAsymPlnPln.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillet (a cylinder) and the chamfers (a plane) between two planes along a line.
use super::common::{
    frame_normal, fwd_if, line2, line3, oriented, set_interference, set_orientation, set_surf,
};
use crate::chfi_ds::{ChamfMode, SurfDataHandle};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomPlane};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::int_ana::QuadQuadGeo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Line3, Plane3, Point2, Point3, Vector3};

/// The oriented normals of the planes.
fn plane_normals(
    pl1: &Plane3,
    pl2: &Plane3,
    or1: Orientation,
    or2: Orientation,
) -> Result<(Direction3, Direction3), Failure> {
    Ok((
        oriented(frame_normal(&pl1.position())?, or1),
        oriented(frame_normal(&pl2.position())?, or2),
    ))
}

/// The intersection line of the planes (None when not done).
fn planes_line(pl1: &Plane3, pl2: &Plane3) -> Result<Option<Line3>, Failure> {
    let mut lint = QuadQuadGeo::new();
    lint.perform_planes(*pl1, *pl2, precision::angular(), precision::confusion())?;
    if !lint.is_done() {
        return Ok(None);
    }
    Ok(Some(lint.line(1)?))
}

/// The point moved by `d` times a direction.
fn moved(p: Point3, d: f64, dir: Direction3) -> Point3 {
    Point3::new(
        p.x() + d * dir.x(),
        p.y() + d * dir.y(),
        p.z() + d * dir.z(),
    )
}

/// The pcurve on a plane of a line through `p` along `d`.
fn plane_line2d(
    pos: Axis3,
    p: Point3,
    d: Direction3,
) -> Result<Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>, Failure> {
    let (u, v) = elslib::plane_parameters(pos, p);
    Ok(line2(
        Point2::new(u, v),
        Direction2::new(d.dot(pos.x_direction()), d.dot(pos.y_direction()))?,
    ))
}

/// `ChFiKPart_MakeFillet(DStr, Data, Pl1, Pl2, Or1, Or2, Radius, Spine, First, Of1)`: the
/// cylinder of the fillet of two planes along the spine line, with its lines on the
/// planes; false when the planes do not cut;
/// `make_fillet_pln_pln(&mut ds, &data, (&p1, &p2), (or1, or2), 1.0, &spine, 0.0, of1)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_fillet_pln_pln(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pl1, pl2): (&Plane3, &Plane3),
    (or1, or2): (Orientation, Orientation),
    radius: f64,
    spine: &Line3,
    first: f64,
    of1: Orientation,
) -> Result<bool, Failure> {
    let (pos1, pos2) = (pl1.position(), pl2.position());
    let (d1, d2) = plane_normals(pl1, pl2, or1, or2)?;
    let Some(l) = planes_line(pl1, pl2)? else {
        return Ok(false);
    };
    let pv = elclib::value(elclib::parameter(&l, elclib::value(first, spine))?, &l);
    let axis_cylinder = spine.direction();
    let ang = d1.angle(&d2);
    let s = Direction3::from_vector(Vector3::from(d1) + Vector3::from(d2))?;
    let c = moved(pv, radius / (ang / 2.).cos(), s);
    let mut cyl_ax3 = Axis3::new(c, axis_cylinder, d1.reversed())?;
    if cyl_ax3.y_direction().dot(d2) >= 0. {
        cyl_ax3.y_reverse();
    }
    set_surf(
        data,
        Handle::new(GeomCylindricalSurface::new(cyl_ax3, radius)?).cast(),
        dstr,
    );
    let (p, deru, derv) = elslib::cylinder_d1(0., 0., cyl_ax3, radius);
    let norcyl = Direction3::from_vector(deru.crossed(derv))?;
    let norpl = frame_normal(&pos1)?;
    set_orientation(data, fwd_if(norcyl.dot(oriented(norpl, of1)) > 0.));
    let trans = fwd_if(norcyl.dot(norpl) > 0.);
    let pc_cyl = line2(Point2::new(0., 0.), Direction2::Y);
    set_interference(
        data,
        1,
        &line3(p, axis_cylinder),
        trans,
        (&plane_line2d(pos1, p, axis_cylinder)?, &pc_cyl),
        dstr,
    );
    let (p, deru, derv) = elslib::cylinder_d1(ang, 0., cyl_ax3, radius);
    let norcyl = Direction3::from_vector(deru.crossed(derv))?;
    let toreverse = norcyl.dot(frame_normal(&pos2)?) <= 0.;
    let pc_cyl = line2(Point2::new(ang, 0.), Direction2::Y);
    set_interference(
        data,
        2,
        &line3(p, axis_cylinder),
        fwd_if(toreverse),
        (&plane_line2d(pos2, p, axis_cylinder)?, &pc_cyl),
        dstr,
    );
    Ok(true)
}

/// The distances of the symmetric and two-distance chamfers by the mode and the angle
/// between the planes.
fn chamfer_distances(mode: ChamfMode, dis1: f64, dis2: f64, alpha: f64) -> (f64, f64) {
    match mode {
        ChamfMode::ConstThroatChamfer => {
            let d = dis1 / (alpha / 2.).cos();
            (d, d)
        }
        ChamfMode::ConstThroatWithPenetrationChamfer => {
            let (a1, a2) = (dis1.min(dis2), dis1.max(dis2));
            let (d11, d22) = (a1 * a1, a2 * a2);
            let (sin_a, cos_a) = (alpha.sin(), alpha.cos());
            let d1 = (d22 - d11).sqrt() - a1 * (cos_a / sin_a);
            let cos_beta = (1. - d11 / d22).sqrt() * cos_a + a1 / a2 * sin_a;
            (d1, a2 / cos_beta - a1 / sin_a)
        }
        _ => (dis1, dis2),
    }
}

/// The chamfer frame shared by the plane chamfers: the intersection point, its line
/// parameter and the line, the translations on the faces and the normals.
struct PlnPlnFrame {
    p: Point3,
    fint: f64,
    l: Line3,
    v1: Direction3,
    v2: Direction3,
    d2: Direction3,
}

/// The chamfer frame (None when the planes do not cut); `strict` reverses the
/// translations on a zero dot product too (the symmetric chamfer).
fn pln_pln_frame(
    pl1: &Plane3,
    pl2: &Plane3,
    (or1, or2): (Orientation, Orientation),
    spine: &Line3,
    first: f64,
    strict: bool,
) -> Result<Option<PlnPlnFrame>, Failure> {
    let (d1, d2) = plane_normals(pl1, pl2, or1, or2)?;
    let Some(l) = planes_line(pl1, pl2)? else {
        return Ok(None);
    };
    let fint = elclib::parameter(&l, elclib::value(first, spine))?;
    let p = elclib::value(fint, &l);
    let lin_ax1 = spine.direction();
    let reversed = |d: f64| if strict { d <= 0. } else { d < 0. };
    let mut v1 = lin_ax1.crossed(d1)?;
    if reversed(v1.dot(d2)) {
        v1.reverse();
    }
    let mut v2 = lin_ax1.crossed(d2)?;
    if reversed(v2.dot(d1)) {
        v2.reverse();
    }
    Ok(Some(PlnPlnFrame {
        p,
        fint,
        l,
        v1,
        v2,
        d2,
    }))
}

/// The chamfer plane through the points at the distances on both faces, with its
/// lines on the faces (the end of every plane chamfer).
#[allow(clippy::too_many_arguments)]
fn pln_pln_chamfer_surface(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pl1, pl2): (&Plane3, &Plane3),
    fr: &PlnPlnFrame,
    (dis1, dis2): (f64, f64),
    xdir: Direction3,
    of1: Orientation,
) -> Result<(), Failure> {
    let (pos1, pos2) = (pl1.position(), pl2.position());
    let (p1, p2) = (moved(fr.p, dis1, fr.v1), moved(fr.p, dis2, fr.v2));
    let po = Point3::new(
        (p1.x() + p2.x()) / 2.,
        (p1.y() + p2.y()) / 2.,
        (p1.z() + p2.z()) / 2.,
    );
    let p22 = moved(elclib::value(fr.fint + 10., &fr.l), dis2, fr.v2);
    let v1 = Direction3::from_vector(Vector3::between(p1, p2))?;
    let v2 = Direction3::from_vector(Vector3::between(p1, p22))?;
    let mut plan_ax3 = Axis3::new(po, v1.crossed(v2)?, xdir)?;
    if plan_ax3.y_direction().dot(fr.d2) >= 0. {
        plan_ax3.y_reverse();
    }
    let gpl = GeomPlane::new(plan_ax3);
    let norplch = frame_normal(&gpl.pln().position())?;
    set_surf(data, Handle::new(gpl).cast(), dstr);
    let norpl = frame_normal(&pos1)?;
    let dir_ch12 = Direction3::from_vector(Vector3::between(p1, p2))?;
    let flip1 = fr.v1.dot(dir_ch12) > 0.;
    set_orientation(
        data,
        fwd_if((norplch.dot(oriented(norpl, of1)) <= 0.) == flip1),
    );
    let pc_ch = |p: Point3| -> Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve> {
        let (u, v) = elslib::plane_parameters(plan_ax3, p);
        line2(Point2::new(u, v), Direction2::X)
    };
    let trans1 = fwd_if((norplch.dot(norpl) <= 0.) != flip1);
    set_interference(
        data,
        1,
        &line3(p1, xdir),
        trans1,
        (&plane_line2d(pos1, p1, xdir)?, &pc_ch(p1)),
        dstr,
    );
    let flip2 = fr.v2.dot(dir_ch12) < 0.;
    let trans2 = fwd_if((norplch.dot(frame_normal(&pos2)?) <= 0.) == flip2);
    set_interference(
        data,
        2,
        &line3(p2, xdir),
        trans2,
        (&plane_line2d(pos2, p2, xdir)?, &pc_ch(p2)),
        dstr,
    );
    Ok(())
}

/// `ChFiKPart_MakeChamfer(DStr, Data, Mode, Pl1, Pl2, Or1, Or2, Dis1, Dis2, Spine, First,
/// Of1)`: the plane of the chamfer of two planes along the spine line at the distances
/// (adjusted for the throat modes); false when the planes do not cut;
/// `make_chamfer_pln_pln(&mut ds, &data, mode, (&p1, &p2), (or1, or2), (1.0, 1.0), &spine, 0.0, of1)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_chamfer_pln_pln(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    mode: ChamfMode,
    planes: (&Plane3, &Plane3),
    ors: (Orientation, Orientation),
    (dis1, dis2): (f64, f64),
    spine: &Line3,
    first: f64,
    of1: Orientation,
) -> Result<bool, Failure> {
    let Some(fr) = pln_pln_frame(planes.0, planes.1, ors, spine, first, true)? else {
        return Ok(false);
    };
    let dis = chamfer_distances(mode, dis1, dis2, fr.v1.angle(&fr.v2));
    pln_pln_chamfer_surface(dstr, data, planes, &fr, dis, spine.direction(), of1)?;
    Ok(true)
}

/// `ChFiKPart_MakeChAsym(DStr, Data, Pl1, Pl2, Or1, Or2, Dis, Angle, Spine, First, Of1,
/// DisOnP1)`: the plane of the chamfer of two planes at the distance on one face and
/// the angle; false when the planes do not cut;
/// `make_ch_asym_pln_pln(&mut ds, &data, (&p1, &p2), (or1, or2), (1.0, 0.5), &spine, 0.0, of1, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_ch_asym_pln_pln(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    planes: (&Plane3, &Plane3),
    ors: (Orientation, Orientation),
    (dis, angle): (f64, f64),
    spine: &Line3,
    first: f64,
    of1: Orientation,
    dis_on_p1: bool,
) -> Result<bool, Failure> {
    let Some(fr) = pln_pln_frame(planes.0, planes.1, ors, spine, first, false)? else {
        return Ok(false);
    };
    let cos_p = fr.v1.dot(fr.v2);
    let sin_p = (1. - cos_p * cos_p).sqrt();
    let other = dis / (cos_p + sin_p / angle.tan());
    let dists = if dis_on_p1 {
        (dis, other)
    } else {
        (other, dis)
    };
    pln_pln_chamfer_surface(dstr, data, planes, &fr, dists, spine.direction(), of1)?;
    Ok(true)
}
