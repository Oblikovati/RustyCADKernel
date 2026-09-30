//! Contap walking-line template specialization against native state transitions.
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, xyz};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::{
    int_surf::{IntSurfPathPoint, IntSurfPntOn2S},
    int_walk::TheIWLine,
};
use rustycad_tkhlr::contap::ContapTheIWLineOfTheIWalking;
use rustycad_tkmath::gp::Vector3;
fn pnt_on_2s(r: &mut Reader) -> IntSurfPntOn2S {
    let x = r.pnt();
    let (a, b, e, d) = (r.real(), r.real(), r.real(), r.real());
    let mut q = IntSurfPntOn2S::new();
    q.set_value_all(x, a, b, e, d);
    q
}
pub fn pnt2s(p: &IntSurfPntOn2S) {
    pnt(p.value());
    let (a, b, c, d) = p.parameters();
    [a, b, c, d].into_iter().for_each(real);
}
pub fn line_report(l: &ContapTheIWLineOfTheIWalking) {
    let n = l.nb_points();
    int(n);
    for i in 1..=n {
        pnt2s(&l.value(i).expect("probe line point"));
    }
    flag(l.is_closed());
    flag(l.has_first_point());
    flag(l.has_last_point());
    show(l.first_point_index(), int);
    show(l.first_point(), |p| pnt(p.value()));
    show(l.last_point_index(), int);
    show(l.last_point(), |p| pnt(p.value()));
    int(l.nb_passing_point());
    for i in 1..=l.nb_passing_point() {
        let (a, b) = l.passing_point(i).expect("probe passing point");
        int(a);
        int(b);
    }
    let (v, idx) = l.tangent_vector();
    xyz(*v.xyz());
    int(idx);
    flag(l.is_tangent_at_begining());
    flag(l.is_tangent_at_end());
}

fn path_point_of(r: &mut Reader) -> IntSurfPathPoint {
    let x = r.pnt();
    let u = r.real();
    IntSurfPathPoint::with_value(x, u, r.real())
}

/// One command of `ga_ip_iwline_case`.
fn iwline_command(r: &mut Reader, l: &ContapTheIWLineOfTheIWalking, c: i32) -> Result<(), Failure> {
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
    let l = ContapTheIWLineOfTheIWalking::with_inc_allocator();
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
