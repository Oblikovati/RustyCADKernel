//! Direct wrapper calls must retain the native virtual area criterion.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Plane3;
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus, shape_upgrade::ShapeUpgradeShapeDivideArea,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn direct_perform_uses_area_policy_without_trait_import() {
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 2., 0., 2.))
        .unwrap()
        .face()
        .unwrap();
    let divider = ShapeUpgradeShapeDivideArea::new(face.into_shape());
    divider.set_numbers_uv_splits(2, 3).unwrap();
    let changed = divider.perform(true).unwrap();
    let count = face_count(&divider.result());
    let observed = format!(
        "{} {} {}",
        i32::from(changed),
        count,
        i32::from(divider.status_flag(ShapeExtendStatus::Done))
    );
    assert_eq!(
        observed,
        include_str!("../../../tools/oracle/shape_area_dispatch.expected").trim()
    );
}

fn face_count(shape: &rustycad_tkbrep::topods::Shape) -> i32 {
    let mut faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while faces.more() {
        count += 1;
        faces.next().unwrap();
    }
    count
}
