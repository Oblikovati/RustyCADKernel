//! Native reset state, bounds rules, physical scaling and conical node ordering.
use rustycad_tkmath::gp::Point2;
use rustycad_tkmesh::{
    brep_mesh::{ConeRangeSplitter, DefaultRangeSplitter},
    imesh_tools::MeshParameters,
};
#[path = "support/range_splitter_fixture.rs"]
mod fixture;
fn initialized(kind: i32, bounds: i32) -> DefaultRangeSplitter {
    let mut splitter = DefaultRangeSplitter::default();
    splitter.reset(&fixture::range_face(kind, 1e-7), &MeshParameters::default());
    let b = fixture::range_bounds(bounds);
    splitter.add_point(Point2::new(b[0], b[2]));
    splitter.add_point(Point2::new(b[1], b[3]));
    splitter
}
#[test]
fn invalid_adjustment_survives_reset_until_next_adjustment() {
    let mut splitter = initialized(2, 3);
    assert!(splitter.is_valid());
    splitter.adjust_range().unwrap();
    assert!(!splitter.is_valid());
    splitter.reset(&fixture::range_face(2, 1e-7), &MeshParameters::default());
    assert!(!splitter.is_valid());
    assert_eq!(splitter.get_delta(), (1.0, 1.0));
    splitter.add_point(Point2::new(0.0, 0.0));
    splitter.add_point(Point2::new(1.0, 1.0));
    splitter.adjust_range().unwrap();
    assert!(splitter.is_valid());
}
#[test]
fn periodic_domain_limits_width_without_shifting_first_parameter() {
    let mut splitter = initialized(1, 1);
    splitter.adjust_range().unwrap();
    assert_eq!(splitter.get_range_u(), (-1.0, -1.0 + std::f64::consts::TAU));
    assert_eq!(splitter.get_range_v(), (-2.0, 3.0));
    assert!(!splitter.get_surface().is_null());
    assert!(!splitter.get_dface().is_null());
}
#[test]
fn finite_domain_clamps_overlap_but_retains_disjoint_bounds() {
    let mut splitter = initialized(2, 1);
    splitter.adjust_range().unwrap();
    assert_eq!(splitter.get_range_u(), (0.0, 1.0));
    assert_eq!(splitter.get_range_v(), (0.0, 1.0));
    let mut disjoint = initialized(2, 4);
    disjoint.adjust_range().unwrap();
    assert_eq!(disjoint.get_range_v(), (-9.0, -7.0));
}
#[test]
fn planar_scaling_uses_three_twenty_segment_isoparameters() {
    let mut splitter = initialized(2, 0);
    splitter.adjust_range().unwrap();
    let point = Point2::new(0.37, 0.62);
    assert!(
        splitter
            .scale(splitter.scale(point, true), false)
            .distance(point)
            < 1e-15
    );
    assert!((splitter.get_delta().0 - 0.5).abs() < 1e-15);
    assert!((splitter.get_delta().1 - 1.0 / 3.0).abs() < 1e-15);
    assert!(splitter.get_tolerance_uv().0 > 0.0);
    assert_eq!(splitter.point(point).unwrap().z(), 0.0);
    assert!(
        splitter
            .generate_surface_nodes(&MeshParameters::default())
            .unwrap()
            .is_null()
    );
}
#[test]
fn cone_nodes_follow_v_rows_and_u_columns() {
    let parameters = MeshParameters {
        min_size: 0.001,
        angle: 0.5,
        ..Default::default()
    };
    let face = fixture::range_face(0, 1e-7);
    face.set_deflection(0.01);
    let mut splitter = ConeRangeSplitter::default();
    splitter.reset(&face, &parameters);
    splitter.add_point(Point2::new(0.0, 0.0));
    splitter.add_point(Point2::new(1.0, 1.0));
    let mut counts = (0, 0);
    let steps = splitter.get_split_steps(&parameters, &mut counts).unwrap();
    let nodes = splitter.generate_surface_nodes(&parameters).unwrap();
    assert!(counts.0 > 0 && nodes.length() > 0);
    let first = *nodes.iter().next().unwrap();
    assert_eq!(first, Point2::new(steps.0, steps.1));
    let points: Vec<_> = nodes.iter().copied().collect();
    assert!(
        points
            .windows(2)
            .all(|pair| pair[1].y() > pair[0].y() || pair[1].x() > pair[0].x())
    );
}
#[test]
fn empty_bounds_and_wrong_cone_type_keep_native_outcomes() {
    let mut splitter = DefaultRangeSplitter::default();
    splitter.reset(&fixture::range_face(2, 1e-7), &MeshParameters::default());
    splitter.adjust_range().unwrap();
    assert!(!splitter.is_valid());
    let mut cone = ConeRangeSplitter::default();
    cone.reset(&fixture::range_face(1, 1e-7), &MeshParameters::default());
    assert!(
        cone.get_split_steps(&MeshParameters::default(), &mut (0, 0))
            .is_err()
    );
}

#[test]
fn polymorphic_cone_preserves_interior_node_override() {
    use rustycad_tkmesh::brep_mesh::RangeSplitter;
    let mut cone = ConeRangeSplitter::default();
    let splitter: &mut dyn RangeSplitter = &mut cone;
    let parameters = MeshParameters {
        min_size: 0.001,
        ..Default::default()
    };
    let face = fixture::range_face(0, 1e-7);
    face.set_deflection(0.01);
    splitter.reset(&face, &parameters);
    splitter.add_point(Point2::new(0.0, 0.0));
    splitter.add_point(Point2::new(1.0, 1.0));
    splitter.adjust_range().unwrap();
    assert!(splitter.is_valid());
    assert!(
        !splitter
            .generate_surface_nodes(&parameters)
            .unwrap()
            .is_null()
    );
    assert_eq!(splitter.range_state().get_range_u(), (0.0, 1.0));
}
