//! Supplemental protected-hook reuse, preserving OCCT's transformation side effects.
use rustycad_tkernel::{ncollection::Array1, standard::Handle};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurfaceBase,
    geom::{GeomPlane, GeomRectangularTrimmedSurface, GeomSurface},
    geom_adaptor::{GeomAdaptorSurface, GeomAdaptorTransformedSurface},
    geom_grid_eval::GeomGridEvalSurface,
};
use rustycad_tkmath::gp::{self, Axis3, Point3, Transform3, Vector3};
use std::fmt::Write;

fn append_surface_state(output: &mut String, evaluator: &GeomGridEvalSurface<'_>) {
    let parameters: Array1<'_, f64> = Array1::with_bounds(-1, -1).unwrap();
    let points = evaluator.evaluate_grid(&parameters, &parameters).unwrap();
    write!(
        output,
        "{} {} {}",
        evaluator.get_type() as i32,
        i32::from(evaluator.has_transformation()),
        points.len()
    )
    .unwrap();
    if !points.is_empty() {
        let point = points[(1, 1)];
        write!(output, " {} {} {}", point.x(), point.y(), point.z()).unwrap();
    }
    output.push('\n');
}

fn transformed_plane() -> (Handle<dyn GeomSurface>, GeomAdaptorTransformedSurface) {
    let plane = Handle::new(GeomPlane::new(Axis3::from_axis2(gp::xoy()))).cast();
    let mut translation = Transform3::default();
    translation.set_translation(Vector3::new(10.0, 20.0, 30.0));
    let adaptor = GeomAdaptorTransformedSurface::with_surface(&plane, translation).unwrap();
    (plane, adaptor)
}

#[test]
fn surface_reinitialization_matches_native_transformation_lifecycle() {
    let (plane, transformed) = transformed_plane();
    let throwing = Adaptor3dSurfaceBase::default();
    let plain = GeomAdaptorSurface::with_surface(&plane).unwrap();
    let mut evaluator = GeomGridEvalSurface::from_adaptor(&transformed).unwrap();
    let mut output = String::new();
    check_handle_replacement(&mut evaluator, &mut output);
    check_null_then_failure(&mut evaluator, &plane, &throwing, &mut output);
    evaluator.initialization_adaptor(&transformed).unwrap();
    append_surface_state(&mut output, &evaluator);
    evaluator.initialization_adaptor(&plain).unwrap();
    append_surface_state(&mut output, &evaluator);
    let trimmed = Handle::new(
        GeomRectangularTrimmedSurface::new(&plane, (4.0, 5.0), (6.0, 7.0), (true, true)).unwrap(),
    );
    evaluator.initialization_surface(&trimmed.cast());
    append_surface_state(&mut output, &evaluator);
    assert_eq!(
        output,
        include_str!("../../../tools/oracle/tkg3d_surface_reinitialization.expected")
    );
}

fn check_handle_replacement(evaluator: &mut GeomGridEvalSurface<'_>, output: &mut String) {
    append_surface_state(output, evaluator);
    let shifted = Handle::new(GeomPlane::from_point(Point3::new(1.0, 2.0, 3.0), gp::dz()).unwrap());
    evaluator.initialization_surface(&shifted.cast());
    append_surface_state(output, evaluator);
}

fn check_null_then_failure<'a>(
    evaluator: &mut GeomGridEvalSurface<'a>,
    plane: &Handle<dyn GeomSurface>,
    throwing: &'a Adaptor3dSurfaceBase,
    output: &mut String,
) {
    evaluator.initialization_surface(&Handle::default());
    append_surface_state(output, evaluator);
    evaluator.initialization_surface(plane);
    append_surface_state(output, evaluator);
    let failure = evaluator.initialization_adaptor(throwing).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NotImplemented");
    output.push_str("GetType raised\n");
    append_surface_state(output, evaluator);
}

#[test]
fn cleared_surface_retains_transform_but_all_derivative_grids_are_empty() {
    let (_, transformed) = transformed_plane();
    let mut evaluator = GeomGridEvalSurface::from_adaptor(&transformed).unwrap();
    evaluator.initialization_surface(&Handle::default());
    let parameters: Array1<'_, f64> = Array1::with_bounds(1, 1).unwrap();
    assert!(evaluator.has_transformation());
    assert!(
        evaluator
            .evaluate_grid(&parameters, &parameters)
            .unwrap()
            .is_empty()
    );
    assert!(
        evaluator
            .evaluate_grid_d1(&parameters, &parameters)
            .unwrap()
            .is_empty()
    );
    assert!(
        evaluator
            .evaluate_grid_d2(&parameters, &parameters)
            .unwrap()
            .is_empty()
    );
    assert!(
        evaluator
            .evaluate_grid_d3(&parameters, &parameters)
            .unwrap()
            .is_empty()
    );
    assert!(
        evaluator
            .evaluate_grid_dn(&parameters, &parameters, 1, 1)
            .unwrap()
            .is_empty()
    );
}
