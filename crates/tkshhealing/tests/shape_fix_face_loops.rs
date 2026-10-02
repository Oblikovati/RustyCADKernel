//! Focused two-loop face split for the pinned self-intersecting prism cap.
use rustycad_tkbrep::topods::ShapeIterator;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixFace;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn bowtie_face_splits_into_two_valid_triangular_faces() {
    let source = bowtie_face();
    let fixer = ShapeFixFace::new(source.into_shape());

    assert!(fixer.perform().unwrap());
    assert!(fixer.status(ShapeExtendStatus::Done7));
    let result = fixer.result();
    assert_eq!(result.shape_type().unwrap(), ShapeEnum::Compound);
    let faces = ShapeIterator::new(&result, false, true)
        .filter(|child| child.shape_type().unwrap() == ShapeEnum::Face)
        .collect::<Vec<_>>();
    assert_eq!(faces.len(), 2);
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

fn bowtie_face() -> rustycad_tkbrep::topods::Face {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    polygon.add_point(Point3::new(0.0, 0.0, 0.0)).unwrap();
    polygon.add_point(Point3::new(1.0, 1.0, 0.0)).unwrap();
    polygon.add_point(Point3::new(1.0, 0.0, 0.0)).unwrap();
    polygon.add_point(Point3::new(0.0, 1.0, 0.0)).unwrap();
    polygon.close().unwrap();
    BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
}
