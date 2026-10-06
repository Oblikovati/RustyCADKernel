//! MAT and MAT2d regressions: the lists, the zones, the curve cutting and the bisecting
//! locus of a rectangle (the values are the native ones, see the tp_mat* probe corpus).
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2};
use rustycad_tktopalgo::mat::{
    MatBasicElt, MatEdge, MatGraph, MatListOfBisector, MatListOfEdge, MatSide, MatZone,
};
use rustycad_tktopalgo::mat2d::{
    Mat2dBiInt, Mat2dCircuit, Mat2dCutCurve, Mat2dFigure, Mat2dLine, Mat2dMat2d, Mat2dTool2d,
};

// The segment from `a` to `b` as a trimmed line, as BRepMAT2d_Explorer gives it.
fn segment(a: (f64, f64), b: (f64, f64)) -> Handle<Geom2dTrimmedCurve> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx.hypot(dy);
    let direction = Direction2::new(dx, dy).expect("segment direction");
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(a.0, a.1),
        direction,
    )))
    .cast();
    Handle::new(Geom2dTrimmedCurve::new(&line, 0.0, length, true, true).expect("segment"))
}

// The figure of one closed polygon (counter-clockwise).
fn polygon(points: &[(f64, f64)]) -> Mat2dFigure {
    let mut line: Mat2dLine = Sequence::default();
    for (i, a) in points.iter().enumerate() {
        line.append(segment(*a, points[(i + 1) % points.len()]).cast());
    }
    let mut figure: Mat2dFigure = Sequence::default();
    figure.append(line);
    figure
}

fn edge(number: i32) -> Handle<MatEdge> {
    let edge = MatEdge::new();
    edge.set_edge_number(number);
    Handle::new(edge)
}

#[test]
fn list_edits_keep_the_native_order() {
    let list: Handle<MatListOfEdge> = Handle::new(MatListOfEdge::new());
    for number in 1..=3 {
        list.back_add(&edge(number));
    }
    list.front_add(&edge(0));
    assert_eq!(list.number(), 4);
    // Brackets moves from the current node: the list starts at its first one.
    let numbers = |l: &MatListOfEdge| -> Vec<i32> {
        l.first();
        (1..=l.number())
            .map(|i| l.brackets(i).unwrap().edge_number())
            .collect()
    };
    assert_eq!(numbers(&list), [0, 1, 2, 3]);
    list.first();
    list.next().unwrap();
    list.permute().unwrap();
    assert_eq!(numbers(&list), [0, 2, 1, 3]);
    // Unlink removes the current node: the second one after the permutation.
    list.first();
    list.next().unwrap();
    list.unlink().unwrap();
    assert_eq!(numbers(&list), [0, 1, 3]);
}

#[test]
fn empty_list_reads_are_native_null_dereferences() {
    let list: MatListOfBisector = MatListOfBisector::new();
    assert!(list.is_empty());
    let failure = list.current().expect_err("a null current node");
    assert_eq!(failure.exception_type(), "Standard_NullObject");
}

#[test]
fn zone_of_an_element_without_end_arc_is_empty() {
    let zone = MatZone::with_basic_elt(&Handle::new(MatBasicElt::new(1))).unwrap();
    assert!(!zone.no_empty_zone());
    assert!(zone.limited());
    assert_eq!(zone.number_of_arcs(), 0);
}

#[test]
fn cut_curve_keeps_a_segment_whole() {
    let mut cuter = Mat2dCutCurve::new();
    cuter
        .perform(&segment((0.0, 0.0), (1.0, 1.0)).cast())
        .unwrap();
    assert!(cuter.un_modified());
    assert_eq!(
        cuter.nb_curves().unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn bi_int_compares_both_indices() {
    let key = Mat2dBiInt::new(1, 2);
    assert!(key.is_equal(&Mat2dBiInt::new(1, 2)));
    assert!(!key.is_equal(&Mat2dBiInt::new(2, 1)));
    assert_eq!(key.hash_code(), Mat2dBiInt::new(1, 2).hash_code());
}

#[test]
fn rectangle_locus_has_the_native_graph() {
    let mut figure = polygon(&[(0.0, 0.0), (5.0, 0.0), (5.0, 2.0), (0.0, 2.0)]);
    let mut closed: Sequence<bool> = Sequence::default();
    closed.append(true);
    let circuit = Handle::new(Mat2dCircuit::new(JoinType::Arc, false));
    circuit.perform(&mut figure, &closed, 1, true).unwrap();
    let mut tool = Mat2dTool2d::new();
    tool.sense(MatSide::Left);
    tool.set_join_type(JoinType::Arc);
    tool.init_items(&circuit);
    let mut mat = Mat2dMat2d::new(false);
    mat.create_mat(&mut tool).unwrap();
    assert!(mat.is_done());
    assert!(!mat.semi_infinite());
    let roots: Handle<MatListOfBisector> = Handle::new(MatListOfBisector::new());
    mat.init().unwrap();
    while mat.more().unwrap() {
        roots.back_add(&mat.bisector().unwrap());
        mat.next().unwrap();
    }
    let graph = MatGraph::new();
    let items = tool.number_of_items().unwrap();
    graph
        .perform(false, &roots, items, mat.number_of_bisectors())
        .unwrap();
    let counts = (
        circuit.number_of_items(),
        mat.number_of_bisectors(),
        graph.number_of_arcs(),
        graph.number_of_nodes(),
        graph.number_of_basic_elts(),
    );
    // Four items, the four corner bisectors and the middle one; four corner nodes on
    // the contour and the two ends of the middle bisector.
    assert_eq!(counts, (4, 5, 5, 6, 4));
}
