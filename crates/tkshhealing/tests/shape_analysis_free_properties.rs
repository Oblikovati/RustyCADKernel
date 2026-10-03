//! Free contour properties and lifecycle compared with pinned OCCT.
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Shape, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomCircle, GeomCurve},
    top_abs::ShapeEnum,
};
use rustycad_tkmath::gp::{Axis2, Plane3, Point3};
use rustycad_tkshhealing::shape_analysis::{
    ShapeAnalysisFreeBoundData, ShapeAnalysisFreeBoundsProperties as Properties,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeWire,
};
fn polygon(notch: bool) -> Wire {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    maker.add_point(Point3::default()).unwrap();
    maker.add_point(Point3::new(4., 0., 0.)).unwrap();
    maker
        .add_point(if notch {
            Point3::new(0., 0.1, 0.)
        } else {
            Point3::new(4., 2., 0.)
        })
        .unwrap();
    if !notch {
        maker.add_point(Point3::new(0., 2., 0.)).unwrap();
        maker.close().unwrap();
    }
    maker.wire().unwrap()
}
fn circle() -> Wire {
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(Axis2::default(), 2.).unwrap()).cast();
    let edge = BRepBuilderAPIMakeEdge::from_curve(&curve)
        .unwrap()
        .edge()
        .unwrap();
    let mut wire = BRepBuilderAPIMakeWire::new();
    wire.add_edge(&edge).unwrap();
    wire.wire().unwrap()
}
fn face() -> Shape {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0., 4., 0., 2.))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}
fn assert_native(name: &str, actual: &[f64]) {
    let line = include_str!("../../../tools/oracle/tkshhealing_free_properties.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap();
    let expected: Vec<f64> = line
        .split_whitespace()
        .skip(1)
        .map(|word| word.parse().unwrap())
        .collect();
    assert_eq!(actual.len(), expected.len());
    for (index, (&value, reference)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (value - reference).abs() < 1e-12 * reference.abs().max(1.),
            "{name}[{index}]: {value} != {reference}"
        );
    }
}
fn measured(wire: &Wire) -> Vec<f64> {
    let analyzer = Properties::new();
    let record = Handle::new(ShapeAnalysisFreeBoundData::with_bound(wire));
    let filled = analyzer.fill_properties(&record, 99.).unwrap();
    let notched = analyzer.check_notches_data(&record, 99.).unwrap();
    vec![
        f64::from(filled),
        f64::from(notched),
        record.area(),
        record.perimeter(),
        record.ratio(),
        record.width(),
        f64::from(record.nb_notches()),
    ]
}
#[test]
fn contour_samples_and_notch_counts_match_native() {
    assert_native("rectangle", &measured(&polygon(false)));
    assert_native("open_notch", &measured(&polygon(true)));
    assert_native("circle", &measured(&circle()));
}
#[test]
fn empty_perform_and_repeated_dispatch_preserve_native_accumulation() {
    let mut analyzer = Properties::new();
    assert_native(
        "default",
        &[
            f64::from(analyzer.is_loaded()),
            f64::from(analyzer.perform().unwrap()),
            f64::from(analyzer.nb_free_bounds()),
            f64::from(analyzer.check_contours(0.).unwrap()),
        ],
    );
    analyzer.init(&face(), false, false);
    analyzer.perform().unwrap();
    analyzer.perform().unwrap();
    assert_native(
        "repeated",
        &[
            f64::from(analyzer.is_loaded()),
            f64::from(analyzer.nb_closed_free_bounds()),
            f64::from(analyzer.nb_open_free_bounds()),
            analyzer.closed_free_bound(1).unwrap().area(),
        ],
    );
    analyzer.init_with_tolerance(&face(), 0.125, false, false);
    analyzer.init(&face(), true, true);
    assert_native(
        "retained",
        &[analyzer.tolerance(), f64::from(analyzer.nb_free_bounds())],
    );
}
fn edge_count(wire: &Wire) -> f64 {
    let mut edges = Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    let mut count = 0.;
    while edges.more() {
        count += 1.;
        edges.next().unwrap();
    }
    count
}
#[test]
fn rejected_notch_resets_wire_and_retains_width() {
    let analyzer = Properties::new();
    let mut notch = Wire::default();
    let mut width = 17.;
    let accepted = analyzer
        .check_notches_at(&polygon(true), 1, &mut notch, &mut width, 99.)
        .unwrap();
    assert_native("notch", &[f64::from(accepted), width, edge_count(&notch)]);
    let accepted = analyzer
        .check_notches_at(&polygon(true), 0, &mut notch, &mut width, 99.)
        .unwrap();
    assert_native("invalid", &[f64::from(accepted), width, edge_count(&notch)]);
}
#[test]
fn constructors_and_shared_result_sequences_retain_identity() {
    let source = face();
    let analyzer = Properties::with_shape(&source, false, false);
    assert!(analyzer.shape().is_same(&source));
    assert_eq!(analyzer.tolerance(), 0.);
    let configured = Properties::with_tolerance(&source, 0.125, true, false);
    assert_eq!(configured.tolerance(), 0.125);
    let record = Handle::new(ShapeAnalysisFreeBoundData::with_bound(&polygon(false)));
    analyzer.open_free_bounds().append(record.clone());
    analyzer.closed_free_bounds().append(record.clone());
    assert!(analyzer.open_free_bound(1).unwrap() == record);
    assert!(analyzer.closed_free_bound(1).unwrap() == record);
    assert_eq!(analyzer.nb_free_bounds(), 2);
    assert!(analyzer.check_contours(0.).unwrap());
    assert_eq!(
        analyzer.open_free_bound(0).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn forecast_constructor_matches_native_sewing_boundary() {
    use rustycad_tkbrep::{brep::BRepBuilder, topods::Compound};
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &face()).unwrap();
    let analyzer = Properties::with_tolerance(&compound, 0.001, false, false);
    let result = analyzer.perform().unwrap();
    assert_native(
        "forecast",
        &[
            f64::from(result),
            f64::from(analyzer.nb_closed_free_bounds()),
            f64::from(analyzer.nb_open_free_bounds()),
            analyzer.closed_free_bound(1).unwrap().area(),
        ],
    );
}
#[test]
fn short_intermediate_edge_is_included_in_attempted_notch() {
    let mut maker = BRepBuilderAPIMakePolygon::new();
    for point in [
        Point3::default(),
        Point3::new(4., 0., 0.),
        Point3::new(4., 0.001, 0.),
        Point3::new(0., 0.1, 0.),
    ] {
        maker.add_point(point).unwrap();
    }
    let analyzer = Properties::with_tolerance(&Shape::default(), 0.01, false, false);
    let mut notch = Wire::default();
    let mut width = 17.;
    let result = analyzer
        .check_notches_at(&maker.wire().unwrap(), 1, &mut notch, &mut width, 0.)
        .unwrap();
    assert_native(
        "tiny_notch",
        &[f64::from(result), width, edge_count(&notch)],
    );
}
