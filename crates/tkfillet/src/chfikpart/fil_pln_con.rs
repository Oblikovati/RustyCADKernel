// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart/ChFiKPart_ComputeData_FilPlnCon.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillet between a plane and a cone along their circle: a torus (a sphere when it
//! closes on the axis).
use super::common::{
    frame_normal, fwd_if, line2, oriented, set_interference, set_orientation, set_surf,
};
use super::fil_pln_cyl::{TorusFrame, revolution_side_u, torus_plane_side};
use crate::chfi_ds::SurfDataHandle;
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomSphericalSurface, GeomToroidalSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::int_ana::QuadQuadGeo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis3, Circle3, Cone3, Direction2, Direction3, Plane3, Point2, Point3, Vector3,
};
use std::f64::consts::PI;

/// The point of the plane-cone circle at the spine origin (None when they do not cut).
fn plane_cone_point(pln: &Plane3, con: &Cone3, pt_sp: Point3) -> Result<Option<Point3>, Failure> {
    let mut cint = QuadQuadGeo::new();
    cint.perform_plane_cone(*pln, *con, precision::angular(), precision::confusion())?;
    if !cint.is_done() {
        return Ok(None);
    }
    let c = cint.circle(1)?;
    Ok(Some(elclib::value(elclib::parameter(&c, pt_sp)?, &c)))
}

/// The cone frame: the cone normal at the circle point, the plane direction from it and
/// the cone generatrix toward the plane side.
fn cone_frame(
    pln: &Plane3,
    con: &Cone3,
    pv: Point3,
    (dp, or2): (Direction3, Orientation),
) -> Result<(Direction3, Direction3, Direction3), Failure> {
    let pos_pl = pln.position();
    let (u, v) = elslib::parameters(con, pv);
    let (_, vu, vv) = elslib::d1(u, v, con);
    let dc = oriented(Direction3::from_vector(vu.crossed(vv))?, or2);
    let pp = Point3::new(pv.x() + dc.x(), pv.y() + dc.y(), pv.z() + dc.z());
    let (u, v) = elslib::plane_parameters(pos_pl, pp);
    let pp = elslib::plane_d0(u, v, pos_pl);
    let ddp = Direction3::from_vector(Vector3::between(pv, pp))?;
    let (u, v) = elslib::parameters(con, pv);
    let (_, _, dcv) = elslib::d1(u, v, con);
    let mut ddc = Direction3::from_vector(dcv)?;
    if ddc.dot(dp) < 0. {
        ddc.reverse();
    }
    Ok((dc, ddp, ddc))
}

/// The torus frame, plane-cone angle, cone offset, circle point and direction.
type ConeTorusFrame = (TorusFrame, f64, f64, Point3, Direction3);

/// The torus frame of the fillet (None when it does not fit inside) with the angle of
/// the plane and the cone, the offset along the cone and the circle point.
#[allow(clippy::too_many_arguments)]
fn cone_torus_frame(
    pln: &Plane3,
    con: &Cone3,
    (or1, or2): (Orientation, Orientation),
    radius: f64,
    spine: &Circle3,
    first: f64,
    plandab: bool,
) -> Result<Option<ConeTorusFrame>, Failure> {
    let pos_pl = pln.position();
    let dp = oriented(frame_normal(&pos_pl)?, or1);
    let (u, v) = elslib::plane_parameters(pos_pl, con.location());
    let c_pln = elslib::plane_d0(u, v, pos_pl);
    let or = Point3::new(
        c_pln.x() + radius * dp.x(),
        c_pln.y() + radius * dp.y(),
        c_pln.z() + radius * dp.z(),
    );
    let (pt_sp, dsp) = elclib::d1(first, spine);
    let Some(pv) = plane_cone_point(pln, con, pt_sp)? else {
        return Ok(None);
    };
    let dx = Direction3::from_vector(Vector3::between(c_pln, pv))?;
    let dy = Direction3::from_vector(dsp)?;
    let (dc, ddp, ddc) = cone_frame(pln, con, pv, (dp, or2))?;
    let ang = ddp.angle(&ddc);
    let rabio = radius / (ang / 2.).tan();
    let maxrad = c_pln.distance(&pv);
    let dedans = dx.dot(dc) <= 0.;
    let mut dz = dp;
    let (mut c1sphere, rad) = (
        false,
        if dedans {
            maxrad - rabio
        } else {
            maxrad + rabio
        },
    );
    if dedans {
        if !plandab {
            dz.reverse();
        }
        if rad.abs() <= precision::confusion() {
            c1sphere = true;
        } else if rad < 0. {
            return Ok(None);
        }
    } else if plandab {
        dz.reverse();
    }
    let mut ax = Axis3::new(or, dz, dx)?;
    if ax.y_direction().dot(dy) <= 0. {
        ax.y_reverse();
    }
    let tf = TorusFrame {
        ax,
        or,
        c_pln,
        c2d_pln: Point2::new(u, v),
        dx,
        dy,
        dsp,
        rad,
        dedans,
        c1sphere,
    };
    Ok(Some((tf, ang, rabio, pv, ddc)))
}

/// The parameter `v` on the torus (sphere of radius `r`) of a point and the normal there;
/// `v` moved by 2 Pi when `shift`.
fn fillet_v_normal(
    tf: &TorusFrame,
    radius: f64,
    sphere_r: f64,
    p: Point3,
    shift: bool,
) -> Result<(f64, Direction3), Failure> {
    let (u, mut v, du, dv) = if tf.c1sphere {
        let (u, v) = elslib::sphere_parameters(tf.ax, sphere_r, p);
        let (_, du, dv) = elslib::sphere_d1(u, v, tf.ax, sphere_r);
        (u, v, du, dv)
    } else {
        let (u, v) = elslib::torus_parameters(tf.ax, tf.rad, radius, p);
        let (_, du, dv) = elslib::torus_d1(u, v, tf.ax, tf.rad, radius);
        (u, v, du, dv)
    };
    let _ = u;
    if !tf.c1sphere && shift {
        v += 2. * PI;
    }
    Ok((v, Direction3::from_vector(du.crossed(dv))?))
}

/// `ChFiKPart_MakeFillet(DStr, Data, Pln, Con, fu, lu, Or1, Or2, Radius, Spine, First, Ofpl,
/// plandab)`: the torus (a sphere when it closes on the axis) of the fillet of a plane
/// and a cone along their circle, with its circles on both; false when they do not cut
/// or it does not fit; `make_fillet_pln_con(&mut ds, &data, (&pln, &con), (fu, lu), (or1, or2), 1.0, &circ, 0.0, ofpl, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_fillet_pln_con(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, con): (&Plane3, &Cone3),
    (fu, lu): (f64, f64),
    ors: (Orientation, Orientation),
    radius: f64,
    spine: &Circle3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
) -> Result<bool, Failure> {
    let Some((tf, ang, rabio, pv, ddc)) =
        cone_torus_frame(pln, con, ors, radius, spine, first, plandab)?
    else {
        return Ok(false);
    };
    let surf = if tf.c1sphere {
        Handle::new(GeomSphericalSurface::new(tf.ax, radius)?).cast()
    } else {
        Handle::new(GeomToroidalSurface::new(tf.ax, tf.rad, radius)?).cast()
    };
    set_surf(data, surf, dstr);
    let p = Point3::new(
        tf.c_pln.x() + tf.rad * tf.dx.x(),
        tf.c_pln.y() + tf.rad * tf.dx.y(),
        tf.c_pln.z() + tf.rad * tf.dx.z(),
    );
    let (v, nor_fil) = fillet_v_normal(
        &tf,
        radius,
        tf.rad,
        p,
        !plandab && ang < PI / 2. && tf.dedans,
    )?;
    let df = oriented(frame_normal(&pln.position())?, ofpl);
    set_orientation(data, fwd_if(nor_fil.dot(df) > 0.));
    torus_plane_side(dstr, data, pln, &tf, nor_fil, v, plandab, false)?;
    let p = Point3::new(
        pv.x() + rabio * ddc.x(),
        pv.y() + rabio * ddc.y(),
        pv.z() + rabio * ddc.z(),
    );
    let (v2, nor_fil) = fillet_v_normal(
        &tf,
        radius,
        radius,
        p,
        plandab && ang < PI / 2. && tf.dedans,
    )?;
    let (u, vc, d2d, deru, derv) = revolution_side_u(con, p, (fu, lu), tf.dy, false);
    let nor_con = Direction3::from_vector(deru.crossed(derv))?;
    let pc_con = line2(Point2::new(u, vc), d2d);
    let dp = oriented(frame_normal(&pln.position())?, ors.0);
    let scal = Vector3::from(dp).dot(Vector3::between(pv, p));
    let pp = Point3::new(
        tf.c_pln.x() + scal * dp.x(),
        tf.c_pln.y() + scal * dp.y(),
        tf.c_pln.z() + scal * dp.z(),
    );
    let mut circ_ax2 = tf.ax.ax2()?;
    circ_ax2.set_location(pp);
    let gc: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(Circle3::new(
        circ_ax2,
        p.distance(&pp),
    )?))
    .cast();
    let trans = fwd_if((nor_fil.dot(nor_con) <= 0.) != plandab);
    let pc_fil = line2(Point2::new(0., v2), Direction2::X);
    set_interference(
        data,
        if plandab { 2 } else { 1 },
        &gc,
        trans,
        (&pc_con, &pc_fil),
        dstr,
    );
    Ok(true)
}
