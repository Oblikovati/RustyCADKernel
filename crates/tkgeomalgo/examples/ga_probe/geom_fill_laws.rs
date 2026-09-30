//! GeomFill section and location law probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomfill_laws.cxx.
use super::common::{Reader, end, flag, int, pnt, real, show};
use super::geom_fill_sweep::{Outputs, filled, pnts, reals, vecs};
use super::geom_fill_trihedron::SHAPES;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::geom_fill::*;

pub use super::geom_fill_sections::{gfl_curve, gfl_section};

fn show_unit(result: Result<(), Failure>) {
    show(result, |()| ());
}

/// `ga_gfl_section_eval`.
pub fn section_eval(s: &dyn GeomFillSectionLaw, u: f64, n: i32, lower: i32) {
    let up = lower + n - 1;
    let mut o = Outputs::new(lower, up);
    show(s.d0(u, &mut o.p, &mut o.w), flag);
    int(o.p.lower());
    pnts(&o.p);
    reals(&o.w);
    let mut o = Outputs::new(lower, up);
    show(
        s.d1(
            u,
            SectionD1 {
                poles: &mut o.p,
                d_poles: &mut o.dp,
                weights: &mut o.w,
                d_weights: &mut o.dw,
            },
        ),
        flag,
    );
    pnts(&o.p);
    vecs(&o.dp);
    reals(&o.w);
    reals(&o.dw);
    let mut o = Outputs::new(lower, up);
    let d1 = SectionD1 {
        poles: &mut o.p,
        d_poles: &mut o.dp,
        weights: &mut o.w,
        d_weights: &mut o.dw,
    };
    show(
        s.d2(
            u,
            SectionD2 {
                d1,
                d2_poles: &mut o.d2p,
                d2_weights: &mut o.d2w,
            },
        ),
        flag,
    );
    pnts(&o.p);
    vecs(&o.dp);
    vecs(&o.d2p);
    reals(&o.w);
    reals(&o.dw);
    reals(&o.d2w);
}

fn tolerances(s: &dyn GeomFillSectionLaw, np: i32, bound_tol: f64) {
    let mut t = filled(1, if np > 1 { np } else { 2 }, 0.0);
    show(s.get_tolerance(bound_tol, 1e-2, 0.1, &mut t), |()| {
        reals(&t)
    });
}

fn shape_queries(s: &dyn GeomFillSectionLaw) {
    for shape in SHAPES {
        show(s.nb_intervals(shape), |n| {
            int(n);
            let mut t = filled(1, n + 1, -1.0);
            show_unit(s.intervals(&mut t, shape));
            reals(&t);
        });
    }
}

/// `ga_gfl_section_queries`; returns NbPoles.
pub fn section_queries(s: &dyn GeomFillSectionLaw) -> i32 {
    let mut np = 0;
    show(s.section_shape(), |(p, k, d)| {
        np = p;
        [p, k, d].into_iter().for_each(int);
    });
    let mut k: Array1<'static, f64> = filled(1, 1, 0.0);
    show(s.knots(&mut k), |()| {
        int(k.lower());
        reals(&k);
    });
    let mut m: Array1<'static, i32> = filled(1, 1, 0);
    show(s.mults(&mut m), |()| {
        m.as_slice().iter().for_each(|v| int(*v))
    });
    show(s.is_rational(), flag);
    show(s.is_u_periodic(), flag);
    show(s.is_v_periodic(), flag);
    shape_queries(s);
    show(s.get_domain(), |(f, l)| [f, l].into_iter().for_each(real));
    show(s.get_interval(), |(f, l)| [f, l].into_iter().for_each(real));
    tolerances(s, np, 1e-3);
    tolerances(s, np, 1e-1);
    show(s.barycentre_of_surf(), pnt);
    show(s.maximal_section(), real);
    let mut w: Array1<'static, f64> = filled(1, 1, 0.0);
    show(s.get_minimal_weight(&mut w), |()| {
        int(w.upper());
        reals(&w);
    });
    let mut e = -7.0;
    show(s.is_constant(&mut e), |b| {
        flag(b);
        real(e);
    });
    show(s.constant_section(), gfl_curve);
    e = -7.0;
    show(s.is_conical_law(&mut e), |b| {
        flag(b);
        real(e);
    });
    show(s.circl_section(0.5), gfl_curve);
    show(s.bspline_surface(), |b| {
        flag(b.is_null());
        if let Some(b) = b.get() {
            int(b.nb_u_poles());
            int(b.nb_v_poles());
            show(rustycad_tkg3d::geom::GeomSurface::value(b, 0.3, 0.6), pnt);
        }
    });
    np
}

/// `section nparams params... lower a b`.
pub fn section_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let law = match gfl_section(&mut r) {
        Ok(s) => s,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    let s = law.get().expect("probe law");
    let params = r.reals();
    let (lower, a, b) = (r.integer(), r.real(), r.real());
    section_queries(s);
    let np = s.section_shape().map(|(p, _, _)| p).unwrap_or(0);
    for u in &params {
        section_eval(s, *u, np, lower);
        section_eval(s, *u, np - 1, 1);
    }
    show_unit(s.set_interval(a, b));
    show(s.get_interval(), |(f, l)| [f, l].into_iter().for_each(real));
    for u in &params {
        section_eval(s, *u, np, 1);
    }
    let ns: Handle<GeomFillNSections> = law.cast();
    if let Some(ns) = ns.get() {
        show_unit(ns.bspline_surface().and_then(|b| {
            ns.set_surface(&b);
            ns.compute_surface()
        }));
        section_queries(ns);
    }
    end();
}

fn mat(m: &rustycad_tkmath::gp::Matrix3) {
    m.coefficients().iter().flatten().for_each(|v| real(*v));
}

fn sentinel() -> rustycad_tkmath::gp::Matrix3 {
    rustycad_tkmath::gp::Matrix3::new([[-1.0; 3]; 3])
}

/// The law, its guided-law section settings and its stop surface.
pub struct LocationSpec {
    pub law: Handle<dyn GeomFillLocationLaw>,
    pub section: Option<(Handle<dyn GeomFillSectionLaw>, [f64; 4])>,
    pub stop: Option<Handle<dyn rustycad_tkg3d::adaptor3d::Adaptor3dSurface>>,
}

/// `ga_gfl_location`.
pub fn gfl_location(r: &mut Reader) -> Result<LocationSpec, Failure> {
    let kind = r.integer();
    if kind == 0 {
        let law = super::geom_fill_trihedron::gft_law(r)?;
        let l = GeomFillCurveAndTrihedron::new(&law);
        return Ok(LocationSpec {
            law: Handle::new(l).cast(),
            section: None,
            stop: None,
        });
    }
    if kind == 1 {
        let (d, a) = (r.dir(), r.real());
        let l = GeomFillLocationDraft::new(d, a)?;
        let stop = if r.flag() {
            Some(super::ics_geometry::surface(r))
        } else {
            None
        };
        return Ok(LocationSpec {
            law: Handle::new(l).cast(),
            section: None,
            stop,
        });
    }
    let t: Handle<dyn GeomFillTrihedronWithGuide> = super::geom_fill_trihedron::gft_law(r)?.cast();
    let section = if r.flag() {
        let sec = gfl_section(r)?;
        Some((sec, [r.real(), r.real(), r.real(), r.real()]))
    } else {
        None
    };
    let l = GeomFillLocationGuide::new(&t)?;
    Ok(LocationSpec {
        law: Handle::new(l).cast(),
        section,
        stop: None,
    })
}

fn poles2d(
    n: i32,
) -> (
    Array1<'static, rustycad_tkmath::gp::Point2>,
    [Array1<'static, rustycad_tkmath::gp::Vector2>; 2],
) {
    let neg = rustycad_tkmath::gp::Vector2::new(-1.0, -1.0);
    (
        filled(1, n, rustycad_tkmath::gp::Point2::new(-1.0, -1.0)),
        [filled(1, n, neg), filled(1, n, neg)],
    )
}

/// `ga_gfl_location_eval`.
pub fn location_eval(l: &dyn GeomFillLocationLaw, u: f64, n: i32) {
    use super::common::{pnt2d, xy, xyz};
    let s = rustycad_tkmath::gp::Vector3::new(-1.0, -1.0, -1.0);
    let mut p0 = LocationD0 {
        m: sentinel(),
        v: s,
    };
    show(l.d0(u, &mut p0), flag);
    mat(&p0.m);
    xyz(*p0.v.xyz());
    let (mut p, _) = poles2d(n);
    let mut p0 = LocationD0 {
        m: sentinel(),
        v: s,
    };
    show(l.d0_poles2d(u, &mut p0, &mut p), flag);
    mat(&p0.m);
    xyz(*p0.v.xyz());
    p.as_slice().iter().for_each(|q| pnt2d(*q));
    let (mut p, [mut dp, _]) = poles2d(n);
    let mut p1 = LocationD1 {
        m: sentinel(),
        v: s,
        dm: sentinel(),
        dv: s,
    };
    show(l.d1(u, &mut p1, &mut p, &mut dp), flag);
    mat(&p1.m);
    xyz(*p1.v.xyz());
    mat(&p1.dm);
    xyz(*p1.dv.xyz());
    for i in 1..=n {
        pnt2d(p[i]);
        xy(dp[i].xy());
    }
    let (mut p, [mut dp, mut d2p]) = poles2d(n);
    let mut p2 = LocationD2 {
        m: sentinel(),
        v: s,
        dm: sentinel(),
        dv: s,
        d2m: sentinel(),
        d2v: s,
    };
    let poles = LocationPoles2d {
        poles2d: &mut p,
        d_poles2d: &mut dp,
        d2_poles2d: &mut d2p,
    };
    show(l.d2(u, &mut p2, poles), flag);
    for (m, v) in [(&p2.m, &p2.v), (&p2.dm, &p2.dv), (&p2.d2m, &p2.d2v)] {
        mat(m);
        xyz(*v.xyz());
    }
    for i in 1..=n {
        pnt2d(p[i]);
        xy(dp[i].xy());
        xy(d2p[i].xy());
    }
    int(l.error_status() as i32);
}

/// `ga_gfl_location_queries`.
pub fn location_queries(l: &dyn GeomFillLocationLaw, res1: bool) {
    int(l.error_status() as i32);
    int(l.nb_2d_curves());
    flag(l.has_first_restriction());
    flag(l.has_last_restriction());
    int(l.trace_number());
    for shape in SHAPES {
        show(l.nb_intervals(shape), |n| {
            int(n);
            let mut t = filled(1, n + 1, -1.0);
            show_unit(l.intervals(&mut t, shape));
            reals(&t);
        });
    }
    show(l.get_domain(), |(f, g)| [f, g].into_iter().for_each(real));
    show(l.get_interval(), |(f, g)| [f, g].into_iter().for_each(real));
    show(l.get_maximal_norm(), real);
    show(l.get_average_law(), |p| {
        mat(&p.m);
        super::common::xyz(*p.v.xyz());
    });
    let mut e = -7.0;
    show(l.is_translation(&mut e), |b| {
        flag(b);
        real(e);
    });
    e = -7.0;
    show(l.is_rotation(&mut e), |b| {
        flag(b);
        real(e);
    });
    let mut c = rustycad_tkmath::gp::Point3::new(-1.0, -1.0, -1.0);
    show(l.rotation(&mut c), |()| pnt(c));
    for i in (if res1 { 1 } else { 2 })..=2 {
        show(l.resolution(i, 1e-3), |(tu, tv)| {
            [tu, tv].into_iter().for_each(real)
        });
    }
}

fn loc_function(law: &Handle<dyn GeomFillLocationLaw>, u: f64) {
    let mut f = GeomFillLocFunction::new(law);
    for order in 0..=2 {
        let mut res = [0.0; 12];
        show(f.dn(u, 0.0, 1.0, order, &mut res), |ier| {
            int(ier);
            res.iter().for_each(|v| real(*v));
        });
    }
}

fn draft_members(l: &GeomFillLocationDraft, law: &dyn GeomFillLocationLaw, params: &[f64], n: i32) {
    flag(l.is_intersec());
    super::common::dir(l.direction());
    show_unit(l.set_angle(0.2));
    if let Some(u) = params.first() {
        location_eval(law, *u, n);
    }
}

fn guide_members(
    g: &GeomFillLocationGuide,
    law: &dyn GeomFillLocationLaw,
    params: &[f64],
    n: i32,
    with_section: bool,
) {
    if with_section {
        show(g.section(), gfl_curve);
    }
    let gd = g.guide();
    let gd = gd.get().expect("probe guide");
    real(gd.first_parameter().expect("probe guide"));
    real(gd.last_parameter().expect("probe guide"));
    show(g.compute_automatic_law(), |(status, pr)| {
        int(status as i32);
        pr.as_slice().iter().for_each(|p| super::common::pnt2d(*p));
    });
    g.erase_rotation();
    int(law.error_status() as i32);
    g.set_origine(0.1, 0.8);
    if let Some(u) = params.first() {
        location_eval(law, *u, n);
    }
}

/// `law path nparams params... a b trsf [9 reals] tol3d`.
pub fn location_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let spec = match gfl_location(&mut r) {
        Ok(spec) => spec,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    let law = spec.law.clone();
    let l = law.get().expect("probe law");
    let c = super::geom_fill_trihedron::gft_curve(&mut r);
    let params = r.reals();
    let (a, b) = (r.real(), r.real());
    let trsf = if r.flag() {
        let mut m = [[0.0; 3]; 3];
        m.iter_mut().flatten().for_each(|v| *v = r.real());
        Some(rustycad_tkmath::gp::Matrix3::new(m))
    } else {
        None
    };
    let tol = r.real();
    show(l.set_curve(&c), flag);
    let draft: Handle<GeomFillLocationDraft> = law.cast();
    if let (Some(d), Some(stop)) = (draft.get(), &spec.stop) {
        show_unit(d.set_stop_surf(stop));
    }
    let guide: Handle<GeomFillLocationGuide> = law.cast();
    if let (Some(g), Some((sec, set))) = (guide.get(), &spec.section) {
        show(g.set(sec, set[0] != 0.0, set[1], set[2], set[3]), real);
    }
    location_queries(l, draft.is_null() || spec.stop.is_some());
    let n = if l.nb_2d_curves() > 0 {
        l.nb_2d_curves()
    } else {
        1
    };
    params.iter().for_each(|u| location_eval(l, *u, n));
    if let Some(u) = params.first() {
        loc_function(&law, *u);
    }
    show(l.copy(), |copy| {
        let copy = copy.get().expect("probe copy");
        flag(copy.get_curve().is_null());
        if let (true, Some(u)) = (guide.is_null(), params.first()) {
            location_eval(copy, *u, n);
        }
    });
    show_unit(l.set_interval(a, b));
    show(l.get_interval(), |(f, h)| [f, h].into_iter().for_each(real));
    params.iter().for_each(|u| location_eval(l, *u, n));
    if let Some(m) = trsf {
        show_unit(l.set_trsf(&m));
        if let Some(u) = params.first() {
            location_eval(l, *u, n);
        }
    }
    show_unit(l.set_tolerance(tol, tol));
    if let Some(d) = draft.get() {
        draft_members(d, l, &params, n);
    }
    if let Some(g) = guide.get() {
        guide_members(g, l, &params, n, spec.section.is_some());
    }
    end();
}
