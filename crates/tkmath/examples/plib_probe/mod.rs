//! PLib, PLib_JacobiPolynomial and PLib_HermitJacobi counterparts of tkmath_plib_probe.cxx.
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::Matrix;
use rustycad_tkmath::plib::{self, PolePoint};

mod basis;
mod curves;
use basis::{jacobi_case, shape};
pub use curves::{curve_case, grid_case};

/// Sequential reader over one request's numeric fields.
pub struct Fields<'v> {
    values: &'v [f64],
    at: usize,
}
impl<'v> Fields<'v> {
    pub fn new(values: &'v [f64]) -> Self {
        Self { values, at: 0 }
    }
    pub fn real(&mut self) -> f64 {
        self.at += 1;
        self.values[self.at - 1]
    }
    pub fn int(&mut self) -> i32 {
        self.real() as i32
    }
    pub fn reals(&mut self, count: i32) -> Vec<f64> {
        (0..count.max(0)).map(|_| self.real()).collect()
    }
}

pub fn out(values: &[f64]) {
    for value in values {
        print!("{} ", value.to_bits());
    }
}
pub fn array(values: &Array1<'_, f64>) {
    out(values.as_slice());
}
pub fn fail(failure: &Failure) {
    print!("{} ", failure.exception_type());
}
pub fn bounded<T: Default>(lower: i32, upper: i32) -> Array1<'static, T> {
    Array1::with_bounds(lower, upper).unwrap()
}
pub fn filled(lower: i32, upper: i32, value: f64) -> Array1<'static, f64> {
    let mut result: Array1<'static, f64> = bounded(lower, upper);
    result.init(value);
    result
}

pub fn poly_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (u, dr, deg, dim) = (f.real(), f.int(), f.int(), f.int());
    let c: Vec<f64> = f.reals((deg + 1) * dim);
    let mut r: Vec<f64> = vec![91.0; ((dr + 1) * dim + 1) as usize];
    plib::eval_polynomial(u, dr, deg, dim, &c, &mut r).unwrap();
    out(&r);
    let mut value: Vec<f64> = vec![91.0; (dim + 1) as usize];
    plib::no_derivative_eval_polynomial(u, deg, dim, deg * dim, &c, &mut value).unwrap();
    out(&value);
    println!();
}

pub fn poly2_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (u, w) = (f.real(), f.real());
    let (udr, vdr, udeg, vdeg, dim) = (f.int(), f.int(), f.int(), f.int(), f.int());
    let c: Vec<f64> = f.reals((udeg + 1) * (vdeg + 1) * dim);
    let mut r: Vec<f64> = vec![91.0; (dim + 1) as usize];
    plib::eval_poly_2var(u, w, udr, vdr, udeg, vdeg, dim, &c, &mut r).unwrap();
    out(&r);
    println!();
}

pub fn rational_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (deg, dr, dim, all) = (f.int(), f.int(), f.int(), f.int() != 0);
    let rows: i32 = (dr.min(deg) + 1).max(1);
    let d: Vec<f64> = f.reals(rows * (dim + 1));
    let length: i32 = if all { (dr + 1) * dim + 1 } else { dim + 1 };
    let mut r: Vec<f64> = vec![91.0; length as usize];
    plib::rational_derivative(deg, dr, dim, &d, &mut r, all).unwrap();
    out(&r);
    let (dim_u, stride) = (dim as usize, (dim + 1) as usize);
    let mut p: Vec<f64> = vec![0.0; ((dr + 1) * dim + 1) as usize];
    let mut w: Vec<f64> = vec![0.0; (dr + 1) as usize];
    for i in 0..rows.min(dr + 1) as usize {
        w[i] = d[i * stride + dim_u];
        p[i * dim_u..(i + 1) * dim_u].copy_from_slice(&d[i * stride..i * stride + dim_u]);
    }
    let mut q: Vec<f64> = vec![91.0; ((dr + 1) * dim + 1) as usize];
    plib::rational_derivatives(dr, dim, &p, &w, &mut q).unwrap();
    out(&q);
    println!();
}

pub fn lagrange_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (u, dr, deg, dim) = (f.real(), f.int(), f.int(), f.int());
    let values: Vec<f64> = f.reals((deg + 1) * dim);
    let nodes: Vec<f64> = f.reals(deg + 1);
    let mut r: Vec<f64> = vec![91.0; ((dr.min(deg).max(0) + 1) * dim + 1) as usize];
    let code: i32 = plib::eval_lagrange(u, dr, deg, dim, &values, &nodes, &mut r).unwrap();
    print!("{code} ");
    out(&r);
    println!();
}

pub fn cubic_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (u, dr, dim) = (f.real(), f.int(), f.int());
    let (values, derivatives, nodes) = (f.reals(2 * dim), f.reals(2 * dim), f.reals(2));
    let mut r: Vec<f64> = vec![91.0; ((dr.clamp(0, 3) + 1) * dim + 1) as usize];
    let code: i32 =
        plib::eval_cubic_hermite(u, dr, dim, &values, &derivatives, &nodes, &mut r).unwrap();
    print!("{code} ");
    out(&r);
    println!();
}

pub fn hermite_coeffs_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (first, last, fo, lo) = (f.real(), f.real(), f.int(), f.int());
    let n: i32 = fo + lo + 2;
    let mut m: Matrix<'static> = Matrix::filled(1, n, 1, n, 91.0).unwrap();
    match plib::hermite_coefficients(first, last, fo, lo, &mut m) {
        Ok(done) => {
            print!("{} ", u8::from(done));
            for i in 1..=n {
                for j in 1..=n {
                    print!("{} ", m[(i, j)].to_bits());
                }
            }
        }
        Err(failure) => fail(&failure),
    }
    println!();
}

pub fn hermite_interp_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (dim, first, last, fo, lo) = (f.int(), f.real(), f.real(), f.int(), f.int());
    let mut fc: Array2<'static, f64> = Array2::with_bounds(1, dim, 0, fo).unwrap();
    let mut lc: Array2<'static, f64> = Array2::with_bounds(1, dim, 0, lo).unwrap();
    for (grid, order) in [(&mut fc, fo), (&mut lc, lo)] {
        for i in 1..=dim {
            for j in 0..=order {
                grid[(i, j)] = f.real();
            }
        }
    }
    let mut c: Array1<'static, f64> = filled(0, (fo + lo + 2) * dim - 1, 91.0);
    match plib::hermite_interpolate(dim, first, last, fo, lo, &fc, &lc, &mut c) {
        Ok(done) => {
            print!("{} ", u8::from(done));
            array(&c);
        }
        Err(failure) => fail(&failure),
    }
    println!();
}

pub fn params_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (order, maxdeg, code, niv, n, p) = (f.int(), f.int(), f.int(), f.int(), f.int(), f.int());
    let (mut points, mut degree) = (-7, -7);
    if let Err(failure) =
        plib::jacobi_parameters(shape(order), maxdeg, code, &mut points, &mut degree)
    {
        fail(&failure);
    }
    print!("{points} {degree} ");
    match plib::niv_constr(shape(order)) {
        Ok(level) => print!("{level} "),
        Err(failure) => fail(&failure),
    }
    match plib::constraint_order(niv) {
        Ok(result) => print!("{} ", result as i32),
        Err(failure) => fail(&failure),
    }
    match plib::bin(n, p) {
        Ok(value) => print!("{} ", value.to_bits()),
        Err(failure) => fail(&failure),
    }
    println!();
}

pub fn length_case(v: &[f64]) {
    let mut f = Fields::new(v);
    let (deg, dim, u1, u2, tol) = (f.int(), f.int(), f.real(), f.real(), f.real());
    let c: Vec<f64> = f.reals((deg + 1) * dim);
    print!(
        "{} ",
        plib::eval_length(deg, dim, &c, u1, u2).unwrap().to_bits()
    );
    let (length, error) = plib::eval_length_with_tolerance(deg, dim, &c, u1, u2, tol).unwrap();
    println!("{} {}", length.to_bits(), error.to_bits());
}

pub fn jacobi_probe_case(v: &[f64], hermit: bool) {
    jacobi_case(&mut Fields::new(v), hermit);
}

/// Writes points coordinate by coordinate, as the native `Coord(k)` loop does.
pub fn points<P: PolePoint>(values: &Array1<'_, P>) {
    for point in values.as_slice() {
        for k in 0..P::DIMENSION {
            print!("{} ", point.pole_coordinate(k).to_bits());
        }
    }
}

/// Builds a pole array from interleaved coordinates.
pub fn pole_array<P: PolePoint>(lower: i32, coordinates: &[f64]) -> Array1<'static, P> {
    let count: i32 = (coordinates.len() / P::DIMENSION) as i32;
    let mut result: Array1<'static, P> = bounded(lower, lower + count - 1);
    for (i, point) in result.as_mut_slice().iter_mut().enumerate() {
        for k in 0..P::DIMENSION {
            point.set_pole_coordinate(k, coordinates[i * P::DIMENSION + k]);
        }
    }
    result
}
