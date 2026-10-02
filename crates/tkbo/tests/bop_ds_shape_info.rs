//! Regression cases for BOPDS_ShapeInfo sentinels and cached metadata.
use rustycad_tkbo::bop_ds::ShapeInfo;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{bnd::BoundingBox3, gp::Point3};

#[test]
fn default_values_and_shape_type_participation_match_native() {
    let mut info = ShapeInfo::default();
    assert!(info.shape().is_null());
    assert_eq!(info.shape_type(), ShapeEnum::Shape);
    assert!(info.box_bounds().is_void());
    assert!(info.sub_shapes().is_empty());
    assert_eq!(info.reference(), -1);
    assert!(!info.has_reference());
    assert_eq!(info.flag(), -1);
    assert!(!info.has_flag());
    for shape_type in ShapeEnum::ALL {
        info.set_shape_type(shape_type);
        let brep = matches!(
            shape_type,
            ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face
        );
        assert_eq!(info.has_brep(), brep);
        assert_eq!(
            info.is_interfering(),
            brep || shape_type == ShapeEnum::Solid
        );
    }
    let mut flag = 99;
    assert!(!info.has_flag_into(&mut flag));
    assert_eq!(flag, -1);
}

#[test]
fn shape_box_indices_and_flags_preserve_copy_and_mutation_rules() {
    let mut info = ShapeInfo::default();
    let shape = Shape::default();
    info.set_shape(&shape);
    info.set_box(&BoundingBox3::new(
        Point3::new(-1.0, -2.0, -3.0),
        Point3::new(4.0, 5.0, 6.0),
    ));
    info.change_box().set_gap(0.5);
    info.set_reference(4);
    info.set_flag(3);
    info.change_sub_shapes().extend([4, 7, 4]);
    assert!(info.shape().is_null());
    assert_eq!(
        info.box_bounds().get().unwrap(),
        [-1.5, -2.5, -3.5, 4.5, 5.5, 6.5]
    );
    assert!(info.has_sub_shape(7));
    assert!(!info.has_sub_shape(5));
    assert_eq!(info.sub_shapes(), [4, 7, 4]);
    assert!(info.has_reference());
    assert_eq!(info.reference(), 4);
    assert!(info.has_flag());
    let mut flag = -9;
    assert!(info.has_flag_into(&mut flag));
    assert_eq!(flag, 3);
    info.set_flag(-2);
    assert!(!info.has_flag());
}

#[test]
fn dump_uses_native_topabs_name_and_subshape_order() {
    let mut info = ShapeInfo::default();
    info.set_shape_type(ShapeEnum::Face);
    info.change_sub_shapes().extend([3, 1, 3]);
    let mut output = String::new();
    info.dump(&mut output).unwrap();
    assert_eq!(output, " FACE { 3 1 3 }");
}
