//! The walking of BRepBlend (`fl_walking`); the twin of
//! tools/oracle/tkfillet_probe_walking.cxx.
use super::blend_func::{geom_guide, geom_surface, vec3, vector};
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::blend::BlendPoint;
use rustycad_tkfillet::blend_func::{ConstRad, ConstRadInv};
use rustycad_tkfillet::brep_blend::{
    BRepBlendExtremity, BRepBlendWalking, WalkingDomain, WalkingTolerances,
};
use rustycad_tkfillet::chfi_ds::{ElSpine, lock};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface, Adaptor3dTopolTool};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_surf::IntSurfTransition;
use rustycad_tkmath::gp::{Axis1, Vector2};
use rustycad_tkmath::math::Vector;
use std::rc::Rc;

fn vec2(v: Vector2) {
    real(v.x());
    real(v.y());
}

/// `fl_transition`.
fn transition(t: &IntSurfTransition) {
    int(t.transition_type() as i32);
    attempt(|| t.is_tangent().map(flag));
    attempt(|| t.situation().map(|s| int(s as i32)));
    attempt(|| t.is_opposite().map(flag));
}

/// `fl_extremity`.
pub(super) fn extremity(e: &BRepBlendExtremity) {
    pnt(e.value());
    let (u, v) = e.parameters();
    [u, v, e.parameter(), e.parameter_on_guide(), e.tolerance()]
        .into_iter()
        .for_each(real);
    flag(e.is_vertex());
    flag(e.has_tangent());
    attempt(|| e.tangent().map(vec3));
    int(e.nb_point_on_rst());
    for i in 1..=e.nb_point_on_rst() {
        let p = e.point_on_rst(i).expect("point on restriction");
        real(p.parameter_on_arc());
        transition(p.transition_on_line());
        transition(p.transition_on_arc());
    }
}

/// `fl_blend_point`.
pub(super) fn blend_point(p: &BlendPoint) {
    real(p.parameter());
    pnt(p.point_on_s1());
    pnt(p.point_on_s2());
    flag(p.is_tangency_point());
    attempt(|| {
        p.parameters_on_s1()
            .map(|(u, v)| [u, v].into_iter().for_each(real))
    });
    attempt(|| {
        p.parameters_on_s2()
            .map(|(u, v)| [u, v].into_iter().for_each(real))
    });
    if p.is_tangency_point() {
        return;
    }
    vec3(p.tangent_on_s1().expect("tangent"));
    vec3(p.tangent_on_s2().expect("tangent"));
    vec2(p.tangent2d_on_s1().expect("tangent 2d"));
    vec2(p.tangent2d_on_s2().expect("tangent 2d"));
}

/// `fl_walk`: done, twists and the line.
fn walk(w: &BRepBlendWalking) {
    flag(w.is_done());
    flag(w.twist_on_s1());
    flag(w.twist_on_s2());
    if !w.is_done() {
        return;
    }
    let line = lock(w.line().expect("line"));
    int(line.nb_points());
    for i in 1..=line.nb_points() {
        blend_point(line.point(i).expect("line point"));
    }
    extremity(line.start_point_on_first());
    extremity(line.start_point_on_second());
    extremity(line.end_point_on_first());
    extremity(line.end_point_on_second());
    attempt(|| line.transition_on_s1().map(|t| int(t as i32)));
    attempt(|| line.transition_on_s2().map(|t| int(t as i32)));
}

/// `fl_trimmed_surface`.
pub(super) fn trimmed_surface(r: &mut Reader<'_>) -> Handle<dyn Adaptor3dSurface> {
    let geom = geom_surface(r);
    let bounds = (r.real(), r.real(), r.real(), r.real());
    Handle::new(GeomAdaptorSurface::with_bounds(&geom, bounds).expect("trimmed surface")).cast()
}

/// `fl_el_spine`.
pub(super) fn el_spine(r: &mut Reader<'_>) -> Handle<ElSpine> {
    let curve = geom_guide(r);
    let spine = ElSpine::new();
    spine.set_curve(&curve).expect("spine curve");
    spine.set_first_parameter(r.real());
    spine.set_last_parameter(r.real());
    for _ in 0..r.integer() {
        let p = r.pnt();
        spine.add_vertex_with_tangent(Axis1::new(p, r.dir()));
    }
    if r.flag() {
        spine.save_first_parameter();
    }
    if r.flag() {
        spine.save_last_parameter();
    }
    Handle::new(spine)
}

fn domain(s: &Handle<dyn Adaptor3dSurface>) -> WalkingDomain {
    Rc::new(Adaptor3dTopolTool::with_surface(s).expect("domain"))
}

fn vector_of(values: &[f64]) -> Vector<'static> {
    let mut v = Vector::with_bounds(1, values.len() as i32).expect("vector");
    for (i, x) in (1..).zip(values) {
        v[i] = *x;
    }
    v
}

/// The settings of a request after the guide.
struct Request {
    mode: i32,
    pdep: f64,
    pmax: f64,
    settings: WalkingTolerances,
    appro: bool,
    x: Vector<'static>,
    pextra: f64,
}

/// The walking of a request, with its singular points and flags.
fn walking_of(
    r: &mut Reader<'_>,
    s: [&Handle<dyn Adaptor3dSurface>; 2],
    spine: &Handle<ElSpine>,
    flags: [bool; 4],
) -> BRepBlendWalking {
    let mut w = BRepBlendWalking::new(s[0], s[1], &domain(s[0]), &domain(s[1]), spine);
    for _ in 0..r.integer() {
        let (t, u1, v1, u2, v2) = (r.real(), r.real(), r.real(), r.real(), r.real());
        let p1 = s[0].get().expect("surface").value(u1, v1).expect("point");
        let p2 = s[1].get().expect("surface").value(u2, v2).expect("point");
        w.add_singular_point(&BlendPoint::on_surfaces(p1, p2, t, [u1, v1, u2, v2]));
    }
    w.classification_on_s1(flags[0]);
    w.classification_on_s2(flags[1]);
    w.check_2d(flags[2]);
    w.check(flags[3]);
    w
}

/// `fl_walking`: see tools/oracle/tkfillet_probe_walking.cxx.
pub fn walking_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let mode = r.integer();
    let (s1, s2) = (trimmed_surface(r), trimmed_surface(r));
    let spine = el_spine(r);
    let (radius, choix) = (r.real(), r.integer());
    let (pdep, pmax) = (r.real(), r.real());
    let settings = WalkingTolerances {
        max_step: r.real(),
        tol3d: r.real(),
        tol_guide: r.real(),
        fleche: r.real(),
    };
    let appro = r.flag();
    let x = vector_of(&[r.real(), r.real(), r.real(), r.real()]);
    let flags = [r.flag(), r.flag(), r.flag(), r.flag()];
    let pextra = r.real();
    let guide: Handle<dyn Adaptor3dCurve> = spine.clone().cast();
    let mut f = ConstRad::new(&s1, &s2, &guide).expect("function");
    f.set_radius(radius, choix);
    let mut finv = ConstRadInv::new(&s1, &s2, &guide);
    finv.set_radius(radius, choix);
    let mut w = walking_of(r, [&s1, &s2], &spine, flags);
    attempt(|| w.line().map(|_| ()));
    let request = Request {
        mode,
        pdep,
        pmax,
        settings,
        appro,
        x,
        pextra,
    };
    run(&mut w, &mut f, &mut finv, request);
    end();
}

fn run(w: &mut BRepBlendWalking, f: &mut ConstRad, finv: &mut ConstRadInv, q: Request) {
    let tols = (q.settings.tol3d, q.settings.tol_guide);
    if q.mode == 5 {
        let mut x = q.x;
        attempt(|| {
            let (ok, p1, p2) = w.perform_first_section(f, q.pdep, &mut x, tols)?;
            flag(ok);
            int(p1 as i32);
            int(p2 as i32);
            vector(&x);
            Ok(())
        });
        return;
    }
    if q.mode == 6 {
        run_on_restrictions(w, f, finv, &q);
        return;
    }
    attempt(|| w.perform(f, finv, (q.pdep, q.pmax), q.settings, &q.x, q.appro));
    walk(w);
    let more: Option<Result<bool, Failure>> = match q.mode {
        1 => Some(w.complete(f, finv, q.pextra)),
        2 => Some(w.continu(f, finv, q.pextra)),
        3 => Some(w.continu_on(f, finv, q.pextra, true)),
        4 => Some(w.continu_on(f, finv, q.pextra, false)),
        _ => None,
    };
    if let Some(result) = more {
        attempt(|| result.map(flag));
        walk(w);
    }
}

/// Mode 6: the first section on both restrictions, then the walking from it.
fn run_on_restrictions(
    w: &mut BRepBlendWalking,
    f: &mut ConstRad,
    finv: &mut ConstRadInv,
    q: &Request,
) {
    let tols = (q.settings.tol3d, q.settings.tol_guide);
    let mut found = None;
    attempt(|| {
        let first = w.perform_first_section_on_restrictions(
            f,
            finv,
            (q.pdep, q.pmax),
            &q.x,
            tols,
            (true, true),
        )?;
        flag(first.is_some());
        let (psol, sol) = first.unwrap_or((0.0, [0.0; 4]));
        real(psol);
        sol.into_iter().for_each(real);
        found = first;
        Ok(())
    });
    if let Some((psol, sol)) = found {
        let sol = vector_of(&sol);
        attempt(|| w.perform(f, finv, (psol, q.pextra), q.settings, &sol, false));
    }
    walk(w);
}
