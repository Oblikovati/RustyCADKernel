//! PLib_JacobiPolynomial and PLib_HermitJacobi counterparts.
use super::{Fields, array, bounded, fail, filled, out};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::plib::{HermitJacobi, JacobiPolynomial};

/// Maps a native GeomAbs_Shape integer (0..6) to the Rust enumeration.
pub fn shape(value: i32) -> Shape {
    [
        Shape::C0,
        Shape::G1,
        Shape::C1,
        Shape::G2,
        Shape::C2,
        Shape::C3,
        Shape::CN,
    ][value as usize]
}

/// The members PLib_JacobiPolynomial and PLib_HermitJacobi share.
trait Basis {
    fn max_error(&self, dim: i32, c: &[f64], new_degree: i32) -> Result<f64, Failure>;
    fn reduce_degree(&self, dim: i32, max: i32, tol: f64, c: &[f64])
    -> Result<(i32, f64), Failure>;
    fn average_error(&self, dim: i32, c: &[f64], new_degree: i32) -> Result<f64, Failure>;
    fn to_coefficients(
        &self,
        dim: i32,
        degree: i32,
        j: &Array1<'_, f64>,
        r: &mut Array1<'_, f64>,
    ) -> Result<(), Failure>;
    fn d0(&self, u: f64, v: &mut Array1<'_, f64>) -> Result<(), Failure>;
    fn d1(&self, u: f64, v: &mut Array1<'_, f64>, d1: &mut Array1<'_, f64>) -> Result<(), Failure>;
    fn d2(
        &self,
        u: f64,
        v: &mut Array1<'_, f64>,
        d1: &mut Array1<'_, f64>,
        d2: &mut Array1<'_, f64>,
    ) -> Result<(), Failure>;
    #[allow(clippy::too_many_arguments)]
    fn d3(
        &self,
        u: f64,
        v: &mut Array1<'_, f64>,
        d1: &mut Array1<'_, f64>,
        d2: &mut Array1<'_, f64>,
        d3: &mut Array1<'_, f64>,
    ) -> Result<(), Failure>;
}

macro_rules! basis_impl {
    ($type:ty) => {
        impl Basis for $type {
            fn max_error(&self, dim: i32, c: &[f64], new_degree: i32) -> Result<f64, Failure> {
                <$type>::max_error(self, dim, c, new_degree)
            }
            fn reduce_degree(
                &self,
                dim: i32,
                max: i32,
                tol: f64,
                c: &[f64],
            ) -> Result<(i32, f64), Failure> {
                <$type>::reduce_degree(self, dim, max, tol, c)
            }
            fn average_error(&self, dim: i32, c: &[f64], new_degree: i32) -> Result<f64, Failure> {
                <$type>::average_error(self, dim, c, new_degree)
            }
            fn to_coefficients(
                &self,
                dim: i32,
                degree: i32,
                j: &Array1<'_, f64>,
                r: &mut Array1<'_, f64>,
            ) -> Result<(), Failure> {
                <$type>::to_coefficients(self, dim, degree, j, r)
            }
            fn d0(&self, u: f64, v: &mut Array1<'_, f64>) -> Result<(), Failure> {
                <$type>::d0(self, u, v)
            }
            fn d1(
                &self,
                u: f64,
                v: &mut Array1<'_, f64>,
                d1: &mut Array1<'_, f64>,
            ) -> Result<(), Failure> {
                <$type>::d1(self, u, v, d1)
            }
            fn d2(
                &self,
                u: f64,
                v: &mut Array1<'_, f64>,
                d1: &mut Array1<'_, f64>,
                d2: &mut Array1<'_, f64>,
            ) -> Result<(), Failure> {
                <$type>::d2(self, u, v, d1, d2)
            }
            fn d3(
                &self,
                u: f64,
                v: &mut Array1<'_, f64>,
                d1: &mut Array1<'_, f64>,
                d2: &mut Array1<'_, f64>,
                d3: &mut Array1<'_, f64>,
            ) -> Result<(), Failure> {
                <$type>::d3(self, u, v, d1, d2, d3)
            }
        }
    };
}
basis_impl!(JacobiPolynomial);
basis_impl!(HermitJacobi);

struct Request {
    work: i32,
    order: i32,
    nb: i32,
    u: f64,
    dim: i32,
    new_degree: i32,
    max_degree: i32,
    tol: f64,
    to_degree: i32,
    coefficients: Vec<f64>,
}

pub fn jacobi_case(f: &mut Fields<'_>, hermit: bool) {
    let (work, order, nb, u, dim) = (f.int(), f.int(), f.int(), f.real(), f.int());
    let (new_degree, max_degree, tol, to_degree) = (f.int(), f.int(), f.real(), f.int());
    let coefficients: Vec<f64> = f.reals((work.max(0) + 1) * dim);
    let r = Request {
        work,
        order,
        nb,
        u,
        dim,
        new_degree,
        max_degree,
        tol,
        to_degree,
        coefficients,
    };
    let result: Result<(), Failure> = if hermit {
        hermit_run(&r)
    } else {
        jacobi_run(&r)
    };
    if let Err(failure) = result {
        fail(&failure);
    }
    println!();
}

fn hermit_run(r: &Request) -> Result<(), Failure> {
    let b = HermitJacobi::new(r.work, shape(r.order))?;
    print!("{} {} ", b.work_degree(), b.niv_constr());
    errors(&b, r);
    convert(&b, r);
    basis(&b, r.u, r.work)
}

fn jacobi_run(r: &Request) -> Result<(), Failure> {
    let b = JacobiPolynomial::new(r.work, shape(r.order))?;
    let degree: i32 = r.work - 2 * (b.niv_constr() + 1);
    print!("{} {} ", b.work_degree(), b.niv_constr());
    let mut pts: Array1<'static, f64> = filled(0, (r.nb / 2).max(0), 91.0);
    match b.points(r.nb, &mut pts) {
        Ok(()) => array(&pts),
        Err(failure) => fail(&failure),
    }
    let mut wts: Array2<'static, f64> =
        Array2::with_bounds(0, (r.nb / 2).max(0), 0, degree).unwrap();
    b.weights(r.nb, &mut wts)?;
    out(wts.as_slice());
    let mut mx: Array1<'static, f64> = bounded(0, degree);
    b.max_value(&mut mx)?;
    array(&mx);
    errors(&b, r);
    convert(&b, r);
    basis(&b, r.u, degree)
}

fn errors(b: &impl Basis, r: &Request) {
    match b.max_error(r.dim, &r.coefficients, r.new_degree) {
        Ok(value) => print!("{} ", value.to_bits()),
        Err(failure) => fail(&failure),
    }
    match b.reduce_degree(r.dim, r.max_degree, r.tol, &r.coefficients) {
        Ok((degree, error)) => print!("{degree} {} ", error.to_bits()),
        Err(failure) => fail(&failure),
    }
    match b.average_error(r.dim, &r.coefficients, r.new_degree) {
        Ok(value) => print!("{} ", value.to_bits()),
        Err(failure) => fail(&failure),
    }
}

fn convert(b: &impl Basis, r: &Request) {
    let count: usize = ((r.to_degree + 1) * r.dim) as usize;
    let mut j: Array1<'static, f64> = bounded(0, count as i32 - 1);
    j.as_mut_slice().copy_from_slice(&r.coefficients[..count]);
    let mut result: Array1<'static, f64> = filled(0, count as i32 - 1, 91.0);
    match b.to_coefficients(r.dim, r.to_degree, &j, &mut result) {
        Ok(()) => array(&result),
        Err(failure) => fail(&failure),
    }
}

fn basis(b: &impl Basis, u: f64, upper: i32) -> Result<(), Failure> {
    let [mut v0, mut v1, mut v2, mut v3]: [Array1<'static, f64>; 4] =
        std::array::from_fn(|_| bounded(0, upper));
    b.d0(u, &mut v0)?;
    array(&v0);
    b.d1(u, &mut v0, &mut v1)?;
    array(&v0);
    array(&v1);
    b.d2(u, &mut v0, &mut v1, &mut v2)?;
    array(&v2);
    b.d3(u, &mut v0, &mut v1, &mut v2, &mut v3)?;
    for values in [&v0, &v1, &v2, &v3] {
        array(values);
    }
    Ok(())
}
