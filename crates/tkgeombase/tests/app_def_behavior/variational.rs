//! AppDef_Variational.
use super::{arc_line, constraints, kind};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeombase::app_def::{MultiLine, Variational, VariationalOptions};
use rustycad_tkgeombase::app_par_curves::Constraint::{self, NoConstraint, PassPoint};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::Matrix;

fn ends(n: i32) -> Vec<(i32, Constraint)> {
    vec![(1, PassPoint), (n, PassPoint)]
}

fn smoothing(
    n: i32,
    options: VariationalOptions,
) -> Result<Variational, rustycad_tkernel::standard::Failure> {
    Variational::new(&arc_line(n), 1, n, &constraints(&ends(n)), options)
}

#[test]
fn bad_arguments_are_refused_by_the_constructor() {
    let bad = |options| kind(smoothing(8, options));
    let o = VariationalOptions::default();
    assert_eq!(
        bad(VariationalOptions { max_degree: 0, ..o }),
        "Standard_DomainError"
    );
    assert_eq!(
        bad(VariationalOptions {
            max_segment: 0,
            ..o
        }),
        "Standard_DomainError"
    );
    assert_eq!(
        bad(VariationalOptions {
            continuity: Shape::G1,
            ..o
        }),
        "Standard_ConstructionError"
    );
    // MaxDegree 50 (clamped to 30 by the constructor) is pinned by the single-test
    // binary app_def_reference_cache.rs: its exception depends on the process-wide
    // reference cache that the other tests of this binary fill in parallel.
}

#[test]
fn degenerate_point_sets_raise() {
    let o = VariationalOptions::default();
    let none = constraints(&[]);
    // One point: a null chord length.
    assert_eq!(
        kind(Variational::new(&arc_line(2), 1, 1, &none, o)),
        "Standard_ConstructionError"
    );
    // Two points: the tangent estimation of the third point reads past the points.
    assert_eq!(
        kind(Variational::new(&arc_line(2), 1, 2, &none, o)),
        "Standard_OutOfRange"
    );
    // FirstPoint above 1: the last tangent is read at LastPoint, not at its offset.
    assert_eq!(
        kind(Variational::new(&arc_line(8), 2, 8, &none, o)),
        "Standard_OutOfRange"
    );
    let same = MultiLine::from_points(&Array1::from_shared(&[Point3::new(1., 2., 3.); 4], 1));
    assert_eq!(
        kind(Variational::new(&same, 1, 4, &none, o)),
        "Standard_ConstructionError"
    );
}

#[test]
fn constraints_before_the_last_point_read_past_the_table() {
    // The search of the constraint of the last point runs one entry past the table.
    let o = VariationalOptions::default();
    let early = constraints(&[(1, PassPoint), (4, PassPoint)]);
    assert_eq!(
        kind(Variational::new(&arc_line(8), 1, 8, &early, o)),
        "Standard_OutOfRange"
    );
    // A NoConstraint couple pads the table: the entry read is 0 in the port
    // (uninitialized natively).
    let padded = constraints(&[(1, PassPoint), (4, PassPoint), (5, NoConstraint)]);
    assert_eq!(
        kind(Variational::new(&arc_line(8), 1, 8, &padded, o)),
        "none"
    );
}

#[test]
fn results_need_an_approximation() {
    let mut v = smoothing(8, VariationalOptions::default()).unwrap();
    assert!(v.is_created() && !v.is_done());
    assert_eq!(kind(v.max_error()), "StdFail_NotDone");
    assert_eq!(kind(v.knots()), "StdFail_NotDone");
    let mut mat = Matrix::with_bounds(1, 1, 1, 8).unwrap();
    assert_eq!(kind(v.distance(&mut mat)), "StdFail_NotDone");
    v.approximate().unwrap();
    assert!(v.is_done());
    v.distance(&mut mat).unwrap();
    assert!(mat[(1, 1)] <= v.max_error().unwrap());
    assert_eq!(v.parameters().unwrap().array1().lower(), 1);
}

#[test]
fn the_over_constrained_flag_is_never_cleared() {
    let o = VariationalOptions {
        max_degree: 6,
        max_segment: 1,
        ..VariationalOptions::default()
    };
    let all: Vec<(i32, Constraint)> = (1..=8).map(|i| (i, PassPoint)).collect();
    let mut v = Variational::new(&arc_line(8), 1, 8, &constraints(&all), o).unwrap();
    assert!(v.is_over_constrained() && !v.is_created());
    assert_eq!(kind(v.approximate()), "StdFail_NotDone");
    // A valid set creates the criterion, yet SetConstraints still answers false.
    assert!(!v.set_constraints(&constraints(&ends(8))).unwrap());
    assert!(v.is_created() && v.is_over_constrained());
}

#[test]
fn setters_keep_the_done_flag_and_the_old_knots_handle() {
    let mut v = smoothing(10, VariationalOptions::default()).unwrap();
    v.approximate().unwrap();
    let knots: Handle<HArray1<f64>> = v.knots().unwrap().clone();
    v.set_tolerance(1e-4).unwrap();
    assert!(v.is_done());
    v.approximate().unwrap();
    // Approximate stores a new knot array: the earlier handle keeps the old one.
    assert!(knots != *v.knots().unwrap());
    assert_eq!(
        kind(v.set_criterium_weight_of(4, 1.)),
        "Standard_ConstructionError"
    );
    assert_eq!(
        kind(v.set_criterium_weight_of(1, -1.)),
        "Standard_DomainError"
    );
    assert_eq!(
        kind(v.set_criterium_weight(1., -1., 1.)),
        "Standard_DomainError"
    );
    // SetMaxDegree neither checks nor clamps: the degree is stored before the rebuild
    // refuses it.
    assert_eq!(kind(v.set_max_degree(40)), "Standard_ConstructionError");
    assert_eq!(v.max_degree(), 40);
}

#[test]
fn set_parameters_takes_the_bounds_of_its_argument() {
    let mut v = smoothing(8, VariationalOptions::default()).unwrap();
    let param = Handle::new(HArray1::from_array(&Array1::from_shared(&[0.0; 8], 3)));
    v.set_parameters(&param).unwrap();
    // The motor copies the parameters with their bounds and reads them from 1.
    assert_eq!(kind(v.approximate()), "Standard_OutOfRange");
}

#[test]
fn dump_prints_the_stream_flags_before_the_tolerance() {
    let v = smoothing(8, VariationalOptions::default()).unwrap();
    let mut text = String::new();
    v.dump(&mut text).unwrap();
    assert!(text.contains(" \nTolerance 40981.000e+00  as size Error.\n"));
    assert!(text.ends_with(" Error in approximation\n"));
    assert!(text.contains("CriteriumWeights : 4.000e-01 , 2.000e-01 , 4.000e-01\n"));
}

#[test]
fn a_c0_element_of_degree_one_has_no_second_derivative() {
    // Straight points: the C0 elements reduce to degree 1, whose second derivative
    // the native FEmTool_Curve::D2 reads outside the element; the port raises.
    let points: Vec<Point3> = (0..10)
        .map(|i| Point3::new(f64::from(i), 2. * f64::from(i), 0.))
        .collect();
    let line = MultiLine::from_points(&Array1::from_shared(&points, 1));
    let o = VariationalOptions {
        max_degree: 8,
        max_segment: 4,
        continuity: Shape::C0,
        with_min_max: true,
        tolerance: 1e-4,
        ..VariationalOptions::default()
    };
    let mut v = Variational::new(&line, 1, 10, &constraints(&ends(10)), o).unwrap();
    assert_eq!(kind(v.approximate()), "Standard_OutOfRange");
}
