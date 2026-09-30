//! Geom2dGcc line solver probe operations, the Rust twin of the line part of
//! tools/oracle/tkgeomalgo_probe_geom2dgcc.cxx.
use super::common::{Reader, attempt, end, flag, int, show};
use super::gcc_ana::{qualifiers2, tangency};
use super::gcc_ent::{lin2d, position};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::geom2d_gcc::{Lin2d2Tan, Lin2d2TanIter, Lin2dTanObl, Lin2dTanOblIter};

// The solutions from index 0 to NbSolutions + 1 with the consultations of `detail`.
fn lin2tan_report(s: &Lin2d2Tan, detail: i32) {
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
}

/// `kind detail args`: Geom2dGcc_Lin2d2Tan.
pub fn lin2tan_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    attempt(|| {
        let q1 = r.g2gcc_q()?;
        let s = if kind == 0 || kind == 2 {
            let q2 = r.g2gcc_q()?;
            let tol = r.real();
            if kind == 0 {
                Lin2d2Tan::new(&q1, &q2, tol)?
            } else {
                let (p1, p2) = (r.real(), r.real());
                Lin2d2Tan::with_parameters(&q1, &q2, tol, p1, p2)?
            }
        } else {
            let p = r.pnt2d();
            let tol = r.real();
            if kind == 1 {
                Lin2d2Tan::through_point(&q1, p, tol)?
            } else {
                Lin2d2Tan::through_point_with_parameter(&q1, p, tol, r.real())?
            }
        };
        lin2tan_report(&s, detail);
        Ok(())
    });
    end();
}

fn build_lin2tan_iter(r: &mut Reader<'_>, kind: i32) -> Result<Lin2d2TanIter, Failure> {
    if kind == 0 {
        let q1 = r.g2gcc_q()?;
        let p = r.pnt2d();
        let (p1, tol) = (r.real(), r.real());
        return Lin2d2TanIter::through_point(&q1, p, p1, tol);
    }
    if kind == 1 {
        let q1 = r.qcirc();
        let q2 = r.g2gcc_q()?;
        let (p2, tol) = (r.real(), r.real());
        return Lin2d2TanIter::circle_curve(q1, &q2, p2, tol);
    }
    let (q1, q2) = (r.g2gcc_q()?, r.g2gcc_q()?);
    let (p1, p2, tol) = (r.real(), r.real(), r.real());
    Lin2d2TanIter::new(&q1, &q2, p1, p2, tol)
}

/// `kind args`: Geom2dGcc_Lin2d2TanIter and every consultation.
pub fn lin2tan_iter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    attempt(|| {
        let s = build_lin2tan_iter(&mut r, kind)?;
        flag(s.is_done());
        show(s.this_solution(), lin2d);
        show(s.which_qualifier(), qualifiers2);
        show(s.tangency1(), tangency);
        show(s.tangency2(), tangency);
        Ok(())
    });
    end();
}

fn lintanobl_report(s: &Lin2dTanObl, detail: i32) {
    flag(s.is_done());
    int(s.nb_solutions());
    for i in 0..=s.nb_solutions() + 1 {
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
}

/// `kind detail q1 line tolang [param1] angle`: Geom2dGcc_Lin2dTanObl.
pub fn lintanobl_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, detail) = (r.integer(), r.integer());
    attempt(|| {
        let q1 = r.g2gcc_q()?;
        let l = r.lin2d();
        let tol = r.real();
        let s = if kind == 0 {
            Lin2dTanObl::new(&q1, l, tol, r.real())?
        } else {
            let p1 = r.real();
            Lin2dTanObl::with_parameter(&q1, l, tol, p1, r.real())?
        };
        lintanobl_report(&s, detail);
        Ok(())
    });
    end();
}

/// `q1 line param1 tolang angle`: Geom2dGcc_Lin2dTanOblIter and every consultation.
pub fn lintanobl_iter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let q1 = r.g2gcc_q()?;
        let l = r.lin2d();
        let (p1, tol, a) = (r.real(), r.real(), r.real());
        let s = Lin2dTanOblIter::new(&q1, l, p1, tol, a)?;
        flag(s.is_done());
        flag(s.is_parallel2());
        show(s.this_solution(), lin2d);
        show(s.which_qualifier(), position);
        show(s.tangency1(), tangency);
        show(s.intersection2(), tangency);
        Ok(())
    });
    end();
}
