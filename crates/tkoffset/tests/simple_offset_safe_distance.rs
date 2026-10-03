//! Differential checks for MakeSimpleOffset's max-angle safe-distance helper.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Plane3;
use rustycad_tkoffset::brep_offset::{
    compute_simple_offset_max_angle, compute_simple_offset_safe_distance,
};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis,
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};

#[test]
fn null_input_returns_zero_without_measuring_angle() {
    assert_eq!(
        compute_simple_offset_safe_distance(&Shape::default(), 1.0).unwrap(),
        0.0
    );
}

#[test]
fn box_uses_largest_sampled_corner_angle_for_safe_distance() {
    let shape = box_shape();
    let maximum_angle = compute_simple_offset_max_angle(&shape).unwrap();
    let safe = compute_simple_offset_safe_distance(&shape, 1.0).unwrap();
    assert!((maximum_angle - std::f64::consts::FRAC_PI_2).abs() < 1.0e-14);
    assert!((safe - 0.31830985435280207).abs() < 1.0e-14);
}

#[test]
fn isolated_face_preserves_zero_angle_division_behavior()
-> Result<(), rustycad_tkernel::standard::Failure> {
    let mut face =
        BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 2.0))?;
    let shape = face.shape()?;
    assert_eq!(compute_simple_offset_max_angle(shape)?, 0.0);
    assert!(compute_simple_offset_safe_distance(shape, 1.0)?.is_infinite());
    assert!(compute_simple_offset_safe_distance(shape, 1.0e-7)?.is_nan());
    Ok(())
}

#[test]
fn cylinder_lateral_seam_has_parallel_normals() -> Result<(), rustycad_tkernel::standard::Failure> {
    let face = BRepPrimAPIMakeCylinder::new(2.0, 3.0).face()?;
    let shape = Shape::from(face);
    assert_eq!(compute_simple_offset_max_angle(&shape)?, 0.0);
    Ok(())
}

fn box_shape() -> Shape {
    BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}
