//! IntImpParGen probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intimppargen.cxx.
use super::common::{Reader, attempt, end, flag, int, real, show};
use super::int_res2d::{read_raw_domain, trans};
use rustycad_tkgeomalgo::int_curve::{determine_transition_lc, normalize_on_circle_domain};
use rustycad_tkgeomalgo::int_imp_par_gen::{
    TransitionSide, determine_crossing_transition, determine_position, determine_transition,
    normalize_on_domain, tool,
};
use rustycad_tkgeomalgo::int_res2d::{Position, Transition};
use rustycad_tkmath::gp::{Point2, Vector2};

pub fn vec2d(r: &mut Reader<'_>) -> Vector2 {
    let x = r.real();
    Vector2::new(x, r.real())
}
pub fn v2(v: Vector2) {
    real(v.x());
    real(v.y());
}
fn position(code: i32) -> Position {
    [Position::Head, Position::Middle, Position::End][code as usize]
}

type Side = (Position, Vector2, Vector2);

fn side<'a>(
    (pos, _, norm): Side,
    tan: &'a mut Vector2,
    trans: &'a mut Transition,
) -> TransitionSide<'a> {
    TransitionSide {
        pos,
        tan,
        norm,
        trans,
    }
}

/// `pos1 tan1 norm1 pos2 tan2 norm2`: the four DetermineTransition variants.
pub fn transition_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s1: Side = (position(r.integer()), vec2d(&mut r), vec2d(&mut r));
    let s2: Side = (position(r.integer()), vec2d(&mut r), vec2d(&mut r));
    attempt(|| {
        let (mut a, mut b, mut x, mut y) = (s1.1, s2.1, Transition::new(), Transition::new());
        determine_transition(side(s1, &mut a, &mut x), side(s2, &mut b, &mut y), 1e-7);
        trans(&x)?;
        trans(&y)?;
        v2(a);
        v2(b);
        let (mut x, mut y) = (Transition::new(), Transition::new());
        flag(determine_crossing_transition(
            (s1.0, s1.1, &mut x),
            (s2.0, s2.1, &mut y),
            1e-7,
        ));
        trans(&x)?;
        trans(&y)?;
        let (mut a, mut b, mut x, mut y) = (s1.1, s2.1, Transition::new(), Transition::new());
        tool::determine_transition(side(s1, &mut a, &mut x), side(s2, &mut b, &mut y), 1e-7);
        trans(&x)?;
        trans(&y)?;
        v2(a);
        v2(b);
        Ok(())
    });
    attempt(|| {
        let (mut a, mut b, mut x, mut y) = (s1.1, s2.1, Transition::new(), Transition::new());
        determine_transition_lc(
            (s1.0, &mut a, s1.2, &mut x),
            (s2.0, &mut b, s2.2, &mut y),
            1e-7,
        )?;
        trans(&x)?;
        trans(&y)?;
        v2(a);
        Ok(())
    });
    end();
}

/// `domain x y param`: positions and normalisations on a domain.
pub fn domain_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mut d, closed, z, zp) = read_raw_domain(&mut r, |u| Point2::new(u, 0.0));
    let point: Point2 = r.pnt2d();
    let param: f64 = r.real();
    if closed {
        attempt(|| d.set_equivalent_parameters(z, zp));
    }
    show(determine_position(&d, point, param).map(|p| p as i32), int);
    show(
        tool::determine_position(&d, point, param).map(|p| p as i32),
        int,
    );
    show(normalize_on_domain(param, &d), real);
    show(tool::normalize_on_domain(param, &d), real);
    show(normalize_on_circle_domain(param, &d), real);
    end();
}
