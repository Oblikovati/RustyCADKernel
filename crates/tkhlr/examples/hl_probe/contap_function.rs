//! Contour function state and all evaluation paths against OCCT.
use super::common::{Reader, attempt, end, flag, int, pnt, real, show, xyz};
use rustycad_tkhlr::contap::ContapSurfFunction as Function;
use rustycad_tkmath::math::{Matrix, Vector};
fn function_state(function: &Function) {
    int(function.nb_variables());
    int(function.nb_equations());
    int(function.function_type() as i32);
    real(function.root());
    real(function.tolerance());
    real(function.angle());
    pnt(function.point());
    pnt(function.eye());
    xyz(function.direction().xyz());
    flag(function.surface() == function.p_surface());
    flag(function.surface().is_null());
}
fn function_tangent(function: &mut Function) {
    show(function.is_tangent(), flag);
    show(function.direction3d(), |direction| xyz(*direction.xyz()));
    show(function.direction2d(), |direction| {
        real(direction.x());
        real(direction.y());
    });
    show(function.is_tangent(), flag);
}
fn function_evaluate(function: &mut Function, u: f64, v: f64) {
    let mut x = Vector::with_bounds(1, 2).unwrap();
    let mut value = Vector::with_bounds(1, 1).unwrap();
    let mut grad = Matrix::with_bounds(1, 1, 1, 2).unwrap();
    x[1] = u;
    x[2] = v;
    value[1] = 17.;
    grad[(1, 1)] = 19.;
    grad[(1, 2)] = 23.;
    show(function.value(&x, &mut value), flag);
    real(value[1]);
    function_state(function);
    function_tangent(function);
    x[1] = u + 0.125;
    show(function.derivatives(&x, &mut grad), flag);
    real(grad[(1, 1)]);
    real(grad[(1, 2)]);
    function_state(function);
    function_tangent(function);
    x[2] = v + 0.25;
    show(function.values(&x, &mut value, &mut grad), flag);
    real(value[1]);
    real(grad[(1, 1)]);
    real(grad[(1, 2)]);
    function_state(function);
    function_tangent(function);
}
/// Exercise retained outputs and cached tangents across setters; `function_case(numbers)`.
pub fn function_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = super::contap_surface::surface_adaptor(&mut reader)?;
        let (u, v, tolerance, angle) = (reader.real(), reader.real(), reader.real(), reader.real());
        let direction = reader.dir();
        let eye = reader.pnt();
        let mut function = Function::new();
        function_state(&function);
        show(function.set_surface(&surface), |()| {});
        function.set_tolerance(tolerance);
        function_state(&function);
        function.set_direction(direction);
        function_evaluate(&mut function, u, v);
        function.set_eye_angle(eye, angle);
        function_tangent(&mut function);
        function_evaluate(&mut function, u, v);
        function.set_eye(eye);
        function_evaluate(&mut function, u, v);
        function.set_direction_angle(direction, angle);
        function_evaluate(&mut function, u, v);
        function.set_direction(direction);
        function_tangent(&mut function);
        show(function.set_surface(&surface), |()| {});
        function_evaluate(&mut function, u, v);
        Ok(())
    });
    end();
}
