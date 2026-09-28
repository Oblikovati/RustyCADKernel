//! Supplemental modern optimization behavior at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::math_opt::*;
use rustycad_tkmath::math_utils::vector::{Booth, Rosenbrock, Sphere};
use rustycad_tkmath::math_utils::{Config, Status, make_vector, make_vector_with_gradient};
#[test]
fn powell_coordinate_and_custom_directions_minimize_sphere() {
    let start = Vector::filled(3, 5, 2.0).unwrap();
    let config = Config::default();
    let result = powell(&mut Sphere::new(3), &start, &config);
    assert_eq!(result.status, Status::Ok);
    assert!(result.value.unwrap() < 1e-14);
    assert_eq!(result.solution.unwrap().lower(), 0);
    let mut directions = Matrix::filled(-2, 0, 4, 6, 0.0).unwrap();
    for i in 0..3 {
        *directions.change_at(i, i).unwrap() = 2.0;
    }
    let result = powell_with_directions(&mut Sphere::new(3), &start, &directions, &config);
    assert_eq!(result.status, Status::Ok);
    assert!(result.value.unwrap() < 1e-14);
}
#[test]
fn bfgs_and_numerical_gradient_minimize_booth() {
    let start = Vector::with_size(2);
    let config = Config::default();
    let result = bfgs(&mut Booth, &start, &config);
    assert_eq!(result.status, Status::Ok);
    assert!(result.value.unwrap() < 1e-12);
    let result = bfgs_numerical(&mut Booth, &start, 1e-8, &config);
    assert_eq!(result.status, Status::Ok);
    assert!(result.value.unwrap() < 1e-12);
    let mut rosen_start = Vector::with_size(2);
    rosen_start[0] = -1.2;
    rosen_start[1] = 1.0;
    let config = Config {
        max_iterations: 1000,
        ..config
    };
    let result = bfgs(&mut Rosenbrock::default(), &rosen_start, &config);
    assert_eq!(result.status, Status::Ok);
    assert!(result.value.unwrap() < 1e-12);
}
#[test]
fn invalid_configuration_and_initial_callback_failure_have_no_solution() {
    let start = Vector::with_size(2);
    let config = Config {
        f_tolerance: 0.0,
        ..Default::default()
    };
    assert_eq!(
        powell(&mut Booth, &start, &config).status,
        Status::InvalidInput
    );
    assert_eq!(
        bfgs(&mut Booth, &start, &config).status,
        Status::InvalidInput
    );
    let mut failing = make_vector(2, |_, _| false);
    let result = powell(&mut failing, &start, &Config::default());
    assert_eq!(result.status, Status::CallbackError);
    assert!(result.solution.is_none());
    let mut gradient_failure = make_vector_with_gradient(
        2,
        |_, value| {
            *value = 1.0;
            true
        },
        |_, _| false,
    );
    let result = bfgs(&mut gradient_failure, &start, &Config::default());
    assert_eq!(result.status, Status::CallbackError);
    assert!(result.solution.is_none());
    assert_eq!(
        bfgs_numerical(&mut Booth, &start, 0.0, &Config::default()).status,
        Status::InvalidInput
    );
}
