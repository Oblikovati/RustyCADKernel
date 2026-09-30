//! The Rust twin of tools/oracle/tkgeombase_probe_femtool_curve.cxx.
use super::super::common::{self as p, Reader, attempt, end};
use super::{read_array2, read_vector, shape, vector};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1, HArray2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeombase::fem_tool::{Assembly, AssemblyTable, Curve};
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib::HermitJacobi;

fn array1(a: &Array1<'_, f64>) {
    p::int(a.lower());
    p::int(a.upper());
    a.iter().for_each(|value| p::real(*value));
}

fn filled1(lower: i32, length: i32) -> Array1<'static, f64> {
    let mut a = Array1::with_bounds(lower, lower + length - 1).unwrap();
    a.init(-7.);
    a
}

fn curve_command(r: &mut Reader<'_>, c: &Curve) -> Result<(), Failure> {
    match r.integer() {
        1 => {
            let el = r.integer();
            let a = read_array2(r);
            c.set_element(el, &a)?;
        }
        2 => {
            let el = r.integer();
            let (rl, cl, rows, cols) = (r.integer(), r.integer(), r.integer(), r.integer());
            let mut a = Array2::with_bounds(rl, rl + rows - 1, cl, cl + cols - 1).unwrap();
            a.init(-7.);
            c.get_element(el, &mut a)?;
            a.iter().for_each(|value| p::real(*value));
        }
        3 => {
            let (lower, length) = (r.integer(), r.integer());
            let mut a = filled1(lower, length);
            c.get_polynom(&mut a)?;
            array1(&a);
        }
        4 => {
            let (order, u, lower, length) = (r.integer(), r.real(), r.integer(), r.integer());
            let mut a = filled1(lower, length);
            match order {
                0 => c.d0(u, &mut a)?,
                1 => c.d1(u, &mut a)?,
                _ => c.d2(u, &mut a)?,
            }
            array1(&a);
        }
        5 => {
            let (a, b) = (r.real(), r.real());
            p::real(c.length(a, b)?);
        }
        6 => {
            let el = r.integer();
            c.set_degree(el, r.integer())?;
        }
        7 => {
            let (el, tol) = (r.integer(), r.real());
            let (degree, error) = c.reduce_degree(el, tol)?;
            p::int(degree);
            p::real(error);
        }
        8 => {
            let (i, value) = (r.integer(), r.real());
            c.knots()[i] = value;
        }
        _ => p::int(c.degree(r.integer())?),
    }
    Ok(())
}

pub fn curve_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (dim, nb, wd, sh) = (r.integer(), r.integer(), r.integer(), r.integer());
    let knots: Vec<f64> = (0..=nb).map(|_| r.real()).collect();
    let count = r.integer();
    let c = match HermitJacobi::new(wd, shape(sh))
        .and_then(|base| Curve::new(dim, nb, &base, 1.0e-7))
    {
        Ok(c) => c,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    knots.iter().zip(1..).for_each(|(k, i)| c.knots()[i] = *k);
    let mut zero = Array2::with_bounds(0, wd, 1, dim).unwrap();
    zero.init(0.);
    (1..=nb).for_each(|i| c.set_element(i, &zero).expect("probe zero element"));
    for _ in 0..count {
        attempt(|| curve_command(&mut r, &c));
    }
    end();
}

fn assembly_command(r: &mut Reader<'_>, a: &mut Assembly) -> Result<(), Failure> {
    match r.integer() {
        1 => {
            let (el, d1, d2) = (r.integer(), r.integer(), r.integer());
            let v = read_array2(r);
            let mut m =
                Matrix::with_bounds(v.lower_row(), v.upper_row(), v.lower_col(), v.upper_col())
                    .unwrap();
            for i in v.lower_row()..=v.upper_row() {
                (v.lower_col()..=v.upper_col()).for_each(|j| m[(i, j)] = v[(i, j)]);
            }
            a.add_matrix(el, d1, d2, &m)?;
        }
        2 => {
            let (el, d) = (r.integer(), r.integer());
            let v = read_vector(r);
            a.add_vector(el, d, &v)?;
        }
        3 => {
            let (idx, el, d) = (r.integer(), r.integer(), r.integer());
            let v = read_vector(r);
            a.add_constraint(idx, el, d, &v, r.real())?;
        }
        4 => p::flag(a.solve()?),
        5 => {
            let lower = r.integer();
            let mut x = Vector::filled(lower, lower + a.nb_glob_var() - 1, -7.)?;
            a.solution(&mut x)?;
            vector(&x);
        }
        6 => a.nullify_matrix(),
        7 => a.nullify_vector(),
        8 => a.reset_constraint(),
        9 => a.nullify_constraint(),
        _ => p::int(a.nb_glob_var()),
    }
    Ok(())
}

pub fn assembly_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (dr, dc) = (r.integer(), r.integer());
    let mut dep = Array2::with_bounds(1, dr, 1, dc).unwrap();
    for i in 1..=dr {
        (1..=dc).for_each(|j| dep[(i, j)] = r.integer());
    }
    let (tr, tc) = (r.integer(), r.integer());
    let table: AssemblyTable = Handle::new(HArray2::with_bounds(1, tr, 1, tc).unwrap());
    for i in 1..=tr {
        for j in 1..=tc {
            let (lower, length) = (r.integer(), r.integer());
            let mut cell = Array1::with_bounds(lower, lower + length - 1).unwrap();
            (0..length).for_each(|k| cell[lower + k] = r.integer());
            table.change_array2()[(i, j)] = Handle::new(HArray1::from_owned(cell));
        }
    }
    let count = r.integer();
    attempt(|| {
        let mut a = Assembly::new(&dep, &table)?;
        for _ in 0..count {
            attempt(|| assembly_command(&mut r, &mut a));
        }
        Ok(())
    });
    end();
}
