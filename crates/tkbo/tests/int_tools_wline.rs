//! Pinned walking-line approximation quirks, including bounds reuse and final flag inversion.
#[path = "../examples/bo_probe/fclass_faces.rs"]
mod faces;
use rustycad_tkbo::int_tools::WLineTool;
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::{
    int_patch::IntPatchWLine,
    int_surf::{IntSurfLineOn2S, IntSurfPntOn2S},
};
fn walking_line(uv: [f64; 4]) -> Handle<IntPatchWLine> {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    let mut point = IntSurfPntOn2S::new();
    point.set_parameters(uv[0], uv[1], uv[2], uv[3]);
    points.add(&point);
    points.add(&point);
    Handle::new(IntPatchWLine::new_undecided(&points, false))
}
#[test]
fn tested_nondegenerate_cylinder_boundary_still_disables_surface_approximation() {
    let cylinder = faces::face(1, 2.).unwrap();
    let plane = faces::face(0, 2.).unwrap();
    let boundary = walking_line([0., 0., 1., 0.]);
    assert!(WLineTool::not_use_surfaces_for_approx(&cylinder, &plane, &boundary, 1, 2).unwrap());
    let interior = walking_line([1., 0., 1., 0.]);
    assert!(!WLineTool::not_use_surfaces_for_approx(&cylinder, &plane, &interior, 1, 2).unwrap());
}
#[test]
fn first_surface_bounds_are_reused_for_second_surface() {
    let plane = faces::face(0, 2.).unwrap();
    let sphere = faces::face(3, 2.).unwrap();
    let pole = walking_line([1., 0., 1., std::f64::consts::FRAC_PI_2]);
    assert!(!WLineTool::not_use_surfaces_for_approx(&plane, &sphere, &pole, 1, 2).unwrap());
    assert!(WLineTool::not_use_surfaces_for_approx(&sphere, &sphere, &pole, 1, 2).unwrap());
}
#[test]
fn first_endpoint_result_short_circuits_invalid_last_index() {
    let sphere = faces::face(3, 2.).unwrap();
    let pole = walking_line([1., std::f64::consts::FRAC_PI_2, 1., 0.]);
    assert!(WLineTool::not_use_surfaces_for_approx(&sphere, &sphere, &pole, 1, 3).unwrap());
    let interior = walking_line([1., 0., 1., 0.]);
    let failure =
        WLineTool::not_use_surfaces_for_approx(&sphere, &sphere, &interior, 1, 3).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_OutOfRange");
}
