//! Argument and option state exposed by the upstream General Fuse API base.
use rustycad_tkbo::bop_algo::Glue;
use rustycad_tkbo::brep_algo_api::BRepAlgoAPIBuilderAlgo;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::Orientation;

#[test]
fn builder_defaults_match_occt() {
    let builder = BRepAlgoAPIBuilderAlgo::new();
    assert!(builder.arguments().is_empty());
    assert!(!builder.non_destructive());
    assert_eq!(builder.glue(), Glue::Off);
    assert!(builder.check_inverted());
    assert!(builder.has_history());
}

#[test]
fn setting_arguments_copies_the_list_nodes_and_preserves_order() {
    let mut arguments = List::default();
    let mut first = Shape::default();
    first.set_orientation(Orientation::Forward);
    let mut second = Shape::default();
    second.set_orientation(Orientation::Reversed);
    arguments.append(first);
    arguments.append(second);
    let mut builder = BRepAlgoAPIBuilderAlgo::new();
    builder.set_arguments(&arguments);
    arguments.clear();
    assert_eq!(builder.arguments().len(), 2);
    let orientations: Vec<Orientation> =
        builder.arguments().iter().map(Shape::orientation).collect();
    assert_eq!(orientations, [Orientation::Forward, Orientation::Reversed]);
}

#[test]
fn option_setters_roundtrip_their_configured_values() {
    let mut builder = BRepAlgoAPIBuilderAlgo::new();
    builder.set_non_destructive(true);
    builder.set_glue(Glue::Full);
    builder.set_check_inverted(false);
    builder.set_to_fill_history(false);
    assert!(builder.non_destructive());
    assert_eq!(builder.glue(), Glue::Full);
    assert!(!builder.check_inverted());
    assert!(!builder.has_history());
}
