//! Circle storage, native squared tolerance and spatial-registration regressions.
use rustycad_tkernel::{ncollection::CellAction, standard::Handle};
use rustycad_tkmath::gp::{Circle2, Xy};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshCircle as Circle, BRepMeshCircleInspector as Inspector, BRepMeshCircleTool as Tool,
};
#[path = "support/circle_random.rs"]
mod random;
fn inspector() -> Inspector {
    Inspector::new(1., 4, &Handle::default())
}
fn tool() -> Tool {
    let mut tool = Tool::new(&Handle::default());
    tool.set_min_max_size(Xy::new(-3., -3.), Xy::new(3., 3.));
    tool.set_cell_size(1.).unwrap();
    tool
}
fn gp_circle(radius: f64) -> Circle2 {
    Circle2::new(Default::default(), radius).unwrap()
}
#[test]
fn compact_circle_preserves_arbitrary_radii_and_coordinates() {
    let mut circle = Circle::default();
    assert_eq!(*circle.location(), Xy::default());
    assert_eq!(circle.radius(), 0.);
    circle.set_location(Xy::new(2., 3.));
    circle.set_radius(-1.);
    assert_eq!(circle, Circle::new(Xy::new(2., 3.), -1.));
    circle.set_radius(f64::NAN);
    assert!(circle.radius().is_nan());
}
#[test]
fn inspector_static_geometry_and_zero_based_sparse_binding() {
    assert_eq!(Inspector::DIMENSION, 2);
    assert_eq!(Inspector::coord(1, &Xy::new(2., 3.)).unwrap(), 3.);
    assert!(Inspector::coord(-1, &Xy::default()).is_err());
    assert_eq!(Inspector::shift(&Xy::new(2., 3.), -1.), Xy::new(1., 2.));
    assert!(Inspector::is_equal(2, 2));
    assert!(!Inspector::is_equal(2, 1));
    let mut inspector = inspector();
    inspector.bind(3, Circle::new(Xy::default(), 1.)).unwrap();
    assert_eq!(inspector.circles().length(), 4);
    assert_eq!(inspector.circles().value(1).unwrap().radius(), 0.);
    assert!(inspector.bind(-1, Circle::default()).is_err());
    assert_eq!(inspector.circles().length(), 4);
}
#[test]
fn inspector_keeps_bug27119_squared_tolerance_and_duplicate_hits() {
    let mut inspector = inspector();
    inspector.bind(0, Circle::new(Xy::default(), 100.)).unwrap();
    inspector.set_point(Xy::new(100.5, 0.));
    assert_eq!(inspector.inspect(0).unwrap(), CellAction::Keep);
    assert!(inspector.get_shot_circles().is_empty());
    inspector.set_point(Xy::new(100., 0.));
    inspector.inspect(0).unwrap();
    inspector.inspect(0).unwrap();
    assert_eq!(inspector.get_shot_circles().extent(), 2);
    inspector.get_shot_circles().append(9);
    assert_eq!(inspector.get_shot_circles().extent(), 3);
    inspector.set_point(Xy::default());
    assert!(inspector.get_shot_circles().is_empty());
}
#[test]
fn inspector_purges_negative_radii_but_keeps_nan() {
    let mut inspector = inspector();
    inspector.bind(0, Circle::new(Xy::default(), -1.)).unwrap();
    assert_eq!(inspector.inspect(0).unwrap(), CellAction::Purge);
    inspector.circle(0).unwrap().set_radius(f64::NAN);
    assert_eq!(inspector.inspect(0).unwrap(), CellAction::Keep);
    assert!(inspector.get_shot_circles().is_empty());
}
#[test]
fn zero_radius_delete_is_a_noop_and_positive_radius_is_purged() {
    let mut tool = tool();
    tool.bind(0, &gp_circle(0.)).unwrap();
    tool.delete(0).unwrap();
    assert_eq!(tool.select(&Xy::default()).unwrap().first(), Some(&0));
    tool.bind(1, &gp_circle(1.)).unwrap();
    tool.delete(1).unwrap();
    assert_eq!(tool.select(&Xy::default()).unwrap().extent(), 1);
    assert!(!tool.is_empty());
}
#[test]
fn rebinding_keeps_duplicate_cell_references_and_selection_list_is_live() {
    let mut tool = tool();
    tool.bind(1, &gp_circle(1.)).unwrap();
    tool.bind(1, &gp_circle(2.)).unwrap();
    let hits = tool.select(&Xy::default()).unwrap();
    assert_eq!(hits.extent(), 2);
    hits.append(99);
    assert_eq!(hits.last(), Some(&99));
    assert_eq!(tool.select(&Xy::default()).unwrap().extent(), 2);
    tool.moc_bind(1).unwrap();
    assert!(tool.select(&Xy::default()).unwrap().is_empty());
}
#[test]
fn initialization_and_cell_reset_retain_circle_storage() {
    let mut tool = Tool::with_reserved(100, &Handle::default());
    assert!(tool.is_empty());
    tool.moc_bind(4).unwrap();
    tool.init(0);
    assert!(!tool.is_empty());
    tool.bind(1, &gp_circle(1.)).unwrap();
    tool.set_cell_size(0.5).unwrap();
    assert!(!tool.is_empty());
    assert!(tool.select(&Xy::default()).unwrap().is_empty());
    tool.set_cell_sizes(0.5, 1.).unwrap();
    assert_eq!(
        tool.set_cell_size(1.).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}
#[test]
fn point_binding_preserves_old_circle_when_construction_fails() {
    let mut tool = tool();
    let points = [Xy::new(-1., 0.), Xy::new(0., 1.), Xy::new(1., 0.)];
    assert!(tool.bind_points(1, &points).unwrap());
    assert!(!tool.bind_points(1, &[Xy::default(); 3]).unwrap());
    assert_eq!(tool.select(&Xy::default()).unwrap().first(), Some(&1));
}
#[test]
fn make_circle_preserves_failed_outputs_and_enlarges_radius() {
    let mut center = Xy::new(7., 8.);
    let mut radius = 9.;
    assert!(!Tool::make_circle(
        &[Xy::default(); 3],
        &mut center,
        &mut radius
    ));
    assert_eq!(center, Xy::new(7., 8.));
    assert_eq!(radius, 9.);
    let points = [Xy::new(-1., 0.), Xy::new(0., 1.), Xy::new(1., 0.)];
    assert!(Tool::make_circle(&points, &mut center, &mut radius));
    assert_eq!(center, Xy::default());
    assert_eq!(radius, 1. + 2. * f64::EPSILON);
    assert!(
        points
            .iter()
            .all(|point| random::on_circle(*point, center, radius))
    );
}
#[test]
fn seeded_linux_random_stream_and_retry_fixture_are_exact() {
    let mut random = random::CircleRandom::seeded();
    assert_eq!(random.next_integer(), 71876166);
    assert_eq!(random.next_integer(), 708592740);
    let points = random.accepted_triangle();
    assert!(!random::degenerate(&points));
    assert!(random::degenerate(&[Xy::default(); 3]));
}
#[test]
fn checked_bind_failure_retains_native_stale_cell_and_reports_bounds_gap() {
    let mut tool = tool();
    assert!(tool.bind(-1, &gp_circle(1.)).is_err());
    assert!(tool.is_empty());
    assert_eq!(
        tool.select(&Xy::default()).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    let mut inspector = inspector();
    inspector.bind(0, Circle::default()).unwrap();
    for index in [-1, 1] {
        assert!(inspector.circle(index).is_err());
        assert!(inspector.inspect(index).is_err());
    }
}

#[test]
fn nan_radius_registration_does_not_duplicate_a_later_finite_binding() {
    let mut tool = tool();
    tool.bind(1, &gp_circle(f64::NAN)).unwrap();
    tool.bind(1, &gp_circle(2.)).unwrap();
    assert_eq!(tool.select(&Xy::default()).unwrap().extent(), 1);
}
