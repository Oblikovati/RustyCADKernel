//! The Rust twin of tools/oracle/tkgeombase_probe_femtool.cxx.
mod curve;

pub use curve::{assembly_case, curve_case};

use super::common::{self as p, Reader, attempt, end};
use rustycad_tkernel::ncollection::{Array2, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::fem_tool::{
    ElementaryCriterion, ElementsOfRefMatrix, LinearFlexion, LinearJerk, LinearTension,
    ProfileMatrix, SparseMatrix,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib::HermitJacobi;

pub fn shape(code: i32) -> Shape {
    [Shape::C0, Shape::C1, Shape::C2, Shape::G1, Shape::C3][code as usize]
}

pub fn vector(v: &Vector<'_>) {
    p::int(v.lower());
    p::int(v.upper());
    (v.lower()..=v.upper()).for_each(|i| p::real(v[i]));
}

pub fn matrix(m: &Matrix<'_>) {
    p::int(m.lower_row());
    p::int(m.upper_row());
    p::int(m.lower_col());
    p::int(m.upper_col());
    for i in m.lower_row()..=m.upper_row() {
        (m.lower_col()..=m.upper_col()).for_each(|j| p::real(m[(i, j)]));
    }
}

pub fn read_vector(r: &mut Reader<'_>) -> Vector<'static> {
    let (lower, length) = (r.integer(), r.integer());
    let mut v = Vector::with_bounds(lower, lower + length - 1).unwrap();
    (0..length).for_each(|i| v[lower + i] = r.real());
    v
}

pub fn read_array2(r: &mut Reader<'_>) -> Array2<'static, f64> {
    let (rl, cl, rows, cols) = (r.integer(), r.integer(), r.integer(), r.integer());
    let mut a = Array2::with_bounds(rl, rl + rows - 1, cl, cl + cols - 1).unwrap();
    for i in 0..rows {
        (0..cols).for_each(|j| a[(rl + i, cl + j)] = r.real());
    }
    a
}

fn captured(text: String) {
    print!("{} ", text.replace('\n', "|"));
}

pub fn elements_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (wd, sh, der, u, flen) = (r.integer(), r.integer(), r.integer(), r.real(), r.integer());
    attempt(|| {
        let base = HermitJacobi::new(wd, shape(sh))?;
        let e = ElementsOfRefMatrix::new(&base, der)?;
        p::int(e.nb_variables());
        p::int(e.nb_equations());
        let x = Vector::filled(1, 1, u)?;
        let mut f = Vector::filled(1, flen, 0.)?;
        p::flag(e.evaluate(&x, &mut f)?);
        vector(&f);
        Ok(())
    });
    end();
}

fn criterion_command(r: &mut Reader<'_>, c: &dyn ElementaryCriterion) -> Result<(), Failure> {
    match r.integer() {
        1 => {
            let table = c.dependence_table()?;
            let t = table.array2();
            [t.lower_row(), t.upper_row(), t.lower_col(), t.upper_col()]
                .into_iter()
                .for_each(p::int);
            for i in t.lower_row()..=t.upper_row() {
                (t.lower_col()..=t.upper_col()).for_each(|j| p::int(t[(i, j)]));
            }
        }
        2 => p::real(c.value()?),
        3 => {
            let (d1, d2) = (r.integer(), r.integer());
            let (rl, ru, cl, cu) = (r.integer(), r.integer(), r.integer(), r.integer());
            let mut h = Matrix::filled(rl, ru, cl, cu, -7.).unwrap();
            c.hessian(d1, d2, &mut h)?;
            matrix(&h);
        }
        4 => {
            let (d, gl, gu) = (r.integer(), r.integer(), r.integer());
            let mut g = Vector::filled(gl, gu, -7.)?;
            c.gradient(d, &mut g)?;
            vector(&g);
        }
        _ => {
            let (a, b) = (r.real(), r.real());
            c.set(a, b);
        }
    }
    Ok(())
}

fn new_criterion(kind: i32, wd: i32, sh: i32) -> Result<Handle<dyn ElementaryCriterion>, Failure> {
    Ok(match kind {
        0 => Handle::new(LinearFlexion::new(wd, shape(sh))?).cast(),
        1 => Handle::new(LinearJerk::new(wd, shape(sh))?).cast(),
        _ => Handle::new(LinearTension::new(wd, shape(sh))?).cast(),
    })
}

pub fn criterion_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, wd, sh, with_coeff) = (r.integer(), r.integer(), r.integer(), r.flag());
    let coeff = with_coeff.then(|| Handle::new(HArray2::from_owned(read_array2(&mut r))));
    let (first, last, count) = (r.real(), r.real(), r.integer());
    let c = match new_criterion(kind, wd, sh) {
        Ok(c) => c,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    if let Some(coeff) = &coeff {
        c.set_coeff(coeff);
    }
    c.set(first, last);
    for _ in 0..count {
        attempt(|| criterion_command(&mut r, &*c));
    }
    end();
}

fn profile_command(r: &mut Reader<'_>, m: &ProfileMatrix) -> Result<(), Failure> {
    match r.integer() {
        1 => m.init(r.real()),
        2 => {
            let (i, j) = (r.integer(), r.integer());
            let value = r.real();
            *m.change_value(i, j)? += value;
            p::real(*m.change_value(i, j)?);
        }
        3 => p::flag(m.decompose()?),
        code @ (4 | 5) => {
            let b = read_vector(r);
            let mut x = Vector::filled(1, m.row_number(), 0.)?;
            if code == 4 {
                m.solve(&b, &mut x)?;
            } else {
                m.multiplied(&b, &mut x)?;
            }
            vector(&x);
        }
        6 => {
            let i = r.integer();
            p::flag(m.is_in_profile(i, r.integer())?);
        }
        7 => captured(m.out_text(true)?),
        8 => captured(m.out_text(false)?),
        9 => p::flag(m.prepare()?),
        10 => {
            let n = m.row_number();
            let b = Vector::filled(1, n, 1.)?;
            let (mut x, mut res) = (Vector::filled(1, n, 0.)?, Vector::filled(1, n, 0.)?);
            m.solve_iterative(&b, &b, &mut x, &mut res, 1.0e-8, 50)?;
        }
        _ => {
            p::int(m.row_number());
            p::int(m.col_number());
        }
    }
    Ok(())
}

pub fn profile_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let mut first = rustycad_tkernel::ncollection::Array1::with_bounds(1, n).unwrap();
    (1..=n).for_each(|i| first[i] = r.integer());
    let count = r.integer();
    let m = ProfileMatrix::new(&first).expect("probe profile");
    m.init(0.);
    for _ in 0..count {
        attempt(|| profile_command(&mut r, &m));
    }
    end();
}
