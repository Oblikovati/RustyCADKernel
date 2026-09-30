//! Saved contour arc states and native callback outputs.
use super::common::{Reader, attempt, end, flag, int, pnt, real, show};
use rustycad_tkhlr::contap::ContapArcFunction as Function;
use rustycad_tkmath::math::{Function as MathFunction, FunctionWithDerivative};
fn arc_function_state(function: &mut Function) {
    pnt(function.last_computed_point());
    flag(function.surface().is_null());
    show(function.nb_samples(), int);
    let index = function.get_state_number();
    int(index);
    for saved in [0, 1, index, index + 1] {
        show(function.valpoint(saved), pnt);
    }
    flag(std::ptr::eq(function.quadric(), function.quadric()));
}
fn arc_function_evaluate(function: &mut Function, u: f64) {
    let mut value = 17.;
    let mut derivative = 19.;
    let success = MathFunction::value(function, u, &mut value);
    show_captured(function, success);
    real(value);
    arc_function_state(function);
    let success = FunctionWithDerivative::derivative(function, u + 0.125, &mut derivative);
    show_captured(function, success);
    real(derivative);
    arc_function_state(function);
    let success = FunctionWithDerivative::values(function, u + 0.25, &mut value, &mut derivative);
    show_captured(function, success);
    real(value);
    real(derivative);
    arc_function_state(function);
}
fn show_captured(function: &mut Function, success: bool) {
    use rustycad_tkgeomalgo::int_start::TheArcFunction;
    show(function.take_failure().map(|()| success), flag);
}
/// All modes and saved-state resets; `arc_function_case(numbers)`.
pub fn arc_function_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| {
        let surface = super::contap_surface::surface_adaptor(&mut reader)?;
        let arc = super::contap_hcurve::arc(&mut reader)?;
        let u = reader.real();
        let angle = reader.real();
        let direction = reader.dir();
        let eye = reader.pnt();
        let mut function = Function::new();
        flag(function.surface().is_null());
        pnt(function.last_computed_point());
        int(function.get_state_number());
        pnt(function.valpoint(1)?);
        show(function.set_surface(&surface), |()| {});
        function.set_arc(&arc);
        arc_function_state(&mut function);
        function.set_direction(direction);
        arc_function_evaluate(&mut function, u);
        function.set_eye_angle(eye, angle);
        arc_function_evaluate(&mut function, u);
        function.set_eye(eye);
        arc_function_evaluate(&mut function, u);
        function.set_direction_angle(direction, angle);
        arc_function_evaluate(&mut function, u);
        function.set_arc(&arc);
        arc_function_state(&mut function);
        show(function.set_surface(&surface), |()| {});
        arc_function_state(&mut function);
        Ok(())
    });
    end();
}
