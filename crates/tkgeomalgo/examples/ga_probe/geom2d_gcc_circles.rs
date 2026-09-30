//! Geom2dGcc circle solver probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geom2dgcc_circles.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show};
use super::gcc_ana::{qualifiers2, tangency};
use super::gcc_ent::{circ2d, position};
use super::geom2d_hatch::hatch_adaptor;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::gcc_ana;
use rustycad_tkgeomalgo::gcc_ent::Position;
use rustycad_tkgeomalgo::geom2d_gcc::{
    Circ2d2TanOn, Circ2d2TanOnGeo, Circ2d2TanOnIter, Circ2d2TanRad, Circ2d2TanRadGeo, Circ2d3Tan,
    Circ2d3TanIter, Circ2dTanCen, Circ2dTanCenGeo,
};
use rustycad_tkmath::gp::{Circle2, Point2};

type Tangency = (f64, f64, Point2);

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

fn centre((a, p): (f64, Point2)) {
    real(a);
    pnt2d(p);
}

/// `detail q1 x y tol`: Geom2dGcc_Circ2dTanCen.
pub fn tancen_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let detail = r.integer();
    attempt(|| {
        let q1 = r.g2gcc_q()?;
        let p = r.g2gcc_point();
        let s = Circ2dTanCen::new(&q1, &p, r.real())?;
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
        Ok(())
    });
    end();
}

/// `detail q1 x y tol`: Geom2dGcc_Circ2dTanCenGeo.
pub fn tancen_geo_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let detail = r.integer();
    attempt(|| {
        let q1 = r.g2gcc_q()?;
        let p = r.pnt2d();
        let s = Circ2dTanCenGeo::new(&q1, p, r.real())?;
        flag(s.is_done());
        int(s.nb_solutions());
        for i in 0..=s.nb_solutions() + 1 {
            show(s.this_solution(i), circ2d);
            if detail & 1 != 0 {
                show(s.which_qualifier(i), position);
            }
            if detail & 2 != 0 {
                show(s.tangency1(i), tangency);
            }
        }
        Ok(())
    });
    end();
}

// The consultations of the two-argument circle solvers.
trait TwoTan {
    fn is_done(&self) -> bool;
    fn nb_solutions(&self) -> i32;
    fn this_solution(&self, i: i32) -> Result<Circle2, Failure>;
    fn which_qualifier(&self, i: i32) -> Result<(Position, Position), Failure>;
    fn same(&self, arg: i32, i: i32) -> Result<bool, Failure>;
    fn tangency(&self, arg: i32, i: i32) -> Result<Tangency, Failure>;
    fn center_on3(&self, _i: i32) -> Option<Result<(f64, Point2), Failure>> {
        None
    }
}

macro_rules! two_tan {
    ($t:ty, $centre:expr) => {
        impl TwoTan for $t {
            fn is_done(&self) -> bool {
                <$t>::is_done(self)
            }
            fn nb_solutions(&self) -> i32 {
                <$t>::nb_solutions(self)
            }
            fn this_solution(&self, i: i32) -> Result<Circle2, Failure> {
                <$t>::this_solution(self, i)
            }
            fn which_qualifier(&self, i: i32) -> Result<(Position, Position), Failure> {
                <$t>::which_qualifier(self, i)
            }
            fn same(&self, arg: i32, i: i32) -> Result<bool, Failure> {
                if arg == 1 {
                    self.is_the_same1(i)
                } else {
                    self.is_the_same2(i)
                }
            }
            fn tangency(&self, arg: i32, i: i32) -> Result<Tangency, Failure> {
                if arg == 1 {
                    self.tangency1(i)
                } else {
                    self.tangency2(i)
                }
            }
            fn center_on3(&self, i: i32) -> Option<Result<(f64, Point2), Failure>> {
                let centre: fn(&$t, i32) -> Option<Result<(f64, Point2), Failure>> = $centre;
                centre(self, i)
            }
        }
    };
}
two_tan!(Circ2d2TanRad, |_, _| None);
two_tan!(Circ2d2TanRadGeo, |_, _| None);
two_tan!(Circ2d2TanOn, |s, i| Some(s.center_on3(i)));
two_tan!(Circ2d2TanOnGeo, |s, i| Some(s.center_on3(i)));

fn two_tan_report(s: &impl TwoTan, detail: i32) {
    flag(s.is_done());
    int(s.nb_solutions());
    for i in 0..=s.nb_solutions() + 1 {
        show(s.this_solution(i), circ2d);
        if detail & 1 != 0 {
            show(s.which_qualifier(i), qualifiers2);
        }
        if detail & 2 != 0 {
            same_tangency(s.same(1, i), || s.tangency(1, i));
        }
        if detail & 4 != 0 {
            same_tangency(s.same(2, i), || s.tangency(2, i));
        }
        if detail & 8 != 0
            && let Some(c) = s.center_on3(i)
        {
            show(c, centre);
        }
    }
}

fn build_2tanrad(r: &mut Reader<'_>, kind: i32) -> Result<Circ2d2TanRad, Failure> {
    if kind == 1 {
        let q1 = r.g2gcc_q()?;
        let p = r.g2gcc_point();
        let (rad, tol) = (r.real(), r.real());
        return Circ2d2TanRad::with_point(&q1, &p, rad, tol);
    }
    if kind == 2 {
        let (p1, p2) = (r.g2gcc_point(), r.g2gcc_point());
        let (rad, tol) = (r.real(), r.real());
        return Circ2d2TanRad::with_points(&p1, &p2, rad, tol);
    }
    let (q1, q2) = (r.g2gcc_q()?, r.g2gcc_q()?);
    let (rad, tol) = (r.real(), r.real());
    let mut s = Circ2d2TanRad::new(&q1, &q2, rad, tol)?;
    if kind == 3 {
        let (a, b) = (r.qcirc(), r.qcirc());
        let (r2, t2) = (r.real(), r.real());
        attempt(|| s.results_ana(&gcc_ana::Circ2d2TanRad::circ_circ(a, b, r2, t2)?));
    }
    if kind == 4 {
        let a = r.qcirc();
        let b = r.g2gcc_q()?;
        let (r2, t2) = (r.real(), r.real());
        attempt(|| s.results_geo(&Circ2d2TanRadGeo::circ_curve(a, &b, r2, t2)?));
    }
    Ok(s)
}

/// `kind detail args`: Geom2dGcc_Circ2d2TanRad (3 and 4 call Results afterwards).
pub fn tanrad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    attempt(|| {
        let s = build_2tanrad(&mut r, kind)?;
        two_tan_report(&s, detail);
        Ok(())
    });
    end();
}

fn build_2tanrad_geo(r: &mut Reader<'_>, kind: i32) -> Result<Circ2d2TanRadGeo, Failure> {
    match kind {
        0 => {
            let a = r.qlin();
            let b = r.g2gcc_q()?;
            let (rad, tol) = (r.real(), r.real());
            Circ2d2TanRadGeo::lin_curve(a, &b, rad, tol)
        }
        1 => {
            let a = r.qcirc();
            let b = r.g2gcc_q()?;
            let (rad, tol) = (r.real(), r.real());
            Circ2d2TanRadGeo::circ_curve(a, &b, rad, tol)
        }
        2 => {
            let (a, b) = (r.g2gcc_q()?, r.g2gcc_q()?);
            let (rad, tol) = (r.real(), r.real());
            Circ2d2TanRadGeo::curve_curve(&a, &b, rad, tol)
        }
        _ => {
            let a = r.g2gcc_q()?;
            let p = r.pnt2d();
            let (rad, tol) = (r.real(), r.real());
            Circ2d2TanRadGeo::curve_pnt(&a, p, rad, tol)
        }
    }
}

/// `kind detail args`: Geom2dGcc_Circ2d2TanRadGeo.
pub fn tanrad_geo_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    attempt(|| {
        let s = build_2tanrad_geo(&mut r, kind)?;
        two_tan_report(&s, detail);
        Ok(())
    });
    end();
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

fn build_3tan(r: &mut Reader<'_>, kind: i32) -> Result<Circ2d3Tan, Failure> {
    if kind == 1 {
        let (q1, q2) = (r.g2gcc_q()?, r.g2gcc_q()?);
        let p = r.g2gcc_point();
        let (tol, p1, p2) = (r.real(), r.real(), r.real());
        return Circ2d3Tan::with_point(&q1, &q2, &p, tol, [p1, p2]);
    }
    if kind == 2 {
        let q1 = r.g2gcc_q()?;
        let (a, b) = (r.g2gcc_point(), r.g2gcc_point());
        let (tol, p1) = (r.real(), r.real());
        return Circ2d3Tan::with_points(&q1, &a, &b, tol, p1);
    }
    if kind == 3 {
        let (a, b, c) = (r.g2gcc_point(), r.g2gcc_point(), r.g2gcc_point());
        return Circ2d3Tan::with_three_points(&a, &b, &c, r.real());
    }
    let (q1, q2, q3) = (r.g2gcc_q()?, r.g2gcc_q()?, r.g2gcc_q()?);
    let (tol, p1, p2, p3) = (r.real(), r.real(), r.real(), r.real());
    let mut s = Circ2d3Tan::new(&q1, &q2, &q3, tol, [p1, p2, p3])?;
    if kind == 4 {
        let (a, b, c) = (r.qcirc(), r.qcirc(), r.qcirc());
        let t2 = r.real();
        let (r1, r2, r3) = (r.integer(), r.integer(), r.integer());
        attempt(|| {
            s.results(
                &gcc_ana::Circ2d3Tan::circ_circ_circ(a, b, c, t2)?,
                r1,
                r2,
                r3,
            )
        });
    }
    Ok(s)
}

/// `kind detail args`: Geom2dGcc_Circ2d3Tan (4 calls Results afterwards).
pub fn circ3tan_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    attempt(|| {
        let s = build_3tan(&mut r, kind)?;
        circ3_report(&s, detail);
        Ok(())
    });
    end();
}

fn build_3tan_iter(r: &mut Reader<'_>, kind: i32) -> Result<Circ2d3TanIter, Failure> {
    type I = Circ2d3TanIter;
    let three = |r: &mut Reader<'_>| [r.real(), r.real(), r.real()];
    match kind {
        0 => {
            let (a, b, c) = (r.g2gcc_q()?, r.g2gcc_q()?, r.g2gcc_q()?);
            let p = three(r);
            I::new(&a, &b, &c, p, r.real())
        }
        1 => {
            let a = r.qcirc();
            let (b, c) = (r.g2gcc_q()?, r.g2gcc_q()?);
            let p = three(r);
            I::circ_curve_curve(a, &b, &c, p, r.real())
        }
        2 => {
            let (a, b) = (r.qcirc(), r.qcirc());
            let c = r.g2gcc_q()?;
            let p = three(r);
            I::circ_circ_curve(a, b, &c, p, r.real())
        }
        3 => {
            let (a, b) = (r.qcirc(), r.qlin());
            let c = r.g2gcc_q()?;
            let p = three(r);
            I::circ_lin_curve(a, b, &c, p, r.real())
        }
        4 => {
            let a = r.qcirc();
            let b = r.g2gcc_q()?;
            let c = r.pnt2d();
            let p = [r.real(), r.real()];
            I::circ_curve_pnt(a, &b, c, p, r.real())
        }
        5 => {
            let (a, b) = (r.qlin(), r.qlin());
            let c = r.g2gcc_q()?;
            let p = three(r);
            I::lin_lin_curve(a, b, &c, p, r.real())
        }
        6 => {
            let a = r.qlin();
            let (b, c) = (r.g2gcc_q()?, r.g2gcc_q()?);
            let p = three(r);
            I::lin_curve_curve(a, &b, &c, p, r.real())
        }
        7 => {
            let a = r.qlin();
            let b = r.g2gcc_q()?;
            let c = r.pnt2d();
            let p = [r.real(), r.real()];
            I::lin_curve_pnt(a, &b, c, p, r.real())
        }
        8 => {
            let a = r.g2gcc_q()?;
            let (b, c) = (r.pnt2d(), r.pnt2d());
            let p1 = r.real();
            I::curve_pnt_pnt(&a, b, c, p1, r.real())
        }
        _ => {
            let (a, b) = (r.g2gcc_q()?, r.g2gcc_q()?);
            let c = r.pnt2d();
            let p = [r.real(), r.real()];
            I::curve_curve_pnt(&a, &b, c, p, r.real())
        }
    }
}

/// `kind args ... tol`: Geom2dGcc_Circ2d3TanIter and every consultation.
pub fn circ3tan_iter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    attempt(|| {
        let s = build_3tan_iter(&mut r, kind)?;
        flag(s.is_done());
        circ2d(s.this_solution());
        show(s.which_qualifier(), |(a, b, c)| {
            position(a);
            position(b);
            position(c);
        });
        show(s.is_the_same1(), flag);
        show(s.is_the_same2(), flag);
        show(s.is_the_same3(), flag);
        show(s.tangency1(), tangency);
        show(s.tangency2(), tangency);
        show(s.tangency3(), tangency);
        Ok(())
    });
    end();
}

fn build_2tanon(r: &mut Reader<'_>, kind: i32) -> Result<Circ2d2TanOn, Failure> {
    if kind == 1 {
        let q1 = r.g2gcc_q()?;
        let p = r.g2gcc_point();
        let on = hatch_adaptor(r)?;
        let (tol, p1, pon) = (r.real(), r.real(), r.real());
        return Circ2d2TanOn::with_point(&q1, &p, &on, tol, [p1, pon]);
    }
    if kind == 2 {
        let (a, b) = (r.g2gcc_point(), r.g2gcc_point());
        let on = hatch_adaptor(r)?;
        return Circ2d2TanOn::with_points(&a, &b, &on, r.real());
    }
    let (q1, q2) = (r.g2gcc_q()?, r.g2gcc_q()?);
    let on = hatch_adaptor(r)?;
    let (tol, p1, p2, pon) = (r.real(), r.real(), r.real(), r.real());
    let mut s = Circ2d2TanOn::new(&q1, &q2, &on, tol, [p1, p2, pon])?;
    if kind == 3 {
        let (a, b) = (r.qcirc(), r.qcirc());
        let l = r.lin2d();
        let t2 = r.real();
        attempt(|| s.results_ana(&gcc_ana::Circ2d2TanOn::circ_circ_on_line(a, b, l, t2)?));
    }
    if kind == 4 {
        let (a, b) = (r.qlin(), r.qlin());
        let on2 = hatch_adaptor(r)?;
        let t2 = r.real();
        attempt(|| s.results_geo(&Circ2d2TanOnGeo::lin_lin(a, b, &on2, t2)?));
    }
    Ok(s)
}

/// `kind detail args`: Geom2dGcc_Circ2d2TanOn (3 and 4 call Results afterwards).
pub fn tanon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    attempt(|| {
        let s = build_2tanon(&mut r, kind)?;
        two_tan_report(&s, detail);
        Ok(())
    });
    end();
}

fn build_2tanon_geo(r: &mut Reader<'_>, kind: i32) -> Result<Circ2d2TanOnGeo, Failure> {
    type G = Circ2d2TanOnGeo;
    match kind {
        0 => {
            let (a, b) = (r.qcirc(), r.qcirc());
            let on = hatch_adaptor(r)?;
            G::circ_circ(a, b, &on, r.real())
        }
        1 => {
            let (a, b) = (r.qcirc(), r.qlin());
            let on = hatch_adaptor(r)?;
            G::circ_lin(a, b, &on, r.real())
        }
        2 => {
            let (a, b) = (r.qcirc(), r.pnt2d());
            let on = hatch_adaptor(r)?;
            G::circ_pnt(a, b, &on, r.real())
        }
        3 => {
            let (a, b) = (r.qlin(), r.qlin());
            let on = hatch_adaptor(r)?;
            G::lin_lin(a, b, &on, r.real())
        }
        4 => {
            let (a, b) = (r.qlin(), r.pnt2d());
            let on = hatch_adaptor(r)?;
            G::lin_pnt(a, b, &on, r.real())
        }
        _ => {
            let (a, b) = (r.pnt2d(), r.pnt2d());
            let on = hatch_adaptor(r)?;
            G::pnt_pnt(a, b, &on, r.real())
        }
    }
}

/// `kind detail args oncurve tol`: Geom2dGcc_Circ2d2TanOnGeo.
pub fn tanon_geo_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    attempt(|| {
        let s = build_2tanon_geo(&mut r, kind)?;
        two_tan_report(&s, detail);
        Ok(())
    });
    end();
}

// The locus of a Geom2dGcc_Circ2d2TanOnIter request.
enum Locus {
    Line(rustycad_tkmath::gp::Line2),
    Circle(Circle2),
    Curve(rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve),
}

fn build_on_iter(r: &mut Reader<'_>, first: i32, on: &Locus) -> Result<Circ2d2TanOnIter, Failure> {
    type I = Circ2d2TanOnIter;
    if first == 3 {
        let a = r.g2gcc_q()?;
        let p = r.pnt2d();
        let (p1, p2, t) = (r.real(), r.real(), r.real());
        return match on {
            Locus::Line(l) => I::curve_pnt_on_line(&a, p, *l, [p1, p2], t),
            Locus::Circle(c) => I::curve_pnt_on_circle(&a, p, *c, [p1, p2], t),
            Locus::Curve(c) => I::curve_pnt_on_curve(&a, p, c, [p1, p2], t),
        };
    }
    let tail = |r: &mut Reader<'_>| ([r.real(), r.real(), r.real()], r.real());
    if first == 0 {
        let a = r.qcirc();
        let b = r.g2gcc_q()?;
        let (p, t) = tail(r);
        return match on {
            Locus::Line(l) => I::circ_curve_on_line(a, &b, *l, p, t),
            Locus::Circle(c) => I::circ_curve_on_circle(a, &b, *c, p, t),
            Locus::Curve(c) => I::circ_curve_on_curve(a, &b, c, p, t),
        };
    }
    if first == 1 {
        let a = r.qlin();
        let b = r.g2gcc_q()?;
        let (p, t) = tail(r);
        return match on {
            Locus::Line(l) => I::lin_curve_on_line(a, &b, *l, p, t),
            Locus::Circle(c) => I::lin_curve_on_circle(a, &b, *c, p, t),
            Locus::Curve(c) => I::lin_curve_on_curve(a, &b, c, p, t),
        };
    }
    let (a, b) = (r.g2gcc_q()?, r.g2gcc_q()?);
    let (p, t) = tail(r);
    match on {
        Locus::Line(l) => I::curve_curve_on_line(&a, &b, *l, p, t),
        Locus::Circle(c) => I::curve_curve_on_circle(&a, &b, *c, p, t),
        Locus::Curve(c) => I::curve_curve_on_curve(&a, &b, c, p, t),
    }
}

/// `kind locus args... tol`: Geom2dGcc_Circ2d2TanOnIter and every consultation.
pub fn tanon_iter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (first, locus) = (kind % 4, kind / 4);
    attempt(|| {
        let on = match locus {
            0 => Locus::Line(r.lin2d()),
            1 => Locus::Circle(r.circ2d()),
            _ => Locus::Curve(hatch_adaptor(&mut r)?),
        };
        let s = build_on_iter(&mut r, first, &on)?;
        flag(s.is_done());
        circ2d(s.this_solution());
        show(s.which_qualifier(), qualifiers2);
        show(s.is_the_same1(), flag);
        show(s.is_the_same2(), flag);
        show(s.tangency1(), tangency);
        show(s.tangency2(), tangency);
        show(s.center_on3(), centre);
        Ok(())
    });
    end();
}
