//! Pinned OCCT ShapeFix_Shape no-change and tolerance behavior.
use rustycad_tkbrep::topods::{Builder, Compound};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn valid_box_remains_valid_and_reports_no_failure() {
    let box_shape = make_box(10.0);
    let fixer = ShapeFixShape::new(box_shape);

    assert!(!fixer.perform().unwrap());
    assert!(!fixer.shape().is_null());
    assert!(!fixer.status(ShapeExtendStatus::Fail));
    assert!(is_valid(&fixer.shape()));
}

#[test]
fn compound_boxes_remain_valid_after_shape_context_application() {
    let first = make_box(10.0);
    let second = BRepPrimAPIMakeBox::at_point(
        rustycad_tkmath::gp::Point3::new(20.0, 0.0, 0.0),
        5.0,
        5.0,
        5.0,
    )
    .unwrap()
    .shape()
    .unwrap()
    .clone();
    let compound = make_compound(&first, &second);
    let fixer = ShapeFixShape::new(compound);

    assert!(!fixer.perform().unwrap());
    assert!(is_valid(&fixer.shape()));
}

#[test]
fn shape_fix_shape_exposes_native_precision_limits() {
    let fixer = ShapeFixShape::default();
    fixer.set_precision(0.01);
    fixer.set_min_tolerance(0.001);
    fixer.set_max_tolerance(0.1);

    assert!((fixer.precision() - 0.01).abs() < 1e-7);
    assert!((fixer.min_tolerance() - 0.001).abs() < 1e-7);
    assert!((fixer.max_tolerance() - 0.1).abs() < 1e-7);
}

fn make_box(size: f64) -> rustycad_tkbrep::topods::Shape {
    BRepPrimAPIMakeBox::from_sizes(size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn make_compound(
    first: &rustycad_tkbrep::topods::Shape,
    second: &rustycad_tkbrep::topods::Shape,
) -> rustycad_tkbrep::topods::Shape {
    let builder = Builder;
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, first).unwrap();
    builder.add(&mut shape, second).unwrap();
    shape
}

fn is_valid(shape: &rustycad_tkbrep::topods::Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}
