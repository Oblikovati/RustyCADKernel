//! GccAna probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_gccana.cxx.
//! Every solver request is `kind detail arguments... tolerance`; `detail` selects the
//! consultations printed per solution (bit 1 qualifiers, bit 2 first argument, bit 4
//! second, bit 8 third or centre).
use super::common::{Reader, end, flag, int, pnt2d, real, show};
use super::gcc_ent::{circ2d, position};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::gcc_ana::{
    Circ2d2TanOn, Circ2d2TanRad, Circ2d3Tan, Circ2dTanCen, Circ2dTanOnRad,
};
use rustycad_tkgeomalgo::gcc_ent::{Position, QualifiedCirc, QualifiedLin};
use rustycad_tkmath::gp::Point2;

type Tangency = (f64, f64, Point2);

impl Reader<'_> {
    /// `position x y dx dy radius`.
    pub(super) fn qcirc(&mut self) -> QualifiedCirc {
        let p = Position::ALL[self.integer() as usize];
        QualifiedCirc::new(self.circ2d(), p)
    }
    /// `position x y dx dy`.
    pub(super) fn qlin(&mut self) -> QualifiedLin {
        let p = Position::ALL[self.integer() as usize];
        QualifiedLin::new(self.lin2d(), p)
    }
}

pub(super) fn tangency((s, a, p): Tangency) {
    real(s);
    real(a);
    pnt2d(p);
}

// GA_SAME_TANGENCY: IsTheSame, then the tangency when it is false.
fn same_tangency(same: Result<bool, Failure>, read: impl FnOnce() -> Result<Tangency, Failure>) {
    let mut is_same = true;
    show(same, |s| {
        is_same = s;
        flag(s);
    });
    if !is_same {
        show(read(), tangency);
    }
}

pub(super) fn qualifiers2(value: (Position, Position)) {
    position(value.0);
    position(value.1);
}

fn centre_on((p, q): (f64, Point2)) {
    real(p);
    pnt2d(q);
}

// Run a solver constructor, printing its exception type when it raises.
pub(super) fn solve<S>(built: Result<S, Failure>, report: impl FnOnce(&S)) {
    show(built, |s| report(&s));
}

fn circ3_report(s: &Circ2d3Tan, detail: i32) {
    flag(s.is_done());
    int(s.nb_solutions());
    for i in 0..=s.nb_solutions() + 1 {
        show(s.this_solution(i), circ2d);
        if detail & 1 != 0 {
            show(s.which_qualifier(i), |(a, b, c)| {
                position(a);
                position(b);
                position(c);
            });
        }
        if detail & 2 != 0 {
            same_tangency(s.is_the_same1(i), || s.tangency1(i));
        }
        if detail & 4 != 0 {
            same_tangency(s.is_the_same2(i), || s.tangency2(i));
        }
        if detail & 8 != 0 {
            same_tangency(s.is_the_same3(i), || s.tangency3(i));
        }
    }
}

/// kind: 0 ccc, 1 ccl, 2 cll, 3 lll, 4 ccp, 5 clp, 6 llp, 7 cpp, 8 lpp, 9 ppp.
pub fn circ3tan_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    let built: Result<Circ2d3Tan, Failure> = match kind {
        0 => {
            let (a, b, c) = (r.qcirc(), r.qcirc(), r.qcirc());
            Circ2d3Tan::circ_circ_circ(a, b, c, r.real())
        }
        1 => {
            let (a, b, c) = (r.qcirc(), r.qcirc(), r.qlin());
            Circ2d3Tan::circ_circ_lin(a, b, c, r.real())
        }
        2 => {
            let (a, b, c) = (r.qcirc(), r.qlin(), r.qlin());
            Circ2d3Tan::circ_lin_lin(a, b, c, r.real())
        }
        3 => {
            let (a, b, c) = (r.qlin(), r.qlin(), r.qlin());
            Circ2d3Tan::lin_lin_lin(a, b, c, r.real())
        }
        4 => {
            let (a, b, c) = (r.qcirc(), r.qcirc(), r.pnt2d());
            Circ2d3Tan::circ_circ_pnt(a, b, c, r.real())
        }
        5 => {
            let (a, b, c) = (r.qcirc(), r.qlin(), r.pnt2d());
            Circ2d3Tan::circ_lin_pnt(a, b, c, r.real())
        }
        6 => {
            let (a, b, c) = (r.qlin(), r.qlin(), r.pnt2d());
            Circ2d3Tan::lin_lin_pnt(a, b, c, r.real())
        }
        7 => {
            let (a, b, c) = (r.qcirc(), r.pnt2d(), r.pnt2d());
            Circ2d3Tan::circ_pnt_pnt(a, b, c, r.real())
        }
        8 => {
            let (a, b, c) = (r.qlin(), r.pnt2d(), r.pnt2d());
            Circ2d3Tan::lin_pnt_pnt(a, b, c, r.real())
        }
        _ => {
            let (a, b, c) = (r.pnt2d(), r.pnt2d(), r.pnt2d());
            Circ2d3Tan::pnt_pnt_pnt(a, b, c, r.real())
        }
    };
    solve(built, |s| circ3_report(s, detail));
    end();
}

fn tanrad_report(s: &Circ2d2TanRad, detail: i32) {
    flag(s.is_done());
    int(s.nb_solutions());
    for i in 0..=s.nb_solutions() + 1 {
        show(s.this_solution(i), circ2d);
        if detail & 1 != 0 {
            show(s.which_qualifier(i), qualifiers2);
        }
        if detail & 2 != 0 {
            same_tangency(s.is_the_same1(i), || s.tangency1(i));
        }
        if detail & 4 != 0 {
            same_tangency(s.is_the_same2(i), || s.tangency2(i));
        }
    }
}

/// kind: 0 cc, 1 cl, 2 cp, 3 lp, 4 ll, 5 pp; then the radius and the tolerance.
pub fn tanrad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    let built: Result<Circ2d2TanRad, Failure> = match kind {
        0 => {
            let (a, b, rad) = (r.qcirc(), r.qcirc(), r.real());
            Circ2d2TanRad::circ_circ(a, b, rad, r.real())
        }
        1 => {
            let (a, b, rad) = (r.qcirc(), r.qlin(), r.real());
            Circ2d2TanRad::circ_lin(a, b, rad, r.real())
        }
        2 => {
            let (a, b, rad) = (r.qcirc(), r.pnt2d(), r.real());
            Circ2d2TanRad::circ_pnt(a, b, rad, r.real())
        }
        3 => {
            let (a, b, rad) = (r.qlin(), r.pnt2d(), r.real());
            Circ2d2TanRad::lin_pnt(a, b, rad, r.real())
        }
        4 => {
            let (a, b, rad) = (r.qlin(), r.qlin(), r.real());
            Circ2d2TanRad::lin_lin(a, b, rad, r.real())
        }
        _ => {
            let (a, b, rad) = (r.pnt2d(), r.pnt2d(), r.real());
            Circ2d2TanRad::pnt_pnt(a, b, rad, r.real())
        }
    };
    solve(built, |s| tanrad_report(s, detail));
    end();
}

fn tanon_report(s: &Circ2d2TanOn, detail: i32) {
    flag(s.is_done());
    int(s.nb_solutions());
    for i in 0..=s.nb_solutions() + 1 {
        show(s.this_solution(i), circ2d);
        if detail & 1 != 0 {
            show(s.which_qualifier(i), qualifiers2);
        }
        if detail & 2 != 0 {
            same_tangency(s.is_the_same1(i), || s.tangency1(i));
        }
        if detail & 4 != 0 {
            same_tangency(s.is_the_same2(i), || s.tangency2(i));
        }
        if detail & 8 != 0 {
            show(s.center_on3(i), centre_on);
        }
    }
}

// The two arguments of a Circ2d2TanOn request: 0 cc, 1 cl, 2 ll, 3 cp, 4 lp, 5 pp.
enum Pair {
    Cc(QualifiedCirc, QualifiedCirc),
    Cl(QualifiedCirc, QualifiedLin),
    Ll(QualifiedLin, QualifiedLin),
    Cp(QualifiedCirc, Point2),
    Lp(QualifiedLin, Point2),
    Pp(Point2, Point2),
}

fn read_pair(r: &mut Reader, args: i32) -> Pair {
    match args {
        0 => Pair::Cc(r.qcirc(), r.qcirc()),
        1 => Pair::Cl(r.qcirc(), r.qlin()),
        2 => Pair::Ll(r.qlin(), r.qlin()),
        3 => Pair::Cp(r.qcirc(), r.pnt2d()),
        4 => Pair::Lp(r.qlin(), r.pnt2d()),
        _ => Pair::Pp(r.pnt2d(), r.pnt2d()),
    }
}

/// kind = 6 * on + args, `on` 0 line, 1 circle.
pub fn tanon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    let pair = read_pair(&mut r, kind % 6);
    let built: Result<Circ2d2TanOn, Failure> = if kind / 6 == 0 {
        let (l, tol) = (r.lin2d(), r.real());
        match pair {
            Pair::Cc(a, b) => Circ2d2TanOn::circ_circ_on_line(a, b, l, tol),
            Pair::Cl(a, b) => Circ2d2TanOn::circ_lin_on_line(a, b, l, tol),
            Pair::Ll(a, b) => Circ2d2TanOn::lin_lin_on_line(a, b, l, tol),
            Pair::Cp(a, b) => Circ2d2TanOn::circ_pnt_on_line(a, b, l, tol),
            Pair::Lp(a, b) => Circ2d2TanOn::lin_pnt_on_line(a, b, l, tol),
            Pair::Pp(a, b) => Circ2d2TanOn::pnt_pnt_on_line(a, b, l, tol),
        }
    } else {
        let (c, tol) = (r.circ2d(), r.real());
        match pair {
            Pair::Cc(a, b) => Circ2d2TanOn::circ_circ_on_circ(a, b, c, tol),
            Pair::Cl(a, b) => Circ2d2TanOn::circ_lin_on_circ(a, b, c, tol),
            Pair::Ll(a, b) => Circ2d2TanOn::lin_lin_on_circ(a, b, c, tol),
            Pair::Cp(a, b) => Circ2d2TanOn::circ_pnt_on_circ(a, b, c, tol),
            Pair::Lp(a, b) => Circ2d2TanOn::lin_pnt_on_circ(a, b, c, tol),
            Pair::Pp(a, b) => Circ2d2TanOn::pnt_pnt_on_circ(a, b, c, tol),
        }
    };
    solve(built, |s| tanon_report(s, detail));
    end();
}

fn tanonrad_report(s: &Circ2dTanOnRad, detail: i32) {
    flag(s.is_done());
    int(s.nb_solutions());
    for i in 0..=s.nb_solutions() + 1 {
        show(s.this_solution(i), circ2d);
        if detail & 1 != 0 {
            show(s.which_qualifier(i), position);
        }
        if detail & 2 != 0 {
            same_tangency(s.is_the_same1(i), || s.tangency1(i));
        }
        if detail & 8 != 0 {
            show(s.center_on3(i), centre_on);
        }
    }
}

/// kind = 3 * on + arg: `on` 0 line, 1 circle; `arg` 0 circle, 1 line, 2 point.
pub fn tanonrad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    let built: Result<Circ2dTanOnRad, Failure> = match (kind / 3, kind % 3) {
        (0, 0) => {
            let (a, l, rad) = (r.qcirc(), r.lin2d(), r.real());
            Circ2dTanOnRad::circ_on_line(a, l, rad, r.real())
        }
        (0, 1) => {
            let (a, l, rad) = (r.qlin(), r.lin2d(), r.real());
            Circ2dTanOnRad::lin_on_line(a, l, rad, r.real())
        }
        (0, _) => {
            let (a, l, rad) = (r.pnt2d(), r.lin2d(), r.real());
            Circ2dTanOnRad::pnt_on_line(a, l, rad, r.real())
        }
        (_, 0) => {
            let (a, c, rad) = (r.qcirc(), r.circ2d(), r.real());
            Circ2dTanOnRad::circ_on_circ(a, c, rad, r.real())
        }
        (_, 1) => {
            let (a, c, rad) = (r.qlin(), r.circ2d(), r.real());
            Circ2dTanOnRad::lin_on_circ(a, c, rad, r.real())
        }
        _ => {
            let (a, c, rad) = (r.pnt2d(), r.circ2d(), r.real());
            Circ2dTanOnRad::pnt_on_circ(a, c, rad, r.real())
        }
    };
    solve(built, |s| tanonrad_report(s, detail));
    end();
}

/// kind 0 `qcirc centre tol`, 1 `line centre`, 2 `point centre`.
pub fn tancen_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    let built: Result<Circ2dTanCen, Failure> = match kind {
        0 => {
            let (a, c) = (r.qcirc(), r.pnt2d());
            Circ2dTanCen::tangent_to_circle(a, c, r.real())
        }
        1 => {
            let l = r.lin2d();
            Circ2dTanCen::tangent_to_line(l, r.pnt2d())
        }
        _ => {
            let p = r.pnt2d();
            Circ2dTanCen::through_point(p, r.pnt2d())
        }
    };
    solve(built, |s| {
        flag(s.is_done());
        int(s.nb_solutions());
        for i in 0..=s.nb_solutions() + 1 {
            show(s.this_solution(i), circ2d);
            if detail & 1 != 0 {
                show(s.which_qualifier(i), position);
            }
            if detail & 2 != 0 {
                same_tangency(s.is_the_same1(i), || s.tangency1(i));
            }
        }
    });
    end();
}
