// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart/ChFiKPart_ComputeData_ChPlnCon.cxx
// and ChFiKPart_ComputeData_ChAsymPlnCon.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfers between a plane and a cone along their circle: a cone (a cylinder when
//! the chamfer is parallel to the axis).
use super::ch_pln_cyl::{CircleChamferFrame, circle_chamfer_frame, plane_circle};
use super::common::{frame_normal, fwd_if, line2, set_interference, set_orientation, set_surf};
use crate::chfi_ds::{ChamfMode, SurfDataHandle};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomElementarySurface,
    GeomSurface,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Cone3, Direction2, Direction3, Plane3, Point2, Point3, Vector3,
};
use std::f64::consts::PI;

/// `ChFiKPart_MakeChamfer(DStr, Data, Mode, Pln, Con, fu, lu, Or1, Or2, Dis1, Dis2, Spine,
/// First, Ofpl, plandab)`: the distances (adjusted for the throat modes) turned into a
/// distance and an angle for `MakeChAsym` (distance on the cone);
/// `make_chamfer_pln_con(&mut ds, &data, mode, (&pln, &con), (fu, lu), (or1, or2), (1.0, 1.0), &circ, 0.0, ofpl, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_chamfer_pln_con(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    mode: ChamfMode,
    (pln, con): (&Plane3, &Cone3),
    range: (f64, f64),
    (or1, or2): (Orientation, Orientation),
    (the_dis1, the_dis2): (f64, f64),
    spine: &Circle3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
) -> Result<bool, Failure> {
    let angcon = con.semi_angle();
    let alpha = PI / 2. - angcon;
    let (dis1, dis2) = match mode {
        ChamfMode::ConstThroatChamfer => {
            let d = the_dis1 / (alpha / 2.).cos();
            (d, d)
        }
        ChamfMode::ConstThroatWithPenetrationChamfer => {
            let (a1, a2) = (the_dis1.min(the_dis2), the_dis1.max(the_dis2));
            let (d11, d22) = (a1 * a1, a2 * a2);
            let (sin_a, cos_a) = (alpha.sin(), alpha.cos());
            let d1 = (d22 - d11).sqrt() - a1 * (cos_a / sin_a);
            let cos_beta = (1. - d11 / d22).sqrt() * cos_a + a1 / a2 * sin_a;
            (d1, a2 / cos_beta - a1 / sin_a)
        }
        ChamfMode::ClassicChamfer => (the_dis1, the_dis2),
    };
    let sincon = angcon.sin().abs();
    let fr = circle_chamfer_frame(pln, con, con.location(), (or1, or2, ofpl), spine, first)?;
    let ouvert = fr.dpl.dot(fr.dsurf) >= 0.;
    let angle = if ouvert {
        ((dis1 * angcon.cos()) / (dis2 + dis1 * sincon)).atan()
    } else if (dis1 - dis2 * sincon).abs() > precision::confusion() {
        ((dis1 * angcon.cos()) / (dis2 - dis1 * sincon).abs()).atan()
    } else {
        angcon
    };
    make_ch_asym_pln_con(
        dstr,
        data,
        (pln, con),
        range,
        (or1, or2),
        (dis2, angle),
        spine,
        first,
        ofpl,
        plandab,
        false,
    )
}

/// The geometry of the asymmetric cone chamfer: the axis shift, the distance on the
/// plane side, the radius and the semi-angle of the chamfer, and its kind.
struct AsymCone {
    mv: f64,
    dis: f64,
    chamf_rad: f64,
    semi: f64,
    pointu: bool,
    iscylinder: bool,
    is_con_par: bool,
}

/// The chamfer with the distance on the plane: None when it does not fit inside or is
/// steeper than the cone.
fn asym_on_plane(
    dis: f64,
    angle: f64,
    ang_con: f64,
    spine_r: f64,
    (ouvert, dedans): (bool, bool),
) -> Option<AsymCone> {
    let (tgang, tgcon) = (angle.tan(), ang_con.tan().abs());
    let mv = if ouvert {
        dis * tgang / (1. - tgcon * tgang)
    } else {
        dis * tgang / (1. + tgcon * tgang)
    };
    let d = if ouvert {
        dis + mv * tgcon
    } else {
        dis - mv * tgcon
    };
    let (chamf_rad, semi) = if dedans {
        (spine_r - dis, PI / 2. - angle)
    } else {
        (spine_r + dis, angle - PI / 2.)
    };
    if dedans && chamf_rad < 0. {
        return None;
    }
    if ouvert && ang_con.abs() - semi.abs() > -precision::confusion() {
        return None;
    }
    Some(AsymCone {
        mv,
        dis: d,
        chamf_rad,
        semi,
        pointu: dedans && chamf_rad.abs() < precision::confusion(),
        iscylinder: false,
        is_con_par: false,
    })
}

/// The chamfer with the distance on the cone: None when it does not fit inside or is
/// beyond the plane.
fn asym_on_cone(
    dis: f64,
    angle: f64,
    ang_con: f64,
    spine_r: f64,
    (ouvert, dedans): (bool, bool),
) -> Option<AsymCone> {
    let mv = dis * ang_con.cos();
    let s = dis * ang_con.sin().abs();
    let (mut semi, mut iscylinder, mut is_con_par, dis1);
    (iscylinder, is_con_par) = (false, false);
    if ouvert {
        semi = ang_con.abs() + angle;
        if PI / 2. - semi < precision::confusion() {
            return None;
        }
        dis1 = mv * semi.tan() - s;
        if !dedans {
            semi = -semi;
        }
    } else {
        semi = ang_con.abs() - angle;
        if semi.abs() < precision::confusion() {
            iscylinder = true;
            dis1 = s;
        } else {
            dis1 = s - mv * semi.tan();
        }
        if semi > precision::confusion() {
            is_con_par = true;
        }
        if dedans {
            semi = -semi;
        }
    }
    let chamf_rad = if dedans {
        spine_r - dis1
    } else {
        spine_r + dis1
    };
    if dedans && chamf_rad < 0. {
        return None;
    }
    let d = if ouvert { dis1 + s } else { dis1 - s };
    Some(AsymCone {
        mv,
        dis: d,
        chamf_rad,
        semi,
        pointu: dedans && chamf_rad.abs() < precision::confusion(),
        iscylinder,
        is_con_par,
    })
}

/// The parameter of the chamfer circle on the cone (0 at the period end, then in
/// `[fu, lu]`), the cone derivatives there and the pcurve direction along the spine.
fn cone_side(
    con: &Cone3,
    p: Point3,
    (fu, lu): (f64, f64),
    dsp: Vector3,
) -> (f64, f64, Vector3, Vector3, Direction2) {
    let (mut u, v) = elslib::parameters(con, p);
    let tol = precision::p_confusion();
    if u >= 2. * PI - tol && u <= 2. * PI {
        u = 0.;
    }
    if u >= fu - tol && u < fu {
        u = fu;
    }
    if u <= lu + tol && u > lu {
        u = lu;
    }
    if u < fu || u > lu {
        u = elclib::in_period(u, fu, fu + 2. * PI);
    }
    let (_, deru, derv) = elslib::d1(u, v, con);
    let d2d = if deru.dot(dsp) <= 0. {
        Direction2::X.reversed()
    } else {
        Direction2::X
    };
    (u, v, deru, derv, d2d)
}

/// The parameters, derivatives and pcurve direction of a point of the cone.
type ConeFrame = (f64, f64, Vector3, Vector3, Direction2);

/// The circle of the chamfer on the cone and its pcurve on the cone, from the chamfer
/// frame shifted along the plane normal.
fn cone_circle(
    con: &Cone3,
    fr: &CircleChamferFrame,
    (cir_ax2, or, rad): (Axis2, Point3, f64),
    range: (f64, f64),
) -> Result<(Handle<dyn GeomCurve>, ConeFrame), Failure> {
    let mut ax2 = cir_ax2;
    ax2.set_location(or);
    let gc: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(ax2, rad)?)).cast();
    let p = Point3::new(
        or.x() + rad * fr.dx.x(),
        or.y() + rad * fr.dx.y(),
        or.z() + rad * fr.dx.z(),
    );
    Ok((gc, cone_side(con, p, range, fr.dsp)))
}

/// The chamfer cylinder (the chamfer parallel to the axis).
#[allow(clippy::too_many_arguments)]
fn asym_cylinder(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, con): (&Plane3, &Cone3),
    range: (f64, f64),
    fr: &CircleChamferFrame,
    ac: &AsymCone,
    or: Point3,
    (plandab, dedans): (bool, bool),
) -> Result<(), Failure> {
    let mut ax = Axis3::new(fr.pt_pl, fr.dpl, fr.dx)?;
    let gcyl = GeomCylindricalSurface::new(ax, ac.chamf_rad)?;
    if !plandab {
        gcyl.v_reverse()?;
        ax = gcyl.position();
    }
    if Vector3::from(ax.y_direction()).dot(fr.dsp) <= 0. {
        ax.y_reverse();
        gcyl.set_position(ax);
    }
    set_surf(data, Handle::new(gcyl).cast(), dstr);
    let mut cylaxe = ax.axis().direction();
    if !ax.direct()? != plandab {
        cylaxe.reverse();
    }
    set_orientation(data, fwd_if((fr.norf.dot(cylaxe) < 0.) != dedans));
    let mut cir_ax2 = ax.ax2()?;
    cir_ax2.set_location(fr.pt_pl);
    let pt = Point3::new(
        fr.pt_pl.x() + ac.chamf_rad * fr.dx.x(),
        fr.pt_pl.y() + ac.chamf_rad * fr.dx.y(),
        fr.pt_pl.z() + ac.chamf_rad * fr.dx.z(),
    );
    let (gc, gc2d) = plane_circle(fr, cir_ax2, ac.chamf_rad, pt, false)?;
    let toreverse = (frame_normal(&pln.position())?.dot(cylaxe) < 0.) == plandab;
    let trans = fwd_if(toreverse == dedans);
    set_interference(
        data,
        if plandab { 1 } else { 2 },
        &gc,
        trans,
        (&gc2d, &line2(Point2::new(0., 0.), Direction2::X)),
        dstr,
    );
    let rad = if dedans {
        ac.chamf_rad + ac.dis
    } else {
        ac.chamf_rad - ac.dis
    };
    let v = (ac.dis * ac.dis + ac.mv * ac.mv).sqrt() * if plandab { 1. } else { -1. };
    let (gcc, (u, vc, deru, derv, d2d)) = cone_circle(con, fr, (cir_ax2, or, rad), range)?;
    let norcon = Direction3::from_vector(deru.crossed(derv))?;
    let mut dir_con = con.axis().direction();
    if con.semi_angle() > precision::confusion() {
        dir_con.reverse();
    }
    let trans = fwd_if((norcon.dot(dir_con) < 0.) != dedans);
    set_interference(
        data,
        if plandab { 2 } else { 1 },
        &gcc,
        trans,
        (
            &line2(Point2::new(u, vc), d2d),
            &line2(Point2::new(0., v), Direction2::X),
        ),
        dstr,
    );
    Ok(())
}

/// The chamfer cone.
#[allow(clippy::too_many_arguments)]
fn asym_cone(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, con): (&Plane3, &Cone3),
    range: (f64, f64),
    fr: &CircleChamferFrame,
    ac: &AsymCone,
    or: Point3,
    (plandab, dedans): (bool, bool),
) -> Result<(), Failure> {
    let mut ax = Axis3::new(fr.pt_pl, fr.dpl, fr.dx)?;
    let gcon = GeomConicalSurface::new(ax, ac.semi, ac.chamf_rad)?;
    let mut semi = ac.semi;
    if !plandab {
        gcon.v_reverse()?;
        ax = gcon.position();
        semi = gcon.semi_angle();
    }
    if Vector3::from(ax.y_direction()).dot(fr.dsp) <= 0. {
        ax.y_reverse();
        gcon.set_position(ax);
    }
    let dir_chamf_axis = gcon.axis().direction();
    set_surf(data, Handle::new(gcon).cast(), dstr);
    let v = (ac.dis * ac.dis + ac.mv * ac.mv).sqrt() * if plandab { 1. } else { -1. };
    let (_, deru, derv) = elslib::cone_d1(0., v, ax, ac.chamf_rad, semi);
    let mut norchamf = Direction3::from_vector(deru.crossed(derv))?;
    set_orientation(data, fwd_if((fr.norf.dot(norchamf) < 0.) == ac.is_con_par));
    let mut cir_ax2 = ax.ax2()?;
    cir_ax2.set_location(fr.pt_pl);
    let pt = Point3::new(
        fr.pt_pl.x() + ac.chamf_rad * fr.dx.x(),
        fr.pt_pl.y() + ac.chamf_rad * fr.dx.y(),
        fr.pt_pl.z() + ac.chamf_rad * fr.dx.z(),
    );
    let (gc, gc2d) = plane_circle(fr, cir_ax2, ac.chamf_rad, pt, ac.pointu)?;
    let (_, deru, derv) = elslib::cone_d1(0., 0., ax, ac.chamf_rad, semi);
    if !ac.pointu {
        norchamf = Direction3::from_vector(deru.crossed(derv))?;
    }
    let toreverse = (norchamf.dot(frame_normal(&pln.position())?) <= 0.) != ac.is_con_par;
    set_interference(
        data,
        if plandab { 1 } else { 2 },
        &gc,
        fwd_if(toreverse == plandab),
        (&gc2d, &line2(Point2::new(0., 0.), Direction2::X)),
        dstr,
    );
    let rad = if dedans {
        ac.chamf_rad + ac.dis
    } else {
        ac.chamf_rad - ac.dis
    };
    let (_, deru, derv) = elslib::cone_d1(0., v, ax, ac.chamf_rad, semi);
    let norchamf = Direction3::from_vector(deru.crossed(derv))?;
    let (gcc, (u, vc, deru, derv, d2d)) = cone_circle(con, fr, (cir_ax2, or, rad), range)?;
    let norcon = Direction3::from_vector(deru.crossed(derv))?;
    let (mut dir_con, mut dir_chamf) = (con.axis().direction(), dir_chamf_axis);
    if con.semi_angle() > precision::confusion() {
        dir_con.reverse();
    }
    if semi > precision::confusion() {
        dir_chamf.reverse();
    }
    let toreverse = (norcon.dot(dir_con) > 0.) != (norchamf.dot(dir_chamf) > 0.);
    set_interference(
        data,
        if plandab { 2 } else { 1 },
        &gcc,
        fwd_if(toreverse != plandab),
        (
            &line2(Point2::new(u, vc), d2d),
            &line2(Point2::new(0., v), Direction2::X),
        ),
        dstr,
    );
    Ok(())
}

/// `ChFiKPart_MakeChAsym(DStr, Data, Pln, Con, fu, lu, Or1, Or2, Dis, Angle, Spine, First,
/// Ofpl, plandab, DisOnP)`: the cone (a cylinder when parallel to the axis) of the
/// chamfer of a plane and a cone at a distance and an angle; false when it does not fit;
/// `make_ch_asym_pln_con(&mut ds, &data, (&pln, &con), (fu, lu), (or1, or2), (1.0, 0.5), &circ, 0.0, ofpl, true, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_ch_asym_pln_con(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, con): (&Plane3, &Cone3),
    range: (f64, f64),
    (or1, or2): (Orientation, Orientation),
    (dis, angle): (f64, f64),
    spine: &Circle3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
    dis_on_p: bool,
) -> Result<bool, Failure> {
    let fr = circle_chamfer_frame(pln, con, con.location(), (or1, or2, ofpl), spine, first)?;
    let dedans = fr.dx.dot(fr.dsurf) <= 0.;
    let ouvert = fr.dpl.dot(fr.dsurf) >= 0.;
    let ang_con = con.semi_angle();
    let ac = if plandab == dis_on_p {
        asym_on_plane(dis, angle, ang_con, spine.radius(), (ouvert, dedans))
    } else {
        asym_on_cone(dis, angle, ang_con, spine.radius(), (ouvert, dedans))
    };
    let Some(ac) = ac else {
        return Ok(false);
    };
    let or = Point3::new(
        fr.pt_pl.x() + ac.mv * fr.dpl.x(),
        fr.pt_pl.y() + ac.mv * fr.dpl.y(),
        fr.pt_pl.z() + ac.mv * fr.dpl.z(),
    );
    if ac.iscylinder {
        asym_cylinder(
            dstr,
            data,
            (pln, con),
            range,
            &fr,
            &ac,
            or,
            (plandab, dedans),
        )?;
    } else {
        asym_cone(
            dstr,
            data,
            (pln, con),
            range,
            &fr,
            &ac,
            or,
            (plandab, dedans),
        )?;
    }
    Ok(true)
}
