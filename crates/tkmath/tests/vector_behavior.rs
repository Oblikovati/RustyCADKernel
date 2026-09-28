//! Native quirks and checked failures beyond the vector legacy declarations.
use rustycad_tkmath::math::Vector;
#[test]
fn empty_norm_and_epsilon_checks_retain_failures() {
    assert_eq!(
        Vector::with_size(0).norm().unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let mut zero = Vector::filled(1, 3, 0.0).unwrap();
    assert_eq!(
        zero.normalize().unwrap_err().exception_type(),
        "Standard_NullValue"
    );
    for divisor in [0.0, -0.0, f64::EPSILON, -f64::EPSILON] {
        assert_eq!(
            zero.divide(divisor).unwrap_err().exception_type(),
            "Standard_DivideByZero"
        );
        assert_eq!(
            zero.divided(divisor).unwrap_err().exception_type(),
            "Standard_DivideByZero"
        );
    }
}
#[test]
fn inversion_retains_the_native_lower_bound_dependent_limit() {
    let mut vector = Vector::with_bounds(5, 9).unwrap();
    for index in 5..=9 {
        vector[index] = f64::from(index);
    }
    vector.invert().unwrap();
    assert_eq!(
        (vector[5], vector[6], vector[7], vector[8], vector[9]),
        (9.0, 6.0, 7.0, 8.0, 5.0)
    );
}
#[test]
fn resizing_inline_storage_retains_initialized_tail() {
    let mut vector = Vector::with_bounds(1, 10).unwrap();
    for index in 1..=10 {
        vector[index] = f64::from(index);
    }
    vector.resize(3).unwrap();
    vector.resize(10).unwrap();
    for index in 1..=10 {
        assert_eq!(vector[index], f64::from(index));
    }
}
#[test]
fn external_writes_and_assignment_preserve_destination_bounds() {
    let mut values = [1.0, 2.0, 3.0];
    {
        let mut external = Vector::from_buffer(&mut values, -1, 1).unwrap();
        external
            .initialized(&Vector::filled(5, 7, 4.0).unwrap())
            .unwrap();
        assert_eq!(external.lower(), -1);
        external[-1] = 9.0;
    }
    assert_eq!(values, [9.0, 4.0, 4.0]);
}
