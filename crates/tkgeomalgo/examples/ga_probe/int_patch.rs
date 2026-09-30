//! IntPatch record probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch.cxx: the points, the walking line of
//! `IntPatchTheIWalking` and the path points and segments of `IntPatchTheSOnBounds`,
//! edited by command scripts over the same fixture of arcs and vertices.
use super::common::{Reader, attempt, captured, end, flag, int, pnt, real};
use super::int_walk::pnt2s;
use super::int_walk_iw::line_report;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dHVertex;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchPoint, IntPatchTheIWLineOfTheIWalking, IntPatchThePathPointOfTheSOnBounds,
    IntPatchTheSegmentOfTheSOnBounds,
};
use rustycad_tkgeomalgo::int_start::{TheArc, TheVertex};
use rustycad_tkgeomalgo::int_surf::{
    IntSurfPathPoint, IntSurfPntOn2S, IntSurfSituation, IntSurfTransition, IntSurfTypeTrans,
};
use rustycad_tkgeomalgo::int_walk::TheIWLine;
use rustycad_tkmath::gp::{Direction2, Point2, Vector3};

/// GaIpFixture: the arcs and vertices of rank 1 to 3 (rank 0 is the null handle).
pub struct Fixture {
    arcs: Vec<TheArc>,
    vtx: Vec<TheVertex>,
}

impl Fixture {
    pub fn new() -> Self {
        let mut arcs: Vec<TheArc> = vec![Handle::default()];
        let mut vtx: Vec<TheVertex> = vec![Handle::default()];
        for k in 1..=3 {
            let dir = Direction2::new(0.0, 1.0).expect("probe direction");
            let line: Handle<dyn Geom2dCurve> =
                Handle::new(Geom2dLine::from_point(Point2::new(f64::from(k), 0.0), dir)).cast();
            let adaptor = Geom2dAdaptorCurve::with_range(&line, 0.0, 10.0).expect("probe arc");
            arcs.push(Handle::new(adaptor).cast());
            let p = Point2::new(f64::from(k), 0.5);
            vtx.push(Handle::new(Adaptor3dHVertex::new(
                p,
                Orientation::Forward,
                1.0e-7 * f64::from(k),
            )));
        }
        Self { arcs, vtx }
    }
    fn slot(k: i32) -> usize {
        if (0..=3).contains(&k) { k as usize } else { 0 }
    }
    pub fn arc(&self, k: i32) -> TheArc {
        self.arcs[Self::slot(k)].clone()
    }
    pub fn vertex(&self, k: i32) -> TheVertex {
        self.vtx[Self::slot(k)].clone()
    }
    pub fn arc_rank(&self, a: &TheArc) -> i32 {
        self.arcs
            .iter()
            .position(|x| x == a)
            .map_or(-1, |k| k as i32)
    }
    pub fn vertex_rank(&self, v: &TheVertex) -> i32 {
        self.vtx
            .iter()
            .position(|x| x == v)
            .map_or(-1, |k| k as i32)
    }
}

/// `ga_ip_transition`.
pub fn transition(c: i32) -> IntSurfTransition {
    match c {
        1 => IntSurfTransition::with_type(false, IntSurfTypeTrans::In),
        2 => IntSurfTransition::with_type(false, IntSurfTypeTrans::Out),
        3 => IntSurfTransition::with_type(true, IntSurfTypeTrans::In),
        4 => IntSurfTransition::with_type(true, IntSurfTypeTrans::Out),
        5 => IntSurfTransition::touch(true, IntSurfSituation::Inside, false),
        6 => IntSurfTransition::touch(true, IntSurfSituation::Outside, true),
        7 => IntSurfTransition::touch(false, IntSurfSituation::Unknown, false),
        _ => IntSurfTransition::new(),
    }
}

pub fn trans(t: &IntSurfTransition) {
    int(t.transition_type() as i32);
    attempt(|| t.is_tangent().map(flag));
    attempt(|| t.situation().map(|s| int(s as i32)));
    attempt(|| t.is_opposite().map(flag));
}

fn pnt_on_2s(r: &mut Reader) -> IntSurfPntOn2S {
    let x = r.pnt();
    let (a, b, e, d) = (r.real(), r.real(), r.real(), r.real());
    let mut q = IntSurfPntOn2S::new();
    q.set_value_all(x, a, b, e, d);
    q
}

/// `ga_ip_point_command`.
fn point_command(r: &mut Reader, fx: &Fixture, p: &mut IntPatchPoint, c: i32) {
    match c {
        1 => {
            let x = r.pnt();
            let tol = r.real();
            p.set_value_tangent(x, tol, r.flag());
        }
        2 => p.set_value(r.pnt()),
        3 => p.set_pnt_on_2s(&pnt_on_2s(r)),
        4 => p.set_tolerance(r.real()),
        5 => {
            let (a, b, e) = (r.real(), r.real(), r.real());
            p.set_parameters(a, b, e, r.real());
        }
        6 => p.set_parameter(r.real()),
        7 => {
            let first = r.flag();
            p.set_vertex(first, &fx.vertex(r.integer()));
        }
        8 => {
            let (first, a, prm) = (r.flag(), r.integer(), r.real());
            let tl = transition(r.integer());
            p.set_arc(first, &fx.arc(a), prm, &tl, &transition(r.integer()));
        }
        9 => p.set_multiple(r.flag()),
        _ => p.reverse_transition(),
    }
}

/// `ga_ip_read_point`.
pub fn read_point(r: &mut Reader, fx: &Fixture) -> IntPatchPoint {
    let mut p = IntPatchPoint::new();
    for _ in 0..r.integer() {
        let c = r.integer();
        point_command(r, fx, &mut p, c);
    }
    p
}

/// `ga_ip_point`.
pub fn point(fx: &Fixture, p: &IntPatchPoint) {
    pnt(p.value());
    real(p.parameter_on_line());
    real(p.tolerance());
    flag(p.is_tangency_point());
    flag(p.is_multiple());
    let ((u1, v1), (u2, v2)) = (p.parameters_on_s1(), p.parameters_on_s2());
    [u1, v1, u2, v2].into_iter().for_each(real);
    pnt2s(&p.pnt_on_2s());
    let (a, b, c, d) = p.parameters();
    [a, b, c, d].into_iter().for_each(real);
    flag(p.is_on_dom_s1());
    flag(p.is_vertex_on_s1());
    attempt(|| p.vertex_on_s1().map(|v| int(fx.vertex_rank(&v))));
    attempt(|| p.arc_on_s1().map(|a| int(fx.arc_rank(&a))));
    attempt(|| p.transition_line_arc1().map(|t| trans(&t)));
    attempt(|| p.transition_on_s1().map(|t| trans(&t)));
    attempt(|| p.parameter_on_arc1().map(real));
    flag(p.is_on_dom_s2());
    flag(p.is_vertex_on_s2());
    attempt(|| p.vertex_on_s2().map(|v| int(fx.vertex_rank(&v))));
    attempt(|| p.arc_on_s2().map(|a| int(fx.arc_rank(&a))));
    attempt(|| p.transition_line_arc2().map(|t| trans(&t)));
    attempt(|| p.transition_on_s2().map(|t| trans(&t)));
    attempt(|| p.parameter_on_arc2().map(real));
}

/// `ga_ip_point_case`.
pub fn point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    let dump = r.flag();
    let p = read_point(&mut r, &fx);
    point(&fx, &p);
    let mut q = p.clone();
    q.reverse_transition();
    point(&fx, &q);
    if dump {
        captured(|out| p.dump(out));
    }
    end();
}

fn path_point_of(r: &mut Reader) -> IntSurfPathPoint {
    let x = r.pnt();
    let u = r.real();
    IntSurfPathPoint::with_value(x, u, r.real())
}

/// One command of `ga_ip_iwline_case`.
fn iwline_command(
    r: &mut Reader,
    l: &IntPatchTheIWLineOfTheIWalking,
    c: i32,
) -> Result<(), Failure> {
    match c {
        1 => l.add_point(&pnt_on_2s(r)),
        2 => {
            let cl = r.flag();
            l.add_status_first(cl, r.flag());
        }
        3 => {
            let (cl, hf, k) = (r.flag(), r.flag(), r.integer());
            l.add_status_first_point(cl, hf, k, &path_point_of(r));
        }
        4 => {
            let (cl, hf) = (r.flag(), r.flag());
            l.add_status_first_last(cl, hf, r.flag());
        }
        5 => l.add_status_last(r.flag()),
        6 => {
            let (hl, k) = (r.flag(), r.integer());
            l.add_status_last_point(hl, k, &path_point_of(r));
        }
        7 => l.add_index_passing(r.integer()),
        8 => {
            let v = Vector3::from(r.xyz());
            l.set_tangent_vector(v, r.integer());
        }
        9 => l.set_tangency_at_begining(r.flag()),
        10 => l.set_tangency_at_end(r.flag()),
        11 => l.reverse(),
        _ => l.cut(r.integer())?,
    }
    Ok(())
}

/// `ga_ip_iwline_case`.
pub fn iwline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let l = IntPatchTheIWLineOfTheIWalking::with_inc_allocator();
    for _ in 0..r.integer() {
        let c = r.integer();
        attempt(|| iwline_command(&mut r, &l, c));
    }
    line_report(&l);
    int(l.line().nb_points());
    attempt(|| l.value(0).map(|_| ()));
    attempt(|| l.value(l.nb_points() + 1).map(|_| ()));
    attempt(|| l.passing_point(0).map(|_| ()));
    attempt(|| l.passing_point(l.nb_passing_point() + 1).map(|_| ()));
    end();
}

fn path_point(fx: &Fixture, p: &IntPatchThePathPointOfTheSOnBounds) {
    pnt(p.value());
    real(p.tolerance());
    flag(p.is_new());
    int(fx.arc_rank(&p.arc()));
    real(p.parameter());
    attempt(|| p.vertex().map(|v| int(fx.vertex_rank(&v))));
}

fn segment_points(fx: &Fixture, s: &IntPatchTheSegmentOfTheSOnBounds) {
    attempt(|| s.first_point().map(|p| path_point(fx, p)));
    attempt(|| s.last_point().map(|p| path_point(fx, p)));
}

/// `ga_ip_pathpoint_case`.
pub fn pathpoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    let (x, tol, v, a, prm) = (r.pnt(), r.real(), r.integer(), r.integer(), r.real());
    let (x2, tol2, a2, prm2) = (r.pnt(), r.real(), r.integer(), r.real());
    let (sa, f, l) = (r.integer(), r.integer(), r.integer());
    let p1 = IntPatchThePathPointOfTheSOnBounds::on_vertex(x, tol, &fx.vertex(v), &fx.arc(a), prm);
    let p2 = IntPatchThePathPointOfTheSOnBounds::on_arc(x2, tol2, &fx.arc(a2), prm2);
    path_point(&fx, &IntPatchThePathPointOfTheSOnBounds::new());
    path_point(&fx, &p1);
    path_point(&fx, &p2);
    let mut q = p1.clone();
    q.set_value_on_arc(x2, tol2, &fx.arc(a2), prm2);
    path_point(&fx, &q);
    q.set_value_on_vertex(x, tol, &fx.vertex(v), &fx.arc(a), prm);
    path_point(&fx, &q);
    let mut s = IntPatchTheSegmentOfTheSOnBounds::new();
    int(fx.arc_rank(&s.curve()));
    flag(s.has_first_point());
    flag(s.has_last_point());
    segment_points(&fx, &s);
    s.set_value(&fx.arc(sa));
    let pick = |k: i32| if k == 1 { &p1 } else { &p2 };
    if f != 0 {
        s.set_limit_point(pick(f), true);
    }
    if l != 0 {
        s.set_limit_point(pick(l), false);
    }
    int(fx.arc_rank(&s.curve()));
    flag(s.has_first_point());
    flag(s.has_last_point());
    segment_points(&fx, &s);
    s.set_value(&fx.arc(sa));
    flag(s.has_first_point());
    flag(s.has_last_point());
    attempt(|| s.first_point().map(|p| path_point(&fx, p)));
    end();
}
