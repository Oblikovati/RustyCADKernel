use super::*;
use rustycad_tkmath::math::{Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient};
struct Objective {
    center: f64,
    fail: usize,
    calls: Vec<(i32, f64)>,
}
impl MultipleVarFunction for Objective {
    fn variables(&self) -> i32 {
        1
    }
    fn value(&mut self, x: &Vector<'_>, out: &mut f64) -> bool {
        self.calls.push((0, *x.at(0).unwrap()));
        let dx: f64 = x.at(0).unwrap() - self.center;
        *out = dx * dx;
        self.calls.len() != self.fail
    }
}
impl MultipleVarFunctionWithGradient for Objective {
    fn gradient(&mut self, x: &Vector<'_>, out: &mut Vector<'_>) -> bool {
        self.calls.push((1, *x.at(0).unwrap()));
        *out.change_at(0).unwrap() = 2.0 * (x.at(0).unwrap() - self.center);
        self.calls.len() != self.fail
    }
    fn values(&mut self, x: &Vector<'_>, value: &mut f64, g: &mut Vector<'_>) -> bool {
        self.value(x, value) && self.gradient(x, g)
    }
}
fn print_result(r: LineSearchResult) {
    print!(
        "{} {} {} {} {} ",
        r.status as i32,
        i32::from(r.is_valid),
        r.alpha.to_bits(),
        r.f_new.to_bits(),
        r.evaluations
    );
}
pub(crate) fn line_case(v: &[f64]) {
    let (center, x, d, alpha, tol, limit): (f64, f64, f64, f64, f64, u32) =
        (v[0], v[1], v[2], v[3], v[4], v[5] as u32);
    let mut point: Vector<'_> = Vector::filled(4, 4, x).unwrap();
    let direction: Vector<'_> = Vector::filled(-1, -1, d).unwrap();
    let gradient: Vector<'_> = Vector::filled(7, 7, 2.0 * (x - center)).unwrap();
    let mut f: Objective = Objective {
        center,
        fail: v[6] as usize,
        calls: Vec::new(),
    };
    let mut fx: f64 = (x - center) * (x - center);
    print_result(armijo_backtrack(
        &mut f,
        &point,
        &direction,
        &gradient,
        fx,
        [alpha, 1e-4, 0.5],
        limit,
    ));
    print_result(wolfe_search(
        &mut f,
        &point,
        &direction,
        &gradient,
        fx,
        [alpha, 1e-4, 0.9],
        limit,
    ));
    print_result(exact_line_search(
        &mut f, &point, &direction, alpha, tol, limit,
    ));
    let mut count: usize = 7;
    let improved: bool = brent_along_coordinate(
        &mut f,
        &mut point,
        0,
        &mut fx,
        [-100.0, 100.0, tol],
        limit,
        &mut count,
    );
    print!(
        "{} {} {} {} ",
        i32::from(improved),
        point[4].to_bits(),
        fx.to_bits(),
        count
    );
    let mut grad: Vector<'_> = Vector::filled(-2, -2, 91.0).unwrap();
    let ok: bool = numerical_gradient_adaptive(&mut f, &mut point, &mut grad, tol);
    print!(
        "{} {} {} ",
        i32::from(ok),
        grad[-2].to_bits(),
        point[4].to_bits()
    );
    let mut hessian: Matrix<'_> = Matrix::filled(3, 3, -2, -2, 92.0).unwrap();
    let ok: bool = numerical_hessian(&mut f, &mut point, &mut hessian, tol);
    print!(
        "{} {} {} ",
        i32::from(ok),
        hessian[(3, -2)].to_bits(),
        point[4].to_bits()
    );
    print!("{} ", f.calls.len());
    for (kind, x) in f.calls {
        print!("{} {} ", kind, x.to_bits());
    }
    println!();
}
