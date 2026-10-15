// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart/ChFiKPart_ComputeData_ChPlnCyl.cxx
// and ChFiKPart_ComputeData_ChAsymPlnCyl.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chamfers between a plane and a cylinder: a cone along a circle spine, a plane
//! along a line spine.
use super::common::{
    frame_normal, fwd_if, line2, oriented, set_interference, set_orientation, set_surf,
};
use super::fil_pln_cyl::revolution_side_u;
use crate::chfi_ds::{ChamfMode, SurfDataHandle};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomElementarySurface, GeomSurface,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis3, Axis22d, Circle2, Circle3, Cylinder3, Direction2, Direction3, Plane3, Point2, Point3,
    Vector2, Vector3,
};

/// The frame of a circle spine chamfer between a plane and a surface of revolution.
pub(super) struct CircleChamferFrame {
    pub(super) pos_pl: Axis3,
    pub(super) dpl: Direction3,
    pub(super) norf: Direction3,
    pub(super) pt_pl: Point3,
    pub(super) pt2d_pln: Point2,
    pub(super) dsp: Vector3,
    pub(super) dx: Direction3,
    pub(super) dy: Direction3,
    pub(super) dsurf: Direction3,
}

/// The circle spine frame: the plane normal (oriented), the projection of the axis
/// origin on the plane, the spine tangent, the radial direction and the normal of the
/// surface at the spine origin.
pub(super) fn circle_chamfer_frame<S: elslib::ElementarySurface>(
    pln: &Plane3,
    surf: &S,
    location: Point3,
    (or1, or2, ofpl): (Orientation, Orientation, Orientation),
    spine: &Circle3,
    first: f64,
) -> Result<CircleChamferFrame, Failure> {
    let pos_pl = pln.position();
    let n = frame_normal(&pos_pl)?;
    let (u, v) = elslib::plane_parameters(pos_pl, location);
    let pt_pl = elslib::plane_d0(u, v, pos_pl);
    let (pt_sp, dsp) = elclib::d1(first, spine);
    let dx = Direction3::from_vector(Vector3::between(pt_pl, pt_sp))?;
    let (us, vs) = elslib::parameters(surf, pt_sp);
    let (_, vu, vv) = elslib::d1(us, vs, surf);
    let dsurf = oriented(Direction3::from_vector(vu.crossed(vv))?, or2);
    Ok(CircleChamferFrame {
        pos_pl,
        dpl: oriented(n, or1),
        norf: oriented(n, ofpl),
        pt_pl,
        pt2d_pln: Point2::new(u, v),
        dsp,
        dx,
        dy: Direction3::from_vector(dsp)?,
        dsurf,
    })
}

/// A 3d curve with its pcurve.
type CurveAndPCurve = (Handle<dyn GeomCurve>, Handle<dyn Geom2dCurve>);

/// The circle of the chamfer on the plane (none when the chamfer closes on the axis)
/// and its pcurve on the plane.
pub(super) fn plane_circle(
    fr: &CircleChamferFrame,
    cir_ax2: rustycad_tkmath::gp::Axis2,
    rad: f64,
    pt: Point3,
    pointu: bool,
) -> Result<CurveAndPCurve, Failure> {
    if pointu {
        return Ok((Handle::default(), Handle::default()));
    }
    let gc: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(cir_ax2, rad)?)).cast();
    let (u, v) = elslib::plane_parameters(fr.pos_pl, pt);
    let d2d = Direction2::new(
        fr.dsp.dot(Vector3::from(fr.pos_pl.x_direction())),
        fr.dsp.dot(Vector3::from(fr.pos_pl.y_direction())),
    )?;
    let ax2d = Axis22d::with_directions(
        fr.pt2d_pln,
        Direction2::from_vector(Vector2::between(fr.pt2d_pln, Point2::new(u, v)))?,
        d2d,
    )?;
    Ok((
        gc,
        Handle::new(Geom2dCircle::from_circ2d(Circle2::new(ax2d, rad)?)).cast(),
    ))
}

/// The chamfer cone of a circle spine: built on `(or, dpl, dx)`, v-reversed when the
/// chamfer runs inward from the plane, its Y along the spine.
fn chamfer_cone(
    or: Point3,
    (dpl, dx, dsp): (Direction3, Direction3, Vector3),
    semi: f64,
    con_rad: f64,
    v_rev: bool,
) -> Result<(GeomConicalSurface, Axis3, f64), Failure> {
    let mut ax = Axis3::new(or, dpl, dx)?;
    let gcon = GeomConicalSurface::new(ax, semi, con_rad)?;
    let mut semi = semi;
    if v_rev {
        gcon.v_reverse()?;
        ax = gcon.position();
        semi = gcon.semi_angle();
    }
    if Vector3::from(ax.y_direction()).dot(dsp) <= 0. {
        ax.y_reverse();
        gcon.set_position(ax);
    }
    Ok((gcon, ax, semi))
}

/// The circle spine chamfer of a plane and a cylinder at the distances (the shared end
/// of the symmetric and asymmetric chamfers); false when it does not fit inside.
#[allow(clippy::too_many_arguments)]
fn circle_chamfer(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    cyl: &Cylinder3,
    (fu, lu): (f64, f64),
    fr: &CircleChamferFrame,
    (dis1, dis2): (f64, f64),
    plandab: bool,
) -> Result<bool, Failure> {
    let dedans = fr.dsurf.dot(fr.dx) <= 0.;
    let or = Point3::new(
        fr.pt_pl.x() + dis2 * fr.dpl.x(),
        fr.pt_pl.y() + dis2 * fr.dpl.y(),
        fr.pt_pl.z() + dis2 * fr.dpl.z(),
    );
    let vec1 = Direction3::from_vector(Vector3::between(fr.pt_pl, or))?;
    let xd = fr.pos_pl.x_direction();
    let pt = Point3::new(
        or.x() + dis1 * xd.x(),
        or.y() + dis1 * xd.y(),
        or.z() + dis1 * xd.z(),
    );
    let vec2 = Direction3::from_vector(Vector3::between(fr.pt_pl, pt))?;
    let (mut pointu, mut dpl) = (false, fr.dpl);
    let rad = if dedans {
        cyl.radius() - dis1
    } else {
        cyl.radius() + dis1
    };
    if dedans {
        if rad.abs() <= precision::confusion() {
            pointu = true;
        }
        if rad < 0. {
            return Ok(false);
        }
    } else {
        dpl = dpl.reversed();
    }
    let con_rad = cyl.radius();
    let (gcon, con_ax3, semi) = chamfer_cone(
        or,
        (dpl, fr.dx, fr.dsp),
        vec1.angle(&vec2),
        con_rad,
        dedans != plandab,
    )?;
    set_surf(data, Handle::new(gcon).cast(), dstr);
    let (pt, deru, derv) = elslib::cone_d1(0., 0., con_ax3, con_rad, semi);
    let nor_con = Direction3::from_vector(deru.crossed(derv))?;
    set_orientation(data, fwd_if(nor_con.dot(fr.norf) > 0.));
    let mut cir_ax2 = con_ax3.ax2()?;
    cir_ax2.set_location(fr.pt_pl);
    let (gc, gc2d) = plane_circle(fr, cir_ax2, rad, pt, pointu)?;
    let v = (dis1 * dis1 + dis2 * dis2).sqrt() * if plandab { -1. } else { 1. };
    let trans = fwd_if((nor_con.dot(frame_normal(&fr.pos_pl)?) <= 0.) == plandab);
    set_interference(
        data,
        if plandab { 1 } else { 2 },
        &gc,
        trans,
        (&gc2d, &line2(Point2::new(0., v), Direction2::X)),
        dstr,
    );
    cir_ax2.set_location(or);
    let gcyl: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(cir_ax2, con_rad)?)).cast();
    let (_, deru, derv) = elslib::cone_d1(0., 0., con_ax3, con_rad, semi);
    let nor_con = Direction3::from_vector(deru.crossed(derv))?;
    let p = Point3::new(
        or.x() + con_rad * fr.dx.x(),
        or.y() + con_rad * fr.dx.y(),
        or.z() + con_rad * fr.dx.z(),
    );
    let (u, vc, d2d, deru, derv) = revolution_side_u(cyl, p, (fu, lu), fr.dy, true);
    let norcyl = Direction3::from_vector(deru.crossed(derv))?;
    let trans = fwd_if((nor_con.dot(norcyl) <= 0.) != plandab);
    set_interference(
        data,
        if plandab { 2 } else { 1 },
        &gcyl,
        trans,
        (
            &line2(Point2::new(u, vc), d2d),
            &line2(Point2::new(0., 0.), Direction2::X),
        ),
        dstr,
    );
    Ok(true)
}

/// `ChFiKPart_MakeChamfer(DStr, Data, Mode, Pln, Cyl, fu, lu, Or1, Or2, Dis1, Dis2,
/// Spine(circle), First, Ofpl, plandab)`: the cone of the chamfer of a plane and a
/// cylinder of perpendicular axis; false when it does not fit inside;
/// `make_chamfer_pln_cyl_circle(&mut ds, &data, mode, (&pln, &cyl), (fu, lu), (or1, or2), (1.0, 1.0), &circ, 0.0, ofpl, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_chamfer_pln_cyl_circle(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    mode: ChamfMode,
    (pln, cyl): (&Plane3, &Cylinder3),
    range: (f64, f64),
    (or1, or2): (Orientation, Orientation),
    (dis1, dis2): (f64, f64),
    spine: &Circle3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
) -> Result<bool, Failure> {
    let dists = match mode {
        ChamfMode::ConstThroatChamfer => (dis1 * 2f64.sqrt(), dis1 * 2f64.sqrt()),
        ChamfMode::ConstThroatWithPenetrationChamfer => {
            let (a2, a1) = (dis1.min(dis2), dis1.max(dis2));
            (a1 * a1 / a2 - a2, (a1 * a1 - a2 * a2).sqrt())
        }
        ChamfMode::ClassicChamfer => (dis1, dis2),
    };
    let fr = circle_chamfer_frame(pln, cyl, cyl.location(), (or1, or2, ofpl), spine, first)?;
    circle_chamfer(dstr, data, cyl, range, &fr, dists, plandab)
}

/// `ChFiKPart_MakeChAsym(DStr, Data, Pln, Cyl, fu, lu, Or1, Or2, Dis, Angle, Spine(circle),
/// First, Ofpl, plandab, DisOnP)`: the cone of the chamfer at a distance and an angle;
/// false when it does not fit inside;
/// `make_ch_asym_pln_cyl_circle(&mut ds, &data, (&pln, &cyl), (fu, lu), (or1, or2), (1.0, 0.5), &circ, 0.0, ofpl, true, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_ch_asym_pln_cyl_circle(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, cyl): (&Plane3, &Cylinder3),
    range: (f64, f64),
    (or1, or2): (Orientation, Orientation),
    (dis, angle): (f64, f64),
    spine: &Circle3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
    dis_on_p: bool,
) -> Result<bool, Failure> {
    let fr = circle_chamfer_frame(pln, cyl, cyl.location(), (or1, or2, ofpl), spine, first)?;
    let cos_np = fr.dpl.dot(fr.dsurf);
    let sin_np = (1. - cos_np * cos_np).sqrt();
    let other = dis / (sin_np / angle.tan() - cos_np);
    let dists = if plandab == dis_on_p {
        (dis, other)
    } else {
        (other, dis)
    };
    circle_chamfer(dstr, data, cyl, range, &fr, dists, plandab)
}
