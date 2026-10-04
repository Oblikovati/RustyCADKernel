//! Native differential tests for standard offset maker state.
use rustycad_tkmath::{geom_abs::JoinType, gp::Plane3};
use rustycad_tkoffset::brep_offset::{
    BRepOffsetMakeOffset, BRepOffsetMakeOffsetConfiguration, BRepOffsetMode,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

#[test]
fn standard_offset_new_has_defined_null_shape_fields() {
    let maker = BRepOffsetMakeOffset::new();
    assert!(maker.shape().is_null());
    assert!(maker.init_shape().is_null());
    assert!(maker.closing_faces().is_empty());
    assert_eq!(maker.linearization_allowed(), None);
    assert!(maker.bad_shape().is_null());
    assert_eq!(maker.error(), None);
    assert!(maker.configuration().is_err());
}

#[test]
fn standard_offset_initialize_add_face_and_clear_match_native_state() {
    let (shape, face) = planar_input();
    let mut maker = BRepOffsetMakeOffset::new();
    maker.allow_linearization(false);
    assert_eq!(maker.linearization_allowed(), Some(false));
    initialize_arc(&mut maker, &shape);
    assert_initialized_state(&maker, &shape);
    maker.add_face(&face);
    maker.add_face(&face);
    assert_eq!(maker.closing_faces().len(), 1);

    maker.clear();
    assert_cleared_state(&maker, &shape);
}

#[test]
fn standard_offset_initialize_replaces_previous_closing_faces_and_options() {
    let (shape, face) = planar_input();
    let mut maker = BRepOffsetMakeOffset::new();
    initialize_arc(&mut maker, &shape);
    maker.add_face(&face);
    initialize_intersection(&mut maker, &shape);
    assert_reinitialized_state(&maker, &shape);
}

fn initialize_arc(maker: &mut BRepOffsetMakeOffset, shape: &rustycad_tkbrep::topods::Shape) {
    maker.initialize(
        shape,
        2.0,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Arc,
        false,
        false,
    );
}

fn initialize_intersection(
    maker: &mut BRepOffsetMakeOffset,
    shape: &rustycad_tkbrep::topods::Shape,
) {
    maker.initialize(
        shape,
        -3.0,
        2.0e-7,
        BRepOffsetMode::Pipe,
        true,
        true,
        JoinType::Intersection,
        true,
        true,
    );
}

fn planar_input() -> (
    rustycad_tkbrep::topods::Shape,
    rustycad_tkbrep::topods::Face,
) {
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (0.0, 1.0, 0.0, 1.0))
        .unwrap()
        .face()
        .unwrap();
    (face.clone().into_shape(), face)
}

fn assert_initialized_state(maker: &BRepOffsetMakeOffset, shape: &rustycad_tkbrep::topods::Shape) {
    assert!(maker.init_shape().is_same(shape));
    assert!(maker.shape().is_null());
    assert!(!maker.is_done());
    assert_eq!(maker.join_type().unwrap(), JoinType::Arc);
    assert_eq!(maker.linearization_allowed(), Some(true));
    assert_eq!(
        maker.configuration().unwrap(),
        BRepOffsetMakeOffsetConfiguration {
            offset: 2.0,
            tolerance: 1.0e-7,
            mode: BRepOffsetMode::Skin,
            intersection: false,
            self_intersection: false,
            join_type: JoinType::Arc,
            thickening: false,
            remove_internal_edges: false,
        }
    );
}

fn assert_cleared_state(maker: &BRepOffsetMakeOffset, shape: &rustycad_tkbrep::topods::Shape) {
    assert!(maker.init_shape().is_same(shape));
    assert!(maker.shape().is_null());
    assert!(!maker.is_done());
    assert_eq!(maker.join_type().unwrap(), JoinType::Arc);
    assert!(maker.closing_faces().is_empty());
}

fn assert_reinitialized_state(
    maker: &BRepOffsetMakeOffset,
    shape: &rustycad_tkbrep::topods::Shape,
) {
    assert!(maker.init_shape().is_same(shape));
    assert!(!maker.is_done());
    assert_eq!(maker.join_type().unwrap(), JoinType::Intersection);
    assert_eq!(maker.linearization_allowed(), Some(true));
    assert!(maker.closing_faces().is_empty());
    let config = maker.configuration().unwrap();
    assert_eq!(config.offset, -3.0);
    assert_eq!(config.tolerance, 2.0e-7);
    assert_eq!(config.mode, BRepOffsetMode::Pipe);
    assert!(config.intersection && config.self_intersection && config.thickening);
    assert!(config.remove_internal_edges);
}
