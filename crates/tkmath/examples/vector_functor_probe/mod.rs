//! Vector functor differential counterpart.
use rustycad_tkmath::math::{Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use rustycad_tkmath::math_utils::vector::*;
fn both(mut f: impl MultipleVarFunctionWithGradient, x: &Vector<'_>) {
    let mut value: f64 = 91.0;
    let mut gradient: Vector<'_> = Vector::filled(-2, -1, 92.0).unwrap();
    let ok: bool = f.value(x, &mut value);
    print!("{} {} ", i32::from(ok), value.to_bits());
    let ok: bool = f.gradient(x, &mut gradient);
    print!(
        "{} {} {} ",
        i32::from(ok),
        gradient[-2].to_bits(),
        gradient[-1].to_bits()
    );
    value = 71.0;
    gradient.init(72.0);
    let ok: bool = f.values(x, &mut value, &mut gradient);
    print!(
        "{} {} {} {} ",
        i32::from(ok),
        value.to_bits(),
        gradient[-2].to_bits(),
        gradient[-1].to_bits()
    );
}
pub(super) fn vector_functor_case(v: &[f64]) {
    let (mut x, y, mut a, b, c, d): (f64, f64, f64, f64, f64, f64) =
        (v[0], v[1], v[2], v[3], v[4], v[5]);
    match v[6] as i32 {
        1 => x = f64::NAN,
        2 => a = f64::INFINITY,
        _ => {}
    }
    let mut point: Vector<'_> = Vector::filled(4, 5, x).unwrap();
    point[5] = y;
    both(Rosenbrock::new(a, b), &point);
    both(Booth, &point);
    both(Beale, &point);
    both(Himmelblau, &point);
    both(Sphere::new(2), &point);
    both(Rastrigin::new(2, a), &point);
    let mut matrix: Matrix<'_> = Matrix::filled(-1, 0, 3, 4, 0.0).unwrap();
    matrix[(-1, 3)] = a;
    matrix[(-1, 4)] = b;
    matrix[(0, 3)] = c;
    matrix[(0, 4)] = d;
    let mut rhs: Vector<'_> = Vector::filled(7, 8, a).unwrap();
    rhs[8] = b;
    both(QuadraticForm::new(&matrix, &rhs, c), &point);
    both(LinearResidual::new(&matrix, &rhs), &point);
    let mut ackley: Ackley = Ackley::new(2, a, b, c);
    let mut value: f64 = 91.0;
    let ok: bool = ackley.value(&point, &mut value);
    print!("{} {} ", i32::from(ok), value.to_bits());
    value = 91.0;
    let ok: bool = ackley.value(&Vector::with_size(0), &mut value);
    println!("{} {}", i32::from(ok), value.to_bits());
}
