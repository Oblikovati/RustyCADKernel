//! GccAna probe operations on lines and bisectors (Lin2d2Tan, Lin2dTanObl,
//! Lin2dBisec and the bisector classes), the Rust twin of the corresponding cases of
//! tools/oracle/tkgeomalgo_probe_gccana.cxx.
use super::common::{Reader, end, flag, int, show};
use super::gcc_ana::{qualifiers2, solve, tangency};
use super::gcc_ent::{lin2d, position};
use super::gcc_int::accessors;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::gcc_ana::{
    Circ2dBisec, CircLin2dBisec, CircPnt2dBisec, Lin2d2Tan, Lin2dBisec, Lin2dTanObl, LinPnt2dBisec,
    Pnt2dBisec,
};
use rustycad_tkgeomalgo::gcc_int::Bisec;

/// kind 0 `p1 p2 tol`, 1 `qcirc point tol`, 2 `qcirc qcirc tol`.
pub fn lin2tan_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    let built: Result<Lin2d2Tan, Failure> = match kind {
        0 => {
            let (a, b) = (r.pnt2d(), r.pnt2d());
            Lin2d2Tan::through_points(a, b, r.real())
        }
        1 => {
            let (a, b) = (r.qcirc(), r.pnt2d());
            Lin2d2Tan::tangent_through_point(a, b, r.real())
        }
        _ => {
            let (a, b) = (r.qcirc(), r.qcirc());
            Lin2d2Tan::tangent_to_circles(a, b, r.real())
        }
    };
    solve(built, |s| {
        flag(s.is_done());
        int(s.nb_solutions());
        for i in 0..=s.nb_solutions() + 1 {
            show(s.this_solution(i), lin2d);
            if detail & 1 != 0 {
                show(s.which_qualifier(i), qualifiers2);
            }
            if detail & 2 != 0 {
                show(s.tangency1(i), tangency);
            }
            if detail & 4 != 0 {
                show(s.tangency2(i), tangency);
            }
        }
    });
    end();
}

/// kind 0 `point line angle`, 1 `qcirc line angle`.
pub fn lintanobl_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    let built: Result<Lin2dTanObl, Failure> = if kind == 0 {
        let (p, l) = (r.pnt2d(), r.lin2d());
        Lin2dTanObl::through_point(p, l, r.real())
    } else {
        let (a, l) = (r.qcirc(), r.lin2d());
        Lin2dTanObl::tangent_to_circle(a, l, r.real())
    };
    solve(built, |s| {
        flag(s.is_done());
        let mut n = -1;
        show(s.nb_solutions(), |count| {
            n = count;
            int(count);
        });
        for i in 0..=n + 1 {
            show(s.this_solution(i), lin2d);
            if detail & 1 != 0 {
                show(s.which_qualifier(i), position);
            }
            if detail & 2 != 0 {
                show(s.tangency1(i), tangency);
            }
            if detail & 4 != 0 {
                show(s.intersection2(i), tangency);
            }
        }
    });
    end();
}

/// `line line`: the bisectors and their intersections with both lines.
pub fn linbisec_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (r.lin2d(), r.lin2d());
    solve(Lin2dBisec::new(a, b), |s| {
        flag(s.is_done());
        let mut n = -1;
        show(s.nb_solutions(), |count| {
            n = count;
            int(count);
        });
        for i in 0..=n + 1 {
            show(s.this_solution(i), lin2d);
            show(s.intersection1(i), tangency);
            show(s.intersection2(i), tangency);
        }
    });
    end();
}

fn bisec_report(b: Handle<dyn Bisec>) {
    if b.is_null() {
        print!("null ");
        return;
    }
    accessors(&b);
}

fn bisec_solutions(
    done: bool,
    count: i32,
    solution: impl Fn(i32) -> Result<Handle<dyn Bisec>, Failure>,
) {
    flag(done);
    int(count);
    for i in 0..=count + 1 {
        show(solution(i), bisec_report);
    }
}

/// kind 0 `circ circ`, 1 `circ line`, 2 `circ point`, 3 `circ point tol`, 4 `line
/// point`, 5 `point point`.
pub fn bisec_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => {
            let (a, b) = (r.circ2d(), r.circ2d());
            let s = Circ2dBisec::new(a, b);
            flag(s.is_done());
            let mut n = -1;
            show(s.nb_solutions(), |count| {
                n = count;
                int(count);
            });
            for i in 0..=n + 1 {
                show(s.this_solution(i), bisec_report);
            }
        }
        1 => {
            let (a, b) = (r.circ2d(), r.lin2d());
            let s = CircLin2dBisec::new(a, b);
            bisec_solutions(s.is_done(), s.nb_solutions(), |i| s.this_solution(i));
        }
        kind @ (2 | 3) => {
            let (a, b) = (r.circ2d(), r.pnt2d());
            let s = if kind == 2 {
                CircPnt2dBisec::new(a, b)
            } else {
                CircPnt2dBisec::with_tolerance(a, b, r.real())
            };
            bisec_solutions(s.is_done(), s.nb_solutions(), |i| s.this_solution(i));
        }
        4 => {
            let a = r.lin2d();
            solve(LinPnt2dBisec::new(a, r.pnt2d()), |s| {
                flag(s.is_done());
                show(s.this_solution(), bisec_report);
            });
        }
        _ => {
            let (a, b) = (r.pnt2d(), r.pnt2d());
            solve(Pnt2dBisec::new(a, b), |s| {
                flag(s.is_done());
                flag(s.has_solution());
                show(s.this_solution(), lin2d);
            });
        }
    }
    end();
}
