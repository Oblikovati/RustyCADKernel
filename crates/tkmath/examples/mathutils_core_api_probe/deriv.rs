//! Finite differences, `Utils::EvaluateFinite` and the iteration-count BracketMinimum overload.
use super::{decode, flags, out};
use rustycad_tkmath::math::{Function, FunctionSet, Matrix, MultipleVarFunction, Vector};
use rustycad_tkmath::math_utils::*;

struct Scalar {
    mode: i32,
    center: f64,
    calls: Vec<f64>,
}
impl Function for Scalar {
    fn value(&mut self, x: f64, value: &mut f64) -> bool {
        self.calls.push(x);
        let c: f64 = self.center;
        *value = (x - c) * (x - c) * (x - c) + 0.5 * x * x;
        if self.mode == 1 && x > c {
            return false;
        }
        if self.mode == 2 && x > c {
            *value = f64::NAN;
        }
        if self.mode == 3 {
            *value = 1.0 / (x - c);
        }
        true
    }
}
struct Field {
    mode: i32,
    center: f64,
    calls: i32,
}
impl MultipleVarFunction for Field {
    fn variables(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, value: &mut f64) -> bool {
        self.calls += 1;
        let (x0, x1): (f64, f64) = (*x.at(0).unwrap(), *x.at(1).unwrap());
        *value = (x0 - self.center) * (x0 - self.center) * x1 + x1 * x1 * x1;
        if self.mode == 1 && x0 > self.center {
            return false;
        }
        if self.mode == 2 && x0 > self.center {
            *value = f64::NAN;
        }
        true
    }
}
impl FunctionSet for Field {
    fn variables(&self) -> i32 {
        2
    }
    fn equations(&self) -> i32 {
        2
    }
    fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> bool {
        self.calls += 1;
        let (x0, x1): (f64, f64) = (*x.at(0).unwrap(), *x.at(1).unwrap());
        *f.change_at(0).unwrap() = x0 * x1 - self.center;
        *f.change_at(1).unwrap() = x0 * x0 + x1;
        if self.mode == 1 && x0 > self.center {
            return false;
        }
        if self.mode == 2 && x0 > self.center {
            *f.change_at(1).unwrap() = f64::INFINITY;
        }
        true
    }
}
fn scalar_part(f: &mut Scalar, x: f64, step: f64, fx: f64, iterations: u32) {
    let mut d: f64 = -5.0;
    flags(&[central_difference(f, x, &mut d, step)]);
    out(&[d]);
    flags(&[forward_difference(f, x, fx, &mut d, step)]);
    out(&[d]);
    flags(&[second_derivative(f, x, fx, &mut d, step)]);
    out(&[d]);
    let (mut status, mut value): (Status, f64) = (Status::Ok, -5.0);
    let ok: bool = utils::evaluate_finite(f, x, &mut value, &mut status);
    print!("{} {} ", i32::from(ok), status as i32);
    out(&[value]);
    let m: MinBracketResult = bracket_minimum_iterations(f, x, x + step, iterations);
    print!(
        "{} {} {} ",
        m.status as i32,
        i32::from(m.is_valid),
        m.iterations
    );
    out(&[m.a, m.b, m.c, m.fa, m.fb, m.fc]);
    print!("{} ", f.calls.len());
    out(&f.calls);
}
fn vector_part(g: &mut Field, x: f64, x1: f64, step: f64) {
    let mut point: Vector<'_> = Vector::filled(3, 4, x).unwrap();
    point[4] = x1;
    let mut gradient: Vector<'_> = Vector::filled(0, 1, -9.0).unwrap();
    flags(&[numerical_gradient(g, &mut point, &mut gradient, step)]);
    out(&[gradient[0], gradient[1], point[3], point[4]]);
    gradient.init(-9.0);
    flags(&[numerical_gradient_adaptive(
        g,
        &mut point,
        &mut gradient,
        step,
    )]);
    out(&[gradient[0], gradient[1], point[3], point[4]]);
    let mut jacobian: Matrix<'_> = Matrix::filled(1, 2, 5, 6, -9.0).unwrap();
    flags(&[numerical_jacobian(g, &mut point, &mut jacobian, step)]);
    let at = |r: i32, c: i32| *jacobian.value(r, c).unwrap();
    out(&[at(1, 5), at(1, 6), at(2, 5), at(2, 6), point[3], point[4]]);
    println!("{}", g.calls);
}
/// `mathutils_core_deriv mode center x step fx x1 iterations`.
pub(in super::super) fn mathutils_core_deriv_case(v: &[f64]) {
    let v: &[f64] = &decode(v);
    let (mode, center, x, step, fx, x1) = (v[0] as i32, v[1], v[2], v[3], v[4], v[5]);
    let mut f: Scalar = Scalar {
        mode,
        center,
        calls: Vec::new(),
    };
    scalar_part(&mut f, x, step, fx, v[6] as i32 as u32);
    let mut g: Field = Field {
        mode,
        center,
        calls: 0,
    };
    vector_part(&mut g, x, x1, step);
}
