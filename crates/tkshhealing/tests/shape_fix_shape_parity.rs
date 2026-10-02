//! Pinned OCCT ShapeFix_Shape no-change and tolerance behavior.
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Compound, ShapeIterator, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakePrism};
use rustycad_tkshhealing::shape_analysis::{ShapeAnalysisWire, ShapeAnalysisWireOrder};
use rustycad_tkshhealing::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkshhealing::shape_fix::{ShapeFixShape, ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

#[test]
fn fix_valid_box() {
    let box_shape = make_box(10.0);
    let fixer = ShapeFixShape::new(box_shape);

    assert!(!fixer.perform().unwrap());
    assert!(!fixer.shape().is_null());
    assert!(!fixer.status(ShapeExtendStatus::Fail));
    assert!(is_valid(&fixer.shape()));
}

#[test]
fn fix_compound() {
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
fn set_precision() {
    let fixer = ShapeFixShape::default();
    fixer.set_precision(0.01);
    fixer.set_min_tolerance(0.001);
    fixer.set_max_tolerance(0.1);

    assert!((fixer.precision() - 0.01).abs() < 1e-7);
    assert!((fixer.min_tolerance() - 0.001).abs() < 1e-7);
    assert!((fixer.max_tolerance() - 0.1).abs() < 1e-7);
}

#[test]
fn heal_prism_from_self_intersecting_face() {
    let cap = bowtie_face();
    let prism =
        BRepPrimAPIMakePrism::new(&cap.into_shape(), Vector3::new(0.0, 0.0, 1.0), true, true)
            .unwrap()
            .shape()
            .unwrap()
            .clone();
    let fixer = ShapeFixShape::new(prism);

    assert!(fixer.perform().unwrap());
    let result = fixer.shape();
    assert_eq!(count_kind(&result, ShapeEnum::Face), 8);
    assert_eq!(count_kind(&result, ShapeEnum::Solid), 1);
    assert!(is_valid(&result));
}

#[test]
fn status_after_fix() {
    let fixer = ShapeFixShape::new(make_box(10.0));

    fixer.perform().unwrap();

    assert!(!fixer.status(ShapeExtendStatus::Fail));
}

#[test]
fn occ24881_valid_box_has_no_wire_gap_failures() {
    let box_shape = BRepPrimAPIMakeBox::from_sizes(100.0, 200.0, 300.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let wire_fixer = ShapeFixWire::new();
    let mut wire_count = 0;

    for face_shape in Explorer::new(&box_shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap() {
        let face = topods::face(&face_shape).unwrap();
        for child in ShapeIterator::new(&face_shape, true, true) {
            if child.shape_type().unwrap() != ShapeEnum::Wire {
                continue;
            }
            let wire = topods::wire(&child).unwrap();
            wire_fixer.load(&wire).unwrap();
            wire_fixer.set_face(&face).unwrap();
            wire_fixer.fix_reorder().unwrap();
            wire_fixer.fix_gaps_3d().unwrap();
            assert!(!wire_fixer.status_gaps_3d(ShapeExtendStatus::Fail));
            wire_fixer.fix_gaps_2d().unwrap();
            assert!(!wire_fixer.status_gaps_2d(ShapeExtendStatus::Fail));
            wire_count += 1;
        }
    }

    assert_eq!(wire_count, 6);
}

#[test]
fn occ25014_empty_wire_reorder_does_not_throw() {
    let mut empty_wire = Wire::default();
    Builder.make_wire(&mut empty_wire);
    let empty_data = Handle::new(ShapeExtendWireData::new());
    let analyzer = Handle::new(
        ShapeAnalysisWire::from_wire_data(&empty_data, &Default::default(), 1e-7).unwrap(),
    );
    let mut order = ShapeAnalysisWireOrder::with_mode(true, 1e-7, false);
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);

    analyzer
        .check_order_with(&mut order, true, true, false)
        .unwrap();
    fixer.fix_reorder_with(&order).unwrap();

    assert_eq!(order.nb_edges(), 0);
    assert_eq!(empty_wire.shape_type().unwrap(), ShapeEnum::Wire);
}

fn make_box(size: f64) -> rustycad_tkbrep::topods::Shape {
    BRepPrimAPIMakeBox::from_sizes(size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
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

fn count_kind(shape: &rustycad_tkbrep::topods::Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
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
