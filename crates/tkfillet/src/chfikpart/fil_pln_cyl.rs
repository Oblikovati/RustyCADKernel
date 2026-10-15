// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart/ChFiKPart_ComputeData_FilPlnCyl.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fillet between a plane and a cylinder: a cylinder along a line spine, a torus
//! (a sphere when the radii are equal) along a circle spine.
use super::common::{
    frame_normal, fwd_if, line2, line3, oriented, set_interference, set_orientation, set_surf,
};
use super::fcts::in_period;
use crate::chfi_ds::{SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomCylindricalSurface, GeomSphericalSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeombase::int_ana::QuadQuadGeo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis3, Axis22d, Circle2, Circle3, Cylinder3, Direction2, Direction3, Line3, Plane3, Point2,
    Point3, Vector2, Vector3,
};
use std::f64::consts::PI;

/// The origin of the fillet: the point of the offsets' intersection nearest to the
/// spine origin, and its direction (None when they do not cut).
fn offsets_fillet_line(
    pln: &Plane3,
    cyl: &Cylinder3,
    (nor_f, radius, roff): (Direction3, f64, f64),
    or_spine: Point3,
) -> Result<Option<(Point3, Direction3)>, Failure> {
    let plan_offset = pln.translated(Vector3::from(nor_f) * radius);
    let cyl_offset = Cylinder3::new(cyl.position(), roff)?;
    let mut lint = QuadQuadGeo::new();
    lint.perform_plane_cylinder(
        plan_offset,
        cyl_offset,
        precision::angular(),
        precision::confusion(),
        0.,
    )?;
    if !lint.is_done() {
        return Ok(None);
    }
    let l1 = lint.line(1)?;
    let p1 = elclib::value(elclib::parameter(&l1, or_spine)?, &l1);
    let mut or_fillet = p1;
    if lint.nb_solutions()? == 2 {
        let l2 = lint.line(2)?;
        let p2 = elclib::value(elclib::parameter(&l2, or_spine)?, &l2);
        if p1.square_distance(&or_spine) >= p2.square_distance(&or_spine) {
            or_fillet = p2;
        }
    }
    Ok(Some((or_fillet, l1.direction())))
}

/// The radius of the offset cylinder (None when the fillet does not fit inside).
fn offset_radius(cyl: &Cylinder3, or2: Orientation, radius: f64) -> Result<Option<f64>, Failure> {
    let r = cyl.radius();
    let direct = cyl.direct()?;
    if (or2 == Orientation::Forward && direct) || (or2 == Orientation::Reversed && !direct) {
        return Ok(Some(r + radius));
    }
    Ok((radius < r).then_some(r - radius))
}

/// The parameter on the fillet cylinder of a point (0 beyond Pi), and the line on it.
fn fillet_v_line(ax_fil: Axis3, radius: f64, p: Point3) -> (f64, Handle<dyn Geom2dCurve>) {
    let (mut u, v) = elslib::cylinder_parameters(ax_fil, radius, p);
    if u > PI {
        u = 0.;
    }
    (u, line2(Point2::new(u, v), Direction2::Y))
}

/// The unit normal of the fillet cylinder at a parameter.
fn fillet_normal(ax_fil: Axis3, radius: f64, u: f64) -> Result<Direction3, Failure> {
    let (_, deru, derv) = elslib::cylinder_d1(u, 0., ax_fil, radius);
    Direction3::from_vector(deru.crossed(derv))
}

/// `ChFiKPart_MakeFillet(DStr, Data, Pln, Cyl, fu, lu, Or1, Or2, Radius, Spine(line),
/// First, Ofpl, plandab)`: the cylinder of the fillet of a plane and a cylinder of
/// parallel axes, with its lines on both; false when it does not fit;
/// `make_fillet_pln_cyl_line(&mut ds, &data, (&pln, &cyl), (fu, lu), (or1, or2), 1.0, &spine, 0.0, ofpl, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_fillet_pln_cyl_line(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, cyl): (&Plane3, &Cylinder3),
    (fu, lu): (f64, f64),
    (or1, or2): (Orientation, Orientation),
    radius: f64,
    spine: &Line3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
) -> Result<bool, Failure> {
    let ax_pln = pln.position();
    let nor_pln = frame_normal(&ax_pln)?;
    let nor_f = oriented(nor_pln, or1);
    let Some(roff) = offset_radius(cyl, or2, radius)? else {
        return Ok(false);
    };
    let Some((or_fillet, mut dir_fillet)) =
        offsets_fillet_line(pln, cyl, (nor_f, radius, roff), elclib::value(first, spine))?
    else {
        return Ok(false);
    };
    if dir_fillet.dot(spine.direction()) < 0. {
        dir_fillet.reverse();
    }
    let (mut u_on_cyl, v_on_cyl) = elslib::parameters(cyl, or_fillet);
    let tesp = precision::confusion();
    if u_on_cyl < fu - tesp || u_on_cyl > lu + tesp {
        u_on_cyl = elclib::in_period(u_on_cyl, fu, fu + 2. * PI);
    }
    let (u_on_pln, v_on_pln) = elslib::parameters(pln, or_fillet);
    let to_cyl = Vector3::from(Direction3::from_vector(Vector3::between(
        or_fillet,
        elslib::value(u_on_cyl, v_on_cyl, cyl),
    ))?);
    let (xdir, other_dir) = if plandab {
        (Vector3::from(nor_f.reversed()), to_cyl)
    } else {
        (to_cyl, Vector3::from(nor_f.reversed()))
    };
    let mut ax_fil = Axis3::new(or_fillet, dir_fillet, Direction3::from_vector(xdir)?)?;
    if xdir.crossed(other_dir).dot(Vector3::from(dir_fillet)) < 0. {
        ax_fil.y_reverse();
    }
    set_surf(
        data,
        Handle::new(GeomCylindricalSurface::new(ax_fil, radius)?).cast(),
        dstr,
    );
    let pc_pln = line2(
        Point2::new(u_on_pln, v_on_pln),
        Direction2::new(
            dir_fillet.dot(ax_pln.x_direction()),
            dir_fillet.dot(ax_pln.y_direction()),
        )?,
    );
    let p_on_pln = elslib::value(u_on_pln, v_on_pln, pln);
    let (u_fil, lfil) = fillet_v_line(ax_fil, radius, p_on_pln);
    let toreverse = fillet_normal(ax_fil, radius, u_fil)?.dot(nor_pln) <= 0.;
    set_orientation(
        data,
        if toreverse {
            top_abs::reverse(ofpl)
        } else {
            ofpl
        },
    );
    let trans = fwd_if(toreverse != plandab);
    set_interference(
        data,
        if plandab { 1 } else { 2 },
        &line3(p_on_pln, dir_fillet),
        trans,
        (&pc_pln, &lfil),
        dstr,
    );
    let mut dpc = Direction2::Y;
    if dir_fillet.dot(cyl.position().direction()) < 0. {
        dpc.reverse();
    }
    let pc_cyl = line2(Point2::new(u_on_cyl, v_on_cyl), dpc);
    let p_on_cyl = elslib::value(u_on_cyl, v_on_cyl, cyl);
    let (u_fil, lfil) = fillet_v_line(ax_fil, radius, p_on_cyl);
    let nor_fil = fillet_normal(ax_fil, radius, u_fil)?;
    let (_, deru, derv) = elslib::cylinder_d1(u_on_cyl, v_on_cyl, cyl.position(), cyl.radius());
    let toreverse = nor_fil.dot(Direction3::from_vector(deru.crossed(derv))?) <= 0.;
    let trans = fwd_if(toreverse == plandab);
    set_interference(
        data,
        if plandab { 2 } else { 1 },
        &line3(p_on_cyl, dir_fillet),
        trans,
        (&pc_cyl, &lfil),
        dstr,
    );
    Ok(true)
}

/// The frame of the fillet along a circle spine: the point of the spine (moved off the
/// seam when the spine crosses it), its tangent and the parameters on the cylinder.
fn circle_spine_point(
    cyl: &Cylinder3,
    spine: &Circle3,
    first: f64,
) -> Result<(Point3, Vector3, f64, f64), Failure> {
    let acote = 1e-7;
    let (mut pt_sp, mut dsp) = elclib::d1(first, spine);
    let (mut u, v) = elslib::parameters(cyl, pt_sp);
    if u.abs() < acote || (u - 2. * PI).abs() < acote {
        let (pt_sp2, _) = elclib::d1(first + 0.2, spine);
        let (u2, _) = elslib::parameters(cyl, pt_sp2);
        if (u2 - u).abs() > PI {
            u = 2. * PI - u;
            pt_sp = elslib::value(u, v, cyl);
            let pr = elclib::parameter(spine, pt_sp)?;
            dsp = elclib::d1(pr, spine).1;
        }
    }
    Ok((pt_sp, dsp, u, v))
}

/// The torus (sphere) frame data of the circle spine fillets.
pub(super) struct TorusFrame {
    pub(super) ax: Axis3,
    pub(super) or: Point3,
    pub(super) c_pln: Point3,
    pub(super) c2d_pln: Point2,
    pub(super) dx: Direction3,
    pub(super) dy: Direction3,
    pub(super) dsp: Vector3,
    pub(super) rad: f64,
    pub(super) dedans: bool,
    pub(super) c1sphere: bool,
}

/// The frame of the torus: its centre above the cylinder centre on the plane, its
/// radius inside or outside the cylinder (None when the fillet does not fit inside).
#[allow(clippy::too_many_arguments)]
fn torus_frame(
    pln: &Plane3,
    cyl: &Cylinder3,
    (or1, or2): (Orientation, Orientation),
    radius: f64,
    spine: &Circle3,
    first: f64,
    plandab: bool,
) -> Result<Option<TorusFrame>, Failure> {
    let pos_pl = pln.position();
    let dp = oriented(frame_normal(&pos_pl)?, or1);
    let (u, v) = elslib::plane_parameters(pos_pl, cyl.location());
    let c_pln = elslib::plane_d0(u, v, pos_pl);
    let or = Point3::new(
        c_pln.x() + radius * dp.x(),
        c_pln.y() + radius * dp.y(),
        c_pln.z() + radius * dp.z(),
    );
    let (pt_sp, dsp, uc, vc) = circle_spine_point(cyl, spine, first)?;
    let dx = Direction3::from_vector(Vector3::between(or, pt_sp))?;
    let dx = dp.crossed(dx.crossed(dp)?)?;
    let (_, vu, vv) = elslib::d1(uc, vc, cyl);
    let dc = oriented(Direction3::from_vector(vu.crossed(vv))?, or2);
    let mut dz = dp;
    let cylrad = cyl.radius();
    let dedans = dx.dot(dc) <= 0.;
    let (mut c1sphere, rad) = (
        false,
        if dedans {
            cylrad - radius
        } else {
            cylrad + radius
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
    let dy = Direction3::from_vector(dsp)?;
    let mut ax = Axis3::new(or, dz, dx)?;
    if ax.y_direction().dot(dy) <= 0. {
        ax.y_reverse();
    }
    Ok(Some(TorusFrame {
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
    }))
}

/// The normal of the torus (sphere) at `(0, v)`: for the sphere's first point the frame
/// normal (reversed below the equator).
fn torus_normal(
    tf: &TorusFrame,
    radius: f64,
    v: f64,
    cylrad: f64,
    frame_n: bool,
) -> Result<Direction3, Failure> {
    if tf.c1sphere {
        if frame_n {
            let n = frame_normal(&tf.ax)?;
            return Ok(if v < 0. { n.reversed() } else { n });
        }
        let (_, du, dv) = elslib::sphere_d1(0., v, tf.ax, cylrad);
        return Direction3::from_vector(du.crossed(dv));
    }
    let (_, du, dv) = elslib::torus_d1(0., v, tf.ax, tf.rad, radius);
    Direction3::from_vector(du.crossed(dv))
}

/// The plane side of the circle spine fillets: the circle on the plane (none for a
/// sphere, whose 2d points are set instead when `set_points`) and its interference.
#[allow(clippy::too_many_arguments)]
pub(super) fn torus_plane_side(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    pln: &Plane3,
    tf: &TorusFrame,
    nor_fil: Direction3,
    v: f64,
    plandab: bool,
    set_points: bool,
) -> Result<(), Failure> {
    let pos_pl = pln.position();
    let p = Point3::new(
        tf.c_pln.x() + tf.rad * tf.dx.x(),
        tf.c_pln.y() + tf.rad * tf.dx.y(),
        tf.c_pln.z() + tf.rad * tf.dx.z(),
    );
    let (u, vp) = elslib::plane_parameters(pos_pl, p);
    let p2d_pln = Point2::new(u, vp);
    let (mut gc2d, mut gc): (Handle<dyn Geom2dCurve>, Handle<dyn GeomCurve>) =
        (Handle::default(), Handle::default());
    if !tf.c1sphere {
        let d2d = Direction2::new(
            tf.dsp.dot(Vector3::from(pos_pl.x_direction())),
            tf.dsp.dot(Vector3::from(pos_pl.y_direction())),
        )?;
        let ax2d = Axis22d::with_directions(
            tf.c2d_pln,
            Direction2::from_vector(Vector2::between(tf.c2d_pln, p2d_pln))?,
            d2d,
        )?;
        gc2d = Handle::new(Geom2dCircle::from_circ2d(Circle2::new(ax2d, tf.rad)?)).cast();
        let mut circ_ax2 = tf.ax.ax2()?;
        circ_ax2.set_location(tf.c_pln);
        gc = Handle::new(GeomCircle::from_circ(Circle3::new(circ_ax2, tf.rad)?)).cast();
    } else if set_points {
        let pbid = Point2::default();
        lock(data).set_2d_points(if plandab {
            [p2d_pln, p2d_pln, pbid, pbid]
        } else {
            [pbid, pbid, p2d_pln, p2d_pln]
        });
    }
    let toreverse = nor_fil.dot(frame_normal(&pos_pl)?) <= 0.;
    let trans = fwd_if(toreverse == plandab);
    let pc_fil = line2(Point2::new(0., v), Direction2::X);
    set_interference(
        data,
        if plandab { 1 } else { 2 },
        &gc,
        trans,
        (&gc2d, &pc_fil),
        dstr,
    );
    Ok(())
}

/// The parameter on the surface of revolution of the fillet circle on it, in `[fu, lu]`
/// (by ChFiKPart_InPeriod, else ElCLib::InPeriod, moved to the other bound of a closed
/// range by the sense of the pcurve), and the pcurve direction.
pub(super) fn revolution_side_u<S: elslib::ElementarySurface>(
    surf: &S,
    p: Point3,
    (fu, lu): (f64, f64),
    dy: Direction3,
    kp_period: bool,
) -> (f64, f64, Direction2, Vector3, Vector3) {
    let (mut u, v) = elslib::parameters(surf, p);
    let tol = precision::p_confusion();
    let careaboutsens = (lu - fu - 2. * PI).abs() < tol;
    if u >= fu - tol && u < fu {
        u = fu;
    }
    if u <= lu + tol && u > lu {
        u = lu;
    }
    if u < fu || u > lu {
        u = if kp_period {
            in_period(u, fu, fu + 2. * PI, tol)
        } else {
            elclib::in_period(u, fu, fu + 2. * PI)
        };
    }
    let (_, deru, derv) = elslib::d1(u, v, surf);
    let mut d2d = Direction2::X;
    if deru.dot(Vector3::from(dy)) < 0. {
        d2d.reverse();
        if careaboutsens && (fu - u).abs() < tol {
            u = lu;
        }
    } else if careaboutsens && (lu - u).abs() < tol {
        u = fu;
    }
    (u, v, d2d, deru, derv)
}

/// `ChFiKPart_MakeFillet(DStr, Data, Pln, Cyl, fu, lu, Or1, Or2, Radius, Spine(circle),
/// First, Ofpl, plandab)`: the torus (a sphere when the radii are equal) of the fillet
/// of a plane and a cylinder of perpendicular axis, with its circles on both; false
/// when it does not fit; `make_fillet_pln_cyl_circle(&mut ds, &data, (&pln, &cyl), (fu, lu), (or1, or2), 1.0, &circ, 0.0, ofpl, true)?`.
#[allow(clippy::too_many_arguments)]
pub fn make_fillet_pln_cyl_circle(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pln, cyl): (&Plane3, &Cylinder3),
    (fu, lu): (f64, f64),
    ors: (Orientation, Orientation),
    radius: f64,
    spine: &Circle3,
    first: f64,
    ofpl: Orientation,
    plandab: bool,
) -> Result<bool, Failure> {
    let Some(tf) = torus_frame(pln, cyl, ors, radius, spine, first, plandab)? else {
        return Ok(false);
    };
    let surf = if tf.c1sphere {
        Handle::new(GeomSphericalSurface::new(tf.ax, radius)?).cast()
    } else {
        Handle::new(GeomToroidalSurface::new(tf.ax, tf.rad, radius)?).cast()
    };
    set_surf(data, surf, dstr);
    let cylrad = cyl.radius();
    let v = if tf.dedans == plandab {
        if tf.c1sphere { -PI / 2. } else { 3. * PI / 2. }
    } else {
        PI / 2.
    };
    let nor_fil = torus_normal(&tf, radius, v, cylrad, true)?;
    let df = oriented(frame_normal(&pln.position())?, ofpl);
    set_orientation(data, fwd_if(nor_fil.dot(df) > 0.));
    torus_plane_side(dstr, data, pln, &tf, nor_fil, v, plandab, true)?;
    let p = Point3::new(
        tf.or.x() + cylrad * tf.dx.x(),
        tf.or.y() + cylrad * tf.dx.y(),
        tf.or.z() + cylrad * tf.dx.z(),
    );
    let v2 = if tf.dedans {
        if plandab && !tf.c1sphere { 2. * PI } else { 0. }
    } else {
        PI
    };
    let nor_fil = torus_normal(&tf, radius, v2, cylrad, false)?;
    let (u, vc, d2d, deru, derv) = revolution_side_u(cyl, p, (fu, lu), tf.dy, true);
    let norcyl = Direction3::from_vector(deru.crossed(derv))?;
    let pc_cyl = line2(Point2::new(u, vc), d2d);
    let mut circ_ax2 = tf.ax.ax2()?;
    circ_ax2.set_location(tf.or);
    let gc: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::from_circ(Circle3::new(circ_ax2, cylrad)?)).cast();
    let trans = fwd_if((nor_fil.dot(norcyl) <= 0.) != plandab);
    let pc_fil = line2(Point2::new(0., v2), Direction2::X);
    set_interference(
        data,
        if plandab { 2 } else { 1 },
        &gc,
        trans,
        (&pc_cyl, &pc_fil),
        dstr,
    );
    Ok(true)
}
