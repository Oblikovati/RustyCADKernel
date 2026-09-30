//! Behaviour tests for the knot, pole, weight and parametrisation edits of
//! Geom_BSplineSurface.
use super::common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use super::{array, assert_same_point, bicubic, nonuniform, u_periodic, wavy};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::bsplslib::MoveRange;
use rustycad_tkmath::gp::Point3;

const SAMPLES: [(f64, f64); 3] = [(0.1, 0.2), (0.5, 0.5), (0.8, 0.35)];

fn sample(surface: &impl GeomSurface) -> Vec<Point3> {
    SAMPLES
        .iter()
        .map(|&(u, v)| surface.value(u, v).unwrap())
        .collect()
}

fn assert_same_shape(before: &[Point3], surface: &impl GeomSurface, tolerance: f64) {
    for (point, after) in before.iter().zip(sample(surface)) {
        assert_same_point(*point, after, tolerance);
    }
}

#[test]
fn Geom_BSplineSurfaceBehavior__ExchangeUV() {
    let surface = nonuniform(Some(|i, j| f64::from(i + 2 * j)));
    surface.exchange_uv().unwrap();
    assert_eq!((surface.u_degree(), surface.v_degree()), (2, 3));
    assert_eq!((surface.nb_u_poles(), surface.nb_v_poles()), (3, 5));
    assert_eq!(surface.v_knots().as_slice(), &[0.0, 0.5, 1.0]);
    assert_same_point(surface.pole(3, 2).unwrap(), wavy(2, 3), 0.0);
    EXPECT_DOUBLE_EQ(surface.weight(3, 2).unwrap(), 8.0);
    assert!(surface.is_u_rational() && surface.is_v_rational());
}

#[test]
fn Geom_BSplineSurfaceBehavior__Periodicity() {
    let periodic = u_periodic();
    let before = periodic.value(1.3, 0.4).unwrap();
    periodic.set_u_not_periodic().unwrap();
    assert!(!periodic.is_u_periodic());
    assert_same_point(periodic.value(1.3, 0.4).unwrap(), before, 1e-14);
    periodic.set_u_periodic().unwrap();
    assert!(periodic.is_u_periodic());
    // Already non-periodic in V: SetVNotPeriodic changes nothing.
    periodic.set_v_not_periodic().unwrap();
    assert_eq!(periodic.nb_v_poles(), 2);
    let surface = bicubic();
    surface.set_v_periodic().unwrap();
    assert!(surface.is_v_periodic());
    EXPECT_DOUBLE_EQ(surface.v_period().unwrap(), 1.0);
}

#[test]
fn Geom_BSplineSurfaceBehavior__SetOrigin() {
    let periodic = u_periodic();
    let before = periodic.value(2.5, 0.5).unwrap();
    periodic.set_u_origin(3).unwrap();
    EXPECT_DOUBLE_EQ(periodic.u_knot(1).unwrap(), 2.0);
    assert_eq!(periodic.bounds(), (2.0, 7.0, 0.0, 1.0));
    assert_same_point(periodic.value(2.5, 0.5).unwrap(), before, 1e-14);
    // Quirk: the index error of SetUOrigin names Geom_BSplineCurve.
    let error = periodic.set_u_origin(9).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert!(
        error.message().contains("Geom_BSplineCurve"),
        "{}",
        error.message()
    );
    let error = periodic.set_v_origin(1).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
}

#[test]
fn Geom_BSplineSurfaceBehavior__Reverse() {
    let surface = nonuniform(None);
    let before = surface.value(0.2, 0.7).unwrap();
    surface.u_reverse().unwrap();
    assert_eq!(surface.u_knots().as_slice(), &[0.0, 0.5, 1.0]);
    assert_same_point(surface.pole(1, 1).unwrap(), wavy(5, 1), 0.0);
    EXPECT_DOUBLE_EQ(surface.u_reversed_parameter(0.2).unwrap(), 0.8);
    assert_same_point(surface.value(0.8, 0.7).unwrap(), before, 1e-14);
    surface.v_reverse().unwrap();
    assert_same_point(surface.pole(1, 1).unwrap(), wavy(5, 3), 0.0);
    EXPECT_DOUBLE_EQ(
        surface.v_reversed_parameter(0.7).unwrap(),
        0.30000000000000004,
    );
}

#[test]
fn Geom_BSplineSurfaceBehavior__IncreaseDegree() {
    let surface = bicubic();
    let before = sample(&surface);
    surface.increase_degree(4, 5).unwrap();
    assert_eq!((surface.u_degree(), surface.v_degree()), (4, 5));
    assert_same_shape(&before, &surface, 1e-14);
    assert_eq!(
        surface.increase_degree(3, 5).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    // Quirk: U is raised and stored before the V degree is checked.
    assert!(surface.increase_degree(6, 2).is_err());
    assert_eq!((surface.u_degree(), surface.v_degree()), (6, 5));
}

#[test]
fn Geom_BSplineSurfaceBehavior__InsertAndRemoveKnots() {
    let surface = bicubic();
    let before = sample(&surface);
    surface
        .insert_u_knots(&array(&[0.25, 0.75]), &array(&[1, 2]), 0.0, false)
        .unwrap();
    assert_eq!(surface.u_multiplicities().as_slice(), &[4, 1, 2, 4]);
    assert_eq!(surface.nb_u_poles(), 7);
    surface.insert_v_knot(0.5, 1, 0.0, false).unwrap();
    surface.insert_v_knot(0.5, 1, 0.0, true).unwrap();
    assert_eq!(surface.v_multiplicities().as_slice(), &[4, 2, 4]);
    surface
        .insert_v_knots(&array(&[0.3]), &array(&[1]), 0.0, false)
        .unwrap();
    surface.insert_u_knot(0.5, 1, 0.0, false).unwrap();
    assert_same_shape(&before, &surface, 1e-14);
    assert!(surface.remove_u_knot(3, 0, 1e-9).unwrap());
    assert!(surface.remove_v_knot(2, 0, 1e-9).unwrap());
    assert_eq!(surface.nb_u_knots(), 4);
    assert_same_shape(&before, &surface, 1e-12);
    surface.set_pole(3, 3, Point3::new(0.0, 0.0, 9.0)).unwrap();
    assert!(!surface.remove_u_knot(2, 0, 1e-9).unwrap());
    assert!(surface.remove_u_knot(9, 0, 1e-9).is_err());
    // Quirk: an insertion that adds no pole stores nothing, not even the new knot.
    surface.insert_u_knot(0.6, 0, 0.0, false).unwrap();
    assert_eq!(surface.nb_u_knots(), 4);
}

#[test]
fn Geom_BSplineSurfaceBehavior__Multiplicities() {
    let surface = nonuniform(None);
    let before = sample(&surface);
    surface.increase_u_multiplicity(2, 2).unwrap();
    assert_eq!(surface.u_multiplicity(2).unwrap(), 2);
    surface.increment_u_multiplicity(2, 2, 1).unwrap();
    assert_eq!(surface.u_multiplicity(2).unwrap(), 3);
    surface.increase_u_multiplicity_range(1, 3, 3).unwrap();
    assert_eq!(surface.u_multiplicities().as_slice(), &[4, 3, 4]);
    surface.insert_v_knot(0.5, 1, 0.0, false).unwrap();
    surface.increase_v_multiplicity(2, 2).unwrap();
    // Added multiplicities are capped at the degree (BSplCLib::PrepareInsertKnots).
    surface.increment_v_multiplicity(2, 2, 1).unwrap();
    assert_eq!(surface.v_multiplicity(2).unwrap(), 2);
    surface.increase_v_multiplicity_range(2, 2, 1).unwrap();
    assert_eq!(surface.v_multiplicity(2).unwrap(), 2);
    assert_same_shape(&before, &surface, 1e-14);
    surface.increase_u_multiplicity(2, 5).unwrap();
    assert_eq!(surface.u_multiplicity(2).unwrap(), 3);
    // Natively a range past the knots reads beyond them; here it is refused.
    let error = surface.increment_u_multiplicity(2, 7, 1).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn Geom_BSplineSurfaceBehavior__SetKnots() {
    let surface = nonuniform(None);
    surface.set_u_knot(2, 0.4).unwrap();
    EXPECT_DOUBLE_EQ(surface.u_knot(2).unwrap(), 0.4);
    assert_eq!(
        surface.set_u_knot(2, 1.5).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    surface.set_u_knot_with_multiplicity(2, 0.6, 2).unwrap();
    assert_eq!(
        (
            surface.u_knot(2).unwrap(),
            surface.u_multiplicity(2).unwrap()
        ),
        (0.6, 2)
    );
    surface.set_u_knots(&array(&[-1.0, 0.0, 2.0])).unwrap();
    assert_eq!(surface.bounds(), (-1.0, 2.0, 0.0, 1.0));
    surface.set_v_knot(1, -0.5).unwrap();
    surface.set_v_knot_with_multiplicity(2, 2.0, 3).unwrap();
    assert_eq!(surface.v_knots().as_slice(), &[-0.5, 2.0]);
    surface.set_v_knots(&array(&[0.0, 4.0])).unwrap();
    EXPECT_DOUBLE_EQ(surface.v_knot(2).unwrap(), 4.0);
    assert!(surface.set_v_knots(&array(&[1.0, 1.0])).is_err());
}

#[test]
fn Geom_BSplineSurfaceBehavior__Segment() {
    let surface = nonuniform(Some(|i, j| 1.0 + 0.1 * f64::from(i * j)));
    let before = surface.value(0.5, 0.5).unwrap();
    surface
        .segment((0.2, 0.8), (0.1, 0.9), (1e-9, 1e-9))
        .unwrap();
    assert_eq!(surface.bounds(), (0.2, 0.8, 0.1, 0.9));
    assert_same_point(surface.value(0.5, 0.5).unwrap(), before, 1e-14);
    let error = surface
        .segment((0.8, 0.2), (0.1, 0.9), (1e-9, 1e-9))
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    // CheckAndSegment leaves a direction whose bounds are the end knots untouched.
    let knots = surface.u_knots();
    surface
        .check_and_segment((0.2, 0.8), (0.3, 0.6), (1e-9, 1e-9))
        .unwrap();
    assert_eq!(surface.u_knots().as_slice(), knots.as_slice());
    assert_eq!(surface.bounds(), (0.2, 0.8, 0.3, 0.6));
}

#[test]
fn Geom_BSplineSurfaceBehavior__SetPoles() {
    let surface = bicubic();
    surface.set_pole(2, 3, Point3::new(5.0, 5.0, 5.0)).unwrap();
    assert_same_point(surface.pole(2, 3).unwrap(), Point3::new(5.0, 5.0, 5.0), 0.0);
    assert_eq!(
        surface
            .set_pole(0, 1, Point3::default())
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
    let row = array(&[Point3::new(1.0, 0.0, 0.0); 4]);
    surface.set_pole_row(4, &row).unwrap();
    assert_same_point(surface.pole(4, 2).unwrap(), Point3::new(1.0, 0.0, 0.0), 0.0);
    surface.set_pole_col(1, &row).unwrap();
    assert_same_point(surface.pole(3, 1).unwrap(), Point3::new(1.0, 0.0, 0.0), 0.0);
    assert!(
        surface
            .set_pole_col(1, &array(&[Point3::default(); 5]))
            .is_err()
    );
    surface
        .set_pole_weighted(1, 1, Point3::new(0.0, 0.0, 1.0), 2.0)
        .unwrap();
    assert!(surface.is_u_rational() && surface.is_v_rational());
    // SetPole(i, j, P, W) sets the weight first: a bad weight leaves the pole alone.
    assert!(
        surface
            .set_pole_weighted(1, 1, Point3::new(7.0, 7.0, 7.0), 0.0)
            .is_err()
    );
    assert_same_point(surface.pole(1, 1).unwrap(), Point3::new(0.0, 0.0, 1.0), 0.0);
    surface
        .set_pole_row_weighted(2, &row, &array(&[3.0; 4]))
        .unwrap();
    EXPECT_DOUBLE_EQ(surface.weight(2, 4).unwrap(), 3.0);
    surface
        .set_pole_col_weighted(4, &row, &array(&[0.5; 4]))
        .unwrap();
    EXPECT_DOUBLE_EQ(surface.weight(1, 4).unwrap(), 0.5);
}

#[test]
fn Geom_BSplineSurfaceBehavior__SetWeights() {
    let surface = bicubic();
    surface.set_weight(2, 2, 2.0).unwrap();
    assert!(surface.is_u_rational() && surface.is_v_rational());
    surface.set_weight(2, 2, 1.0).unwrap();
    assert!(!surface.is_u_rational() && !surface.is_v_rational());
    assert_eq!(
        surface.set_weight(1, 1, -1.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    // Row 1 differs from row 2: the V isos become rational, the U isos do not.
    surface.set_weight_row(1, &array(&[2.0; 4])).unwrap();
    assert!(!surface.is_u_rational() && surface.is_v_rational());
    // Quirk: SetWeightCol stores the weights before the bad one, then raises.
    assert!(
        surface
            .set_weight_col(3, &array(&[5.0, 0.0, 5.0, 5.0]))
            .is_err()
    );
    EXPECT_DOUBLE_EQ(surface.weight(1, 3).unwrap(), 5.0);
    EXPECT_DOUBLE_EQ(surface.weight(3, 3).unwrap(), 1.0);
    // Quirk: an array inside the row bounds but shorter than the row is accepted and
    // overwrites only its own columns.
    surface.set_weight_row(2, &array(&[4.0; 3])).unwrap();
    EXPECT_DOUBLE_EQ(surface.weight(2, 3).unwrap(), 4.0);
    EXPECT_DOUBLE_EQ(surface.weight(2, 4).unwrap(), 1.0);
    let error = surface.set_weight_row(1, &array(&[2.0; 5])).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    surface.set_weight_col(4, &array(&[1.5; 4])).unwrap();
    EXPECT_DOUBLE_EQ(surface.weight(4, 4).unwrap(), 1.5);
}

#[test]
fn Geom_BSplineSurfaceBehavior__MovePoint() {
    let surface = bicubic();
    let range = MoveRange {
        u_index1: 1,
        u_index2: 4,
        v_index1: 1,
        v_index2: 4,
    };
    let target = Point3::new(0.5, 1.5, 2.0);
    let moved = surface.move_point(0.5, 0.5, target, range).unwrap();
    assert_ne!(moved, (0, 0, 0, 0));
    let reached = surface.value(0.5, 0.5).unwrap();
    EXPECT_NEAR(reached.distance(&target), 0.0, 1e-12);
    // A point already on the surface still reports the moved range, and stays.
    assert_eq!(surface.move_point(0.5, 0.5, reached, range).unwrap(), moved);
    assert_same_point(surface.value(0.5, 0.5).unwrap(), reached, 1e-15);
}
