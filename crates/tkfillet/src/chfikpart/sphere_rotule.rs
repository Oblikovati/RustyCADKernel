// Rust translation of ModelingAlgorithms/TKFillet/ChFiKPart/ChFiKPart_ComputeData_Sphere.cxx
// and ChFiKPart_ComputeData_Rotule.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner fillets of the particular cases: the sphere closing three fillets and the
//! torus joint (rotule) between three planes.
use super::common::{frame_normal, fwd_if, oriented, set_interference, set_orientation, set_surf};
use super::fcts::{pcurve, proj_pc};
use crate::chfi_ds::{SurfDataHandle, lock};
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomCurve, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gce::MakeCirc;
use rustycad_tkgeombase::int_ana::QuadQuadGeo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Axis22d, Direction2, Direction3, Plane3, Point2, Point3, Vector2, Vector3,
};
use std::f64::consts::PI;

/// The centre of the sphere through the point on the first surface and the two on the
/// second, on the side of the oriented normal of the first (None when neither fits).
#[allow(clippy::neg_cmp_op_on_partial_ord)] // NaN keeps the native branch.
fn sphere_center(
    p: (Point3, Point3, Point3),
    rad: f64,
    ds1: Direction3,
) -> Result<Option<(Point3, Direction3)>, Failure> {
    let ci = MakeCirc::through_points(p.0, p.1, p.2)?.value()?;
    let (di, pp, rr) = (ci.axis().direction(), ci.location(), ci.radius());
    let delta = (rad * rad - rr * rr).sqrt();
    for s in [1., -1.] {
        let cen = Point3::new(
            pp.x() + s * delta * di.x(),
            pp.y() + s * delta * di.y(),
            pp.z() + s * delta * di.z(),
        );
        let dz = Direction3::from_vector(Vector3::between(p.0, cen))?;
        // Negated so that a NaN centre (the circle wider than the sphere) is kept, as
        // natively.
        if !((ds1.dot(dz) - 1.).abs() > precision::confusion()) {
            return Ok(Some((cen, dz)));
        }
    }
    Ok(None)
}

/// The pcurve of the circle of the second surface on the sphere, moved onto the point of
/// the first parameters.
fn sphere_circle_pcurve(
    gsph: &Handle<dyn GeomSurface>,
    c: &Handle<dyn GeomCurve>,
    ang: f64,
    (uu1, vv1): (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let c2d = proj_pc(
        &GeomAdaptorCurve::with_range(c, 0., ang)?,
        &GeomAdaptorSurface::with_surface(gsph)?,
    )?;
    let cc = c2d.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "ChFiKPart_Sphere: no projected pcurve",
        )
    })?;
    let p2dbid = cc.value(0.)?;
    let pp2dbid = Point2::new(uu1, vv1);
    if !pp2dbid.is_equal(p2dbid, precision::confusion()) {
        cc.translate(Vector2::between(p2dbid, pp2dbid))?;
    }
    Ok(c2d)
}

/// `ChFiKPart_Sphere(DStr, Data, S1, S2, OrFace1, OrFace2, Or1, Or2, Rad, PS1, P1S2, P2S2)`:
/// the sphere of a corner touching the first surface at a point and the second along a
/// circle arc between two points; false when the sphere does not fit;
/// `sphere(&mut ds, &data, (&s1, &s2), (of1, or1), 1.0, (ps1, p1s2, p2s2))?`.
#[allow(clippy::too_many_arguments, clippy::neg_cmp_op_on_partial_ord)] // NaN keeps the native branches.
pub fn sphere(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (s1, s2): (&dyn Adaptor3dSurface, &dyn Adaptor3dSurface),
    (or_face1, or1): (Orientation, Orientation),
    rad: f64,
    (ps1, p1s2, p2s2): (Point2, Point2, Point2),
) -> Result<bool, Failure> {
    let ptol = precision::confusion();
    let d1 = s1.d1(ps1.x(), ps1.y())?;
    let dnat1 = Direction3::from_vector(d1.d1u.crossed(d1.d1v))?;
    let (ds1, df1, p1) = (oriented(dnat1, or1), oriented(dnat1, or_face1), d1.point);
    let (p2, p3) = (s2.value(p1s2.x(), p1s2.y())?, s2.value(p2s2.x(), p2s2.y())?);
    let Some((cen, dz)) = sphere_center((p1, p2, p3), rad, ds1)? else {
        return Ok(false);
    };
    let ddx = Direction3::from_vector(Vector3::between(cen, p2))?;
    let ddy = Direction3::from_vector(Vector3::between(cen, p3))?;
    let (mut dddx, mut ddz) = (ddx, dz.reversed());
    let mut fil_ax3 = Axis3::new(cen, dz, dz.crossed(ddx.crossed(dz)?)?)?;
    if fil_ax3.y_direction().dot(ddy) <= 0. {
        fil_ax3.y_reverse();
        ddz.reverse();
        dddx.reverse();
    }
    let gsph: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(fil_ax3, rad)?).cast();
    set_surf(data, gsph.clone(), dstr);
    set_orientation(data, fwd_if(!(ddz.dot(df1) <= 0.)));
    let (_, vv1) = elslib::sphere_parameters(fil_ax3, rad, p2);
    let (uu2, _) = elslib::sphere_parameters(fil_ax3, rad, p3);
    let c2d_fil0: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0., -PI / 2.),
        Direction2::X,
    ))
    .cast();
    set_interference(
        data,
        1,
        &Handle::default(),
        fwd_if(ddz.dot(dnat1) <= 0.),
        (&Handle::default(), &c2d_fil0),
        dstr,
    );
    let ang = ddx.angle(&ddy);
    let c: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(
        Axis2::with_x_direction(cen, ddx.crossed(ddy)?, ddx)?,
        rad,
    )?)
    .cast();
    let c2d_fil = sphere_circle_pcurve(&gsph, &c, ang, (0., vv1))?;
    let v2d = Vector2::between(p1s2, p2s2);
    let c2d: Handle<dyn Geom2dCurve> = if (v2d.magnitude() - ang).abs() <= ptol {
        Handle::new(Geom2dLine::from_point(p1s2, Direction2::from_vector(v2d)?)).cast()
    } else {
        pcurve(p1s2, p2s2, 0., ang)?.cast()
    };
    let d2 = s2.d1(p1s2.x(), p1s2.y())?;
    let ds2 = Direction3::from_vector(d2.d1u.crossed(d2.d1v))?;
    set_interference(
        data,
        2,
        &c,
        fwd_if(!(ds2.dot(dddx) <= 0.)),
        (&c2d, &c2d_fil),
        dstr,
    );
    let mut d = lock(data);
    d.change_vertex_first_on_s1().set_point(p1);
    d.change_vertex_last_on_s1().set_point(p1);
    d.change_vertex_first_on_s2().set_point(p2);
    d.change_vertex_last_on_s2().set_point(p3);
    d.change_interference_on_s1().set_first_parameter(0.);
    d.change_interference_on_s1().set_last_parameter(uu2);
    d.change_interference_on_s2().set_first_parameter(0.);
    d.change_interference_on_s2().set_last_parameter(ang);
    Ok(true)
}

/// `ChFiKPart_MakeRotule(DStr, Data, pl, pl1, pl2, opl, opl1, opl2, r, ofpl)`: the torus
/// joint of radius `r` on the plane `pl` between the fillets of the planes `pl1` and
/// `pl2`; false when those planes do not cut;
/// `make_rotule(&mut ds, &data, (&pl, &pl1, &pl2), (opl, opl1, opl2), 1.0, ofpl)?`.
pub fn make_rotule(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
    (pl, pl1, pl2): (&Plane3, &Plane3, &Plane3),
    (opl, opl1, opl2): (Orientation, Orientation, Orientation),
    r: f64,
    ofpl: Orientation,
) -> Result<bool, Failure> {
    let dplnat = frame_normal(&pl.position())?;
    let (dpl, dfpl) = (oriented(dplnat, opl), oriented(dplnat, ofpl));
    let dpl1 = oriented(frame_normal(&pl1.position())?, opl1);
    let dpl2 = oriented(frame_normal(&pl2.position())?, opl2);
    let alpha = dpl1.angle(&dpl2);
    let mut lint = QuadQuadGeo::new();
    lint.perform_planes(*pl1, *pl2, precision::angular(), precision::confusion())?;
    if !lint.is_done() {
        return Ok(false);
    }
    let l = lint.line(1)?;
    let pcirc = elclib::value(elclib::parameter(&l, pl.location())?, &l);
    let ptor = Point3::new(
        pcirc.x() + r * dpl.x(),
        pcirc.y() + r * dpl.y(),
        pcirc.z() + r * dpl.z(),
    );
    let mut ppos = Axis3::new(ptor, dpl.reversed(), dpl1)?;
    if ppos.y_direction().dot(dpl2) < 0. {
        ppos.y_reverse();
    }
    set_surf(
        data,
        Handle::new(GeomToroidalSurface::new(ppos, r, r)?).cast(),
        dstr,
    );
    let (pp, du, dv) = elslib::torus_d1(0., PI / 2., ppos, r, r);
    let drot = Direction3::from_vector(du.crossed(dv))?;
    set_orientation(data, fwd_if(drot.dot(dfpl) > 0.));
    let mut circ_ax2 = ppos.ax2()?;
    circ_ax2.set_location(pcirc);
    let gc: Handle<dyn GeomCurve> = Handle::new(GeomCircle::new(circ_ax2, r)?).cast();
    let (u, v) = elslib::parameters(pl, pcirc);
    let pos = pl.position();
    let dx2d = Direction2::new(dpl1.dot(pos.x_direction()), dpl1.dot(pos.y_direction()))?;
    let dy2d = Direction2::new(
        ppos.y_direction().dot(pos.x_direction()),
        ppos.y_direction().dot(pos.y_direction()),
    )?;
    let gc2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::new(
        Axis22d::with_directions(Point2::new(u, v), dx2d, dy2d)?,
        r,
    )?)
    .cast();
    let trans = fwd_if(drot.dot(dplnat) <= 0.);
    let gl2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0., PI / 2.),
        Direction2::X,
    ))
    .cast();
    set_interference(data, 1, &gc, trans, (&gc2d, &gl2d), dstr);
    let gl2d_coin: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0., PI), Direction2::X)).cast();
    set_interference(
        data,
        2,
        &Handle::default(),
        trans,
        (&Handle::default(), &gl2d_coin),
        dstr,
    );
    let mut d = lock(data);
    d.change_vertex_first_on_s1().set_point(pp);
    d.change_vertex_last_on_s1()
        .set_point(elslib::torus_d0(alpha, PI / 2., ppos, r, r));
    d.change_interference_on_s1().set_first_parameter(0.);
    d.change_interference_on_s1().set_last_parameter(alpha);
    d.change_interference_on_s2().set_first_parameter(0.);
    d.change_interference_on_s2().set_last_parameter(alpha);
    Ok(true)
}
