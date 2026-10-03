use super::ShapeUpgradeFaceDivide;
use rustycad_tkbrep::topods::Face;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn valid_box_face() -> Face {
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(2.0, 3.0, 4.0).unwrap();
    let shape = box_maker.shape().unwrap();
    let faces = rustycad_tkbrep::topexp::Explorer::new(
        shape,
        rustycad_tkg3d::top_abs::ShapeEnum::Face,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap();
    rustycad_tkbrep::topods::face(faces.current()).unwrap()
}

#[test]
fn default_perform_preserves_null_face_and_result() {
    let divider = ShapeUpgradeFaceDivide::default();
    assert!(!divider.perform(0.0).unwrap());
    assert!(divider.result().is_null());
}

#[test]
fn perform_creates_the_default_reshape_context() {
    let divider = ShapeUpgradeFaceDivide::new(valid_box_face());
    divider.perform(0.0).unwrap();
    assert!(!divider.context().is_null());
}

#[test]
fn init_replaces_the_face_and_result() {
    let divider = ShapeUpgradeFaceDivide::default();
    divider.init(&Face::default());
    assert!(divider.result().is_null());
}
