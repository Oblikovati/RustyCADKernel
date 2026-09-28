//! Counterparts of the basis, banded-system and interpolation cases of
//! tkmath_bspl_eval_probe.cxx.
use super::curves::Curve;
use super::eval::{dimension, flat_of};
use super::{In, fail, o, oa, ob, oi, opa};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::bsplclib::{self, CurvePoint};
use rustycad_tkmath::math::Matrix;

// basis: request order flat param periodic rows cols
pub fn basis_case(v: &[f64]) {
    let mut f = In::new(v);
    let (req, order, k, u, per, rows, cols) = (f.i(), f.i(), f.reals(), f.r(), f.b(), f.i(), f.i());
    let mut basis: Matrix<'static> = Matrix::filled(1, rows, 1, cols, 91.0).unwrap();
    let mut first: i32 = -7;
    match bsplclib::eval_bspline_basis(req, order, &k, u, &mut first, &mut basis, per) {
        Ok(code) => {
            oi(code);
            oi(first);
            matrix_out(&basis);
        }
        Err(failure) => fail(&failure),
    }
    println!();
}

fn matrix_out(m: &Matrix<'_>) {
    for r in m.lower_row()..=m.upper_row() {
        for q in m.lower_col()..=m.upper_col() {
            o(m[(r, q)]);
        }
    }
}

// interpolate: curve(knots = flat, poles = values) params contacts
pub fn interpolate_case(v: &[f64]) {
    let mut f = In::new(v);
    let c = Curve::read(&mut f);
    let (t, contacts) = (f.reals(), f.ints());
    let dim: i32 = dimension(&c);
    let n: i32 = c.k.len() as i32 - c.deg - 1;
    let mut s: Array1<'static, f64> = Array1::with_bounds(1, n.max(0)).unwrap();
    match bsplclib::build_schoenberg_points(c.deg, &c.k, &mut s) {
        Ok(()) => oa(&s),
        Err(failure) => fail(&failure),
    }
    let mut m: Matrix<'static> =
        Matrix::filled(t.lower(), t.upper(), 1, 2 * c.deg + 1, 91.0).unwrap();
    let (code, up, lo) = bsplclib::build_bsp_matrix(&t, &contacts, &c.k, c.deg, &mut m).unwrap();
    oi(code);
    oi(up);
    oi(lo);
    matrix_out(&m);
    let (code, pivot) = bsplclib::factor_banded_matrix(&mut m, up, lo).unwrap();
    oi(code);
    oi(pivot);
    matrix_out(&m);
    let values: Vec<f64> = flat_of(&c);
    let weights: Vec<f64> = (0..t.len())
        .map(|k| {
            if c.w.is_empty() {
                1.0
            } else {
                c.w.as_slice()[k]
            }
        })
        .collect();
    solves(&m, [up, lo], dim, &values, &weights);
    interpolations(&c, &t, &contacts, dim, &values, &weights);
    if c.kind > 1 {
        let mut w: Array1<'static, f64> = Array1::with_bounds(1, t.len() as i32).unwrap();
        w.as_mut_slice().copy_from_slice(&weights);
        match c.kind {
            2 => points(&c, &c.p2, &t, &contacts, &w, (&m, up, lo)),
            _ => points(&c, &c.p3, &t, &contacts, &w, (&m, up, lo)),
        }
    }
    println!();
}

fn solves(m: &Matrix<'_>, [up, lo]: [i32; 2], dim: i32, values: &[f64], weights: &[f64]) {
    let mut v: Vec<f64> = values.to_vec();
    oi(bsplclib::solve_banded_system(m, up, lo, dim, &mut v).unwrap());
    ob(&v);
    for homogeneous in [false, true] {
        let (mut v, mut w) = (values.to_vec(), weights.to_vec());
        oi(
            bsplclib::solve_banded_system_rational(m, up, lo, homogeneous, dim, &mut v, &mut w)
                .unwrap(),
        );
        ob(&v);
        ob(&w);
    }
}

fn interpolations(
    c: &Curve,
    t: &Array1<'_, f64>,
    contacts: &Array1<'_, i32>,
    dim: i32,
    values: &[f64],
    weights: &[f64],
) {
    let mut problem: i32 = -7;
    let mut v: Vec<f64> = values.to_vec();
    match bsplclib::interpolate_flat(c.deg, &c.k, t, contacts, dim, &mut v, &mut problem) {
        Ok(()) => {
            oi(problem);
            ob(&v);
        }
        Err(failure) => {
            oi(problem);
            fail(&failure);
        }
    }
    problem = -7;
    let (mut v, mut w) = (values.to_vec(), weights.to_vec());
    match bsplclib::interpolate_flat_rational(
        c.deg,
        &c.k,
        t,
        contacts,
        dim,
        &mut v,
        &mut w,
        &mut problem,
    ) {
        Ok(()) => {
            oi(problem);
            ob(&v);
            ob(&w);
        }
        Err(failure) => {
            oi(problem);
            fail(&failure);
        }
    }
}

fn points<P: CurvePoint>(
    c: &Curve,
    poles: &Array1<'static, P>,
    t: &Array1<'_, f64>,
    contacts: &Array1<'_, i32>,
    weights: &Array1<'static, f64>,
    (m, up, lo): (&Matrix<'_>, i32, i32),
) {
    let mut problem: i32 = -7;
    let mut p = poles.clone();
    let result =
        bsplclib::interpolate(c.deg, &c.k, t, contacts, &mut p, &mut problem).and_then(|()| {
            oi(problem);
            opa(&p);
            p = poles.clone();
            let mut w = weights.clone();
            bsplclib::interpolate_rational(c.deg, &c.k, t, contacts, &mut p, &mut w, &mut problem)?;
            opa(&p);
            oa(&w);
            Ok(())
        });
    if let Err(failure) = result {
        oi(problem);
        fail(&failure);
    }
    let mut p = poles.clone();
    oi(bsplclib::solve_banded_system_points(m, up, lo, &mut p).unwrap());
    opa(&p);
}
