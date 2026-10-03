//! Native-backed no-change behavior of the `ShapeFix_FixSmallFace` spot phase.
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_fix::ShapeFixFixSmallFace;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn regular_box_is_preserved_by_the_spot_phase() {
    let expected =
        include_str!("../../../tools/oracle/tkshhealing_fix_small_face_spot.expected").trim();
    assert_eq!(expected, "regular_solid 1 1");
    let mut box_maker = BRepPrimAPIMakeBox::from_sizes(1.0, 1.0, 1.0).unwrap();
    let input = box_maker.shape().unwrap().clone();
    let fixer = ShapeFixFixSmallFace::new(input.clone());
    let result = fixer.fix_spot_face().unwrap();
    assert!(result.is_same(&input));
    assert!(fixer.shape().is_same(&input));
}
