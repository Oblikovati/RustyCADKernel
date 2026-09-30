//! GeomPlate probe operations (constraints, plate surface, average plane, HalfSpace),
//! the Rust twin of tools/oracle/tkgeomalgo_probe_geomplate.cxx.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, pnt2d, real, xy, xyz};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBezierCurve, GeomBezierSurface, GeomCircle, GeomCurve, GeomCylindricalSurface,
    GeomGeometry, GeomLine, GeomPlane, GeomSphericalSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeomalgo::geom_plate::{
    GeomPlateAij, GeomPlateBuildAveragePlane, GeomPlateCurveConstraint, GeomPlatePointConstraint,
    GeomPlateSurface,
};
use rustycad_tkgeomalgo::law::{LawFunction, LawLinear};
use rustycad_tkgeomalgo::plate::{PlatePinpointConstraint, PlatePlate};
use rustycad_tkgeombase::geom_lprop::SLProps;
use rustycad_tkgeombase::proj_lib::CompProjectedCurve;
use rustycad_tkmath::gp::{Axis2, Axis3, Direction2, Point2, Point3, Transform3, Vector3, Xy};

/// `ga_gp_surface`: -1 none, 0 plane, 1 cylinder, 2 sphere, 3 a 3x3 Bezier surface.
pub fn surface(r: &mut Reader) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    if kind < 0 {
        return Ok(Handle::default());
    }
    if kind == 3 {
        let mut poles: Array2<'static, Point3> =
            Array2::with_bounds(1, 3, 1, 3).expect("3x3 poles");
        for i in 1..=3 {
            for j in 1..=3 {
                poles[(i, j)] = r.pnt();
            }
        }
        return Ok(Handle::new(GeomBezierSurface::new(&poles)?).cast());
    }
    let (p, n, radius) = (r.pnt(), r.dir(), r.real());
    let ax = Axis3::from_normal(p, n)?;
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(ax)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(ax, radius)?).cast(),
        _ => Handle::new(GeomSphericalSurface::new(ax, radius)?).cast(),
    })
}

fn bezier_poles<P: Default + Clone + Send + Sync + 'static>(
    r: &mut Reader,
    read: fn(&mut Reader) -> P,
) -> Array1<'static, P> {
    let n = r.integer();
    let mut poles: Array1<'static, P> = Array1::with_bounds(1, n).expect("probe poles");
    for i in 1..=n {
        poles[i] = read(r);
    }
    poles
}

fn adaptor3d(
    curve: Handle<dyn GeomCurve>,
    range: Option<(f64, f64)>,
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let adaptor = match range {
        Some((t0, t1)) => GeomAdaptorCurve::with_range(&curve, t0, t1)?,
        None => GeomAdaptorCurve::with_curve(&curve)?,
    };
    Ok(Handle::new(adaptor).cast())
}

/// `ga_gp_curve`: 0 Bezier, 1 line, 2 circle, 3 a 2d line on a surface, 4 a 2d Bezier on
/// a surface.
fn curve(r: &mut Reader, kind: i32) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    match kind {
        0 => {
            let poles = bezier_poles(r, |r| r.pnt());
            adaptor3d(Handle::new(GeomBezierCurve::new(&poles)?).cast(), None)
        }
        1 => {
            let (p, d, t0, t1) = (r.pnt(), r.dir(), r.real(), r.real());
            adaptor3d(
                Handle::new(GeomLine::from_point(p, d)).cast(),
                Some((t0, t1)),
            )
        }
        2 => {
            let (c, n, radius, t0, t1) = (r.pnt(), r.dir(), r.real(), r.real(), r.real());
            let circle = GeomCircle::new(Axis2::new(c, n)?, radius)?;
            adaptor3d(Handle::new(circle).cast(), Some((t0, t1)))
        }
        _ => curve_on_surface(r, kind),
    }
}

fn curve_on_surface(r: &mut Reader, kind: i32) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let s = surface(r)?;
    let c2d: Handle<dyn Adaptor2dCurve2d> = if kind == 3 {
        let (p, d, t0, t1) = (r.pnt2d(), r.dir2d(), r.real(), r.real());
        let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p, d)).cast();
        Handle::new(Geom2dAdaptorCurve::with_range(&line, t0, t1)?).cast()
    } else {
        let poles = bezier_poles(r, |r| r.pnt2d());
        let bezier: Handle<dyn Geom2dCurve> = Handle::new(Geom2dBezierCurve::new(&poles)?).cast();
        Handle::new(Geom2dAdaptorCurve::with_curve(&bezier)?).cast()
    };
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast();
    Ok(Handle::new(Adaptor3dCurveOnSurface::with_curve(&c2d, &hs)?).cast())
}

pub fn law(first: f64, last: f64, v0: f64, v1: f64) -> Handle<dyn LawFunction> {
    let law = LawLinear::new();
    law.set(first, v0, last, v1);
    Handle::new(law).cast()
}

/// `ga_gp_curve_constraint`: `kind order npts tolDist tolAng tolCurv crit [laws] geometry`.
pub fn curve_constraint(r: &mut Reader) -> Result<Handle<GeomPlateCurveConstraint>, Failure> {
    let (kind, order, npts) = (r.integer(), r.integer(), r.integer());
    let (td, ta, tc) = (r.real(), r.real(), r.real());
    let crit = r.integer();
    let mut laws = [0.0; 6];
    if crit >= 1 {
        (laws[0], laws[1]) = (r.real(), r.real());
    }
    if crit >= 2 {
        laws.iter_mut().skip(2).for_each(|v| *v = r.real());
    }
    let c = curve(r, kind)?;
    let cc = GeomPlateCurveConstraint::new(&c, order, npts, td, ta, tc)?;
    let geometry = c.get().expect("probe curve");
    let (f, l) = (geometry.first_parameter()?, geometry.last_parameter()?);
    if crit >= 1 {
        cc.set_g0_criterion(&law(f, l, laws[0], laws[1]));
    }
    if crit >= 2 {
        cc.set_g1_criterion(&law(f, l, laws[2], laws[3]))?;
        cc.set_g2_criterion(&law(f, l, laws[4], laws[5]))?;
    }
    Ok(Handle::new(cc))
}

/// `ga_gp_point_constraint`: a point or a surface point, then an optional parameter.
pub fn point_constraint(
    r: &mut Reader,
) -> Result<(i32, Handle<GeomPlatePointConstraint>), Failure> {
    let (kind, order) = (r.integer(), r.integer());
    let pc = if kind == 0 {
        let p = r.pnt();
        GeomPlatePointConstraint::with_point(p, order, r.real())?
    } else {
        let (u, v) = (r.real(), r.real());
        let s = surface(r)?;
        let (td, ta, tc) = (r.real(), r.real(), r.real());
        GeomPlatePointConstraint::on_surface(u, v, &s, order, td, ta, tc)?
    };
    if r.flag() {
        pc.set_pnt2d_on_surf(r.pnt2d());
    }
    Ok((kind, Handle::new(pc)))
}

pub fn vec(v: Vector3) {
    xyz(*v.xyz());
}

fn lprop(p: &mut SLProps) -> Result<(), Failure> {
    pnt(p.value());
    vec(p.d1u()?);
    vec(p.d1v()?);
    flag(p.is_normal_defined()?);
    Ok(())
}

fn point_members(pc: &GeomPlatePointConstraint, kind: i32) {
    int(pc.order());
    real(pc.g0_criterion());
    real(pc.g1_criterion());
    real(pc.g2_criterion());
    pnt(pc.d0());
    let (p, v1, v2) = pc.d1();
    pnt(p);
    vec(v1);
    vec(v2);
    let d2 = pc.d2();
    pnt(d2.point);
    [d2.d1u, d2.d1v, d2.d2u, d2.d2v, d2.d2uv]
        .into_iter()
        .for_each(vec);
    flag(pc.has_pnt2d_on_surf());
    pnt2d(pc.pnt2d_on_surf());
    if kind == 1 {
        attempt(|| lprop(&mut *pc.lprop_surf()?));
    }
}

/// `gplate_point`: every member of GeomPlate_PointConstraint, then the setters.
pub fn point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (kind, pc) = point_constraint(&mut r)?;
        let pc = pc.get().expect("probe constraint");
        point_members(pc, kind);
        pc.set_order(r.integer());
        pc.set_g0_criterion(r.real());
        pc.set_g1_criterion(r.real());
        pc.set_g2_criterion(r.real());
        pc.set_pnt2d_on_surf(r.pnt2d());
        int(pc.order());
        real(pc.g0_criterion());
        real(pc.g1_criterion());
        real(pc.g2_criterion());
        flag(pc.has_pnt2d_on_surf());
        pnt2d(pc.pnt2d_on_surf());
        Ok(())
    });
    end();
}

/// `ga_gp_curve2d`: the 2d curve at its ends and middle, or its nullity.
pub fn curve2d(c: &Handle<dyn Geom2dCurve>, f: f64, l: f64) {
    flag(c.is_null());
    let Some(curve) = c.get() else { return };
    for u in [f, 0.5 * (f + l), l] {
        attempt(|| {
            pnt2d(curve.value(u)?);
            Ok(())
        });
    }
}

fn curve_at(cc: &GeomPlateCurveConstraint, u: f64) {
    attempt(|| {
        pnt(cc.d0(u)?);
        Ok(())
    });
    attempt(|| {
        let d1 = cc.d1(u)?;
        pnt(d1.point);
        vec(d1.d1u);
        vec(d1.d1v);
        Ok(())
    });
    attempt(|| {
        let d2 = cc.d2(u)?;
        pnt(d2.point);
        [d2.d2u, d2.d2v, d2.d2uv].into_iter().for_each(vec);
        Ok(())
    });
    for crit in [cc.g0_criterion(u), cc.g1_criterion(u), cc.g2_criterion(u)] {
        attempt(|| {
            real(crit?);
            Ok(())
        });
    }
    attempt(|| lprop(&mut *cc.lprop_surf(u)?));
}

fn projected_members(r: &mut Reader, cc: &GeomPlateCurveConstraint) -> Result<(), Failure> {
    let s = surface(r)?;
    let (tu, tv) = (r.real(), r.real());
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&s)?).cast();
    let proj = CompProjectedCurve::with_tolerances(&hs, &cc.curve3d(), tu, tv)?;
    let (pf, pl) = (proj.first_parameter()?, proj.last_parameter()?);
    let proj: Handle<dyn Adaptor2dCurve2d> = Handle::new(proj).cast();
    cc.set_projected_curve(&proj, tu, tv);
    flag(cc.projected_curve().is_null());
    attempt(|| {
        real(cc.first_parameter()?);
        Ok(())
    });
    attempt(|| {
        real(cc.last_parameter()?);
        Ok(())
    });
    attempt(|| {
        curve2d(&cc.curve2d_on_surf()?, pf, pl);
        Ok(())
    });
    Ok(())
}

fn curve_members(r: &mut Reader, cc: &GeomPlateCurveConstraint) -> Result<(), Failure> {
    let us = [r.real(), r.real(), r.real()];
    int(cc.order());
    int(cc.nb_points());
    for value in [cc.first_parameter(), cc.last_parameter(), cc.length()] {
        attempt(|| {
            real(value?);
            Ok(())
        });
    }
    us.into_iter().for_each(|u| curve_at(cc, u));
    flag(cc.curve3d().is_null());
    flag(cc.projected_curve().is_null());
    if r.flag() {
        projected_members(r, cc)?;
    }
    cc.set_order(r.integer());
    cc.set_nb_points(r.integer());
    int(cc.order());
    int(cc.nb_points());
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0., 0.),
        Direction2::new(1., 0.)?,
    ))
    .cast();
    cc.set_curve2d_on_surf(&line);
    attempt(|| {
        curve2d(&cc.curve2d_on_surf()?, 0., 1.);
        Ok(())
    });
    Ok(())
}

/// `gplate_curve`: every member of GeomPlate_CurveConstraint.
pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let cc = curve_constraint(&mut r)?;
        curve_members(&mut r, cc.get().expect("probe constraint"))
    });
    end();
}

/// `gplate_curve_empty`: the empty GeomPlate_CurveConstraint.
pub fn curve_empty_case(_values: &[f64]) {
    let cc = GeomPlateCurveConstraint::empty().expect("empty constraint");
    int(cc.order());
    int(cc.nb_points());
    flag(cc.curve3d().is_null());
    flag(cc.projected_curve().is_null());
    attempt(|| {
        flag(cc.curve2d_on_surf()?.is_null());
        Ok(())
    });
    cc.set_g0_criterion(&law(0., 1., 3., 4.));
    attempt(|| {
        real(cc.g0_criterion(0.25)?);
        Ok(())
    });
    attempt(|| cc.set_g1_criterion(&law(0., 1., 1., 2.)));
    attempt(|| {
        real(cc.g1_criterion(0.5)?);
        Ok(())
    });
    attempt(|| cc.set_g2_criterion(&law(0., 1., 5., 7.)));
    attempt(|| {
        real(cc.g2_criterion(0.75)?);
        Ok(())
    });
    end();
}

/// `ga_gp_plate`: a Plate_Plate of `n` pinpoints solved at `order`.
pub fn plate(r: &mut Reader) -> Result<PlatePlate, Failure> {
    let mut plate = PlatePlate::new();
    let n = r.integer();
    for _ in 0..n {
        let (u, v) = (r.real(), r.real());
        let value = r.xyz();
        let (iu, iv) = (r.integer(), r.integer());
        plate.load_pinpoint(&PlatePinpointConstraint::new(Xy::new(u, v), value, iu, iv));
    }
    let order = r.integer();
    plate.solve_ti(order, 1., &ProgressRange::default())?;
    Ok(plate)
}

pub fn eval(s: &dyn GeomSurface, u: f64, v: f64) {
    attempt(|| {
        pnt(s.value(u, v)?);
        Ok(())
    });
    attempt(|| {
        let d1 = s.d1(u, v)?;
        pnt(d1.point);
        vec(d1.d1u);
        vec(d1.d1v);
        Ok(())
    });
    attempt(|| {
        let d2 = s.d2(u, v)?;
        [d2.d2u, d2.d2v, d2.d2uv].into_iter().for_each(vec);
        Ok(())
    });
}

pub fn bounds(s: &dyn GeomSurface) {
    let (a, b, c, d) = s.bounds();
    [a, b, c, d].into_iter().for_each(real);
}

fn surface_queries(gs: &GeomPlateSurface) -> Result<(), Failure> {
    attempt(|| gs.d3(0., 0.).map(|_| ()));
    attempt(|| {
        vec(gs.dn(0., 0., 1, 0)?);
        Ok(())
    });
    attempt(|| {
        flag(gs.is_cn_u(1)?);
        Ok(())
    });
    attempt(|| {
        flag(gs.is_cn_v(1)?);
        Ok(())
    });
    flag(gs.is_u_closed()?);
    flag(gs.is_v_closed()?);
    flag(gs.is_u_periodic());
    flag(gs.is_v_periodic());
    real(gs.u_period()?);
    real(gs.v_period()?);
    int(gs.continuity() as i32);
    real(gs.u_reversed_parameter(0.25)?);
    real(gs.v_reversed_parameter(-0.5)?);
    flag(gs.u_iso(0.5)?.is_null());
    flag(gs.v_iso(0.5)?.is_null());
    let (tu, tv) = gs.transform_parameters(0.3, 0.7, Transform3::default());
    real(tu);
    real(tv);
    gs.u_reverse()?;
    gs.v_reverse()?;
    gs.transform(Transform3::default())?;
    bounds(gs);
    Ok(())
}

fn surface_body(r: &mut Reader) -> Result<(), Failure> {
    let init = surface(r)?;
    let plate = plate(r)?;
    flag(plate.is_done());
    let gs = Handle::new(GeomPlateSurface::new(&init, &plate));
    let g = gs.get().expect("plate surface");
    let b = [r.real(), r.real(), r.real(), r.real()];
    attempt(|| g.set_bounds(b[0], b[1], b[2], b[3]));
    bounds(g);
    attempt(|| {
        let (a, bb, c, d) = g.real_bounds()?;
        [a, bb, c, d].into_iter().for_each(real);
        Ok(())
    });
    let mut seq: Sequence<Xy> = Sequence::default();
    g.constraints(&mut seq);
    int(seq.length());
    seq.iter().for_each(|p| xy(*p));
    let n = r.integer();
    for _ in 0..n {
        let (u, v) = (r.real(), r.real());
        eval(g, u, v);
    }
    surface_queries(g)?;
    flag(g.call_surfinit().object_address() == init.object_address());
    let copy: Handle<GeomPlateSurface> = g.copy()?.cast();
    bounds(copy.get().expect("copy"));
    let nested = GeomPlateSurface::new(&gs.cast::<dyn GeomSurface>(), &plate);
    bounds(&nested);
    attempt(|| {
        eval(&nested, 0.5, 0.5);
        Ok(())
    });
    Ok(())
}

/// `gplate_surface`: every member of GeomPlate_Surface.
pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| surface_body(&mut r));
    end();
}

fn average_out(bap: &GeomPlateBuildAveragePlane) {
    flag(bap.is_plane());
    flag(bap.is_line());
    attempt(|| {
        let pl = bap.plane()?;
        flag(pl.is_null());
        if let Some(pl) = pl.get() {
            let a = pl.pln().position();
            pnt(a.location());
            dir(a.direction());
            dir(a.x_direction());
            dir(a.y_direction());
        }
        let (u1, u2, v1, v2) = bap.min_max_box();
        [u1, u2, v1, v2].into_iter().for_each(real);
        Ok(())
    });
    attempt(|| {
        let ln = bap.line()?;
        flag(ln.is_null());
        if let Some(ln) = ln.get() {
            pnt(ln.position().location());
            dir(ln.position().direction());
        }
        Ok(())
    });
}

fn points(r: &mut Reader, n: i32) -> Handle<HArray1<Point3>> {
    let pts: HArray1<Point3> = HArray1::with_bounds(1, n).expect("probe points");
    for i in 1..=n {
        pts.change_array1()[i] = r.pnt();
    }
    Handle::new(pts)
}

/// `gplate_average`: GeomPlate_BuildAveragePlane of a point cloud.
pub fn average_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let pts = points(&mut r, n);
    let (nb, tol, popt, nopt) = (r.integer(), r.real(), r.integer(), r.integer());
    attempt(|| {
        average_out(&GeomPlateBuildAveragePlane::new(&pts, nb, tol, popt, nopt)?);
        Ok(())
    });
    end();
}

/// `gplate_normals`: GeomPlate_BuildAveragePlane of the best vector of normals.
pub fn normals_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let nn = r.integer();
    let mut normals: Sequence<Vector3> = Sequence::default();
    for _ in 0..nn {
        normals.append(Vector3::from(r.xyz()));
    }
    let n = r.integer();
    let pts = points(&mut r, n);
    attempt(|| {
        average_out(&GeomPlateBuildAveragePlane::with_normals(&normals, &pts)?);
        Ok(())
    });
    end();
}

/// `gplate_halfspace`: successive HalfSpace calls on one set of normals.
pub fn halfspace_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (lin, ang, steps) = (r.real(), r.real(), r.integer());
    let mut normals: Sequence<Vector3> = Sequence::default();
    let mut bset: Sequence<GeomPlateAij> = Sequence::default();
    bset.append(GeomPlateAij::new());
    bset.clear(None);
    for _ in 0..steps {
        let n = r.integer();
        let mut fresh: Sequence<Vector3> = Sequence::default();
        for _ in 0..n {
            fresh.append(Vector3::from(r.xyz()));
        }
        attempt(|| {
            flag(GeomPlateBuildAveragePlane::half_space(
                &fresh,
                &mut normals,
                &mut bset,
                lin,
                ang,
            )?);
            Ok(())
        });
        int(normals.length());
        normals.iter().for_each(|v| vec(*v));
        int(bset.length());
    }
    end();
}
