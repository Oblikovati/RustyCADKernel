//! Native differential checks for protected `BOPAlgo_PaveFiller::Init`.
use rustycad_tkbo::bop_algo::{
    AlertNullInputShapes, AlertTooFewArguments, BOPAlgoPaveFiller, Glue,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{message::Alert, ncollection::List};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn initialize_matches_native_ds_iterator_and_locked_root_behavior() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&arguments());
    filler.initialize().unwrap();
    let ds = filler.data_structure().unwrap();
    let iterator = filler.iterator().unwrap();
    let expected = expected();
    assert_eq!(
        format!(
            "valid 0 {} {} 8 15-59 0 {}",
            ds.nb_source_shapes(),
            ds.nb_ranges(),
            filler.context().is_none() as i32
        ),
        expected[0]
    );
    assert!(!filler.options().has_errors());
    assert!(filler.context().is_some());
    assert!(iterator.run_parallel() == filler.options().run_parallel());

    let locked = box_shape(Point3::default());
    locked.set_locked(true).unwrap();
    filler.set_arguments(&List::from_iter([locked]));
    filler.initialize().unwrap();
    assert_eq!(filler.non_destructive(), expected[3].ends_with('1'));
}

#[test]
fn initialization_alerts_for_empty_and_null_arguments_without_clearing_state() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.initialize().unwrap();
    let too_few = AlertTooFewArguments;
    assert_eq!(
        format!(
            "empty {} {}",
            filler.options().has_errors() as i32,
            filler.data_structure().is_none() as i32
        ),
        expected()[1]
    );
    assert!(filler.options().has_error(too_few.dynamic_type()));
    assert!(filler.data_structure().is_none());

    filler.set_arguments(&List::from_iter([Shape::default()]));
    filler.initialize().unwrap();
    let null_input = AlertNullInputShapes;
    assert_eq!(
        format!(
            "null {} {}",
            filler.options().has_errors() as i32,
            filler.data_structure().is_none() as i32
        ),
        expected()[2]
    );
    assert!(filler.options().has_error(null_input.dynamic_type()));
    assert!(filler.data_structure().is_none());
}

#[test]
fn invalid_reinitialization_keeps_the_previous_native_preparation() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&arguments());
    filler.initialize().unwrap();
    filler.set_arguments(&List::from_iter([Shape::default()]));
    filler.initialize().unwrap();
    let source_shapes = filler.data_structure().unwrap().nb_source_shapes();
    assert!(filler.options().has_errors());
    assert_eq!(
        format!(
            "stale {} {source_shapes}",
            filler.options().has_errors() as i32
        ),
        expected()[6]
    );
}

#[test]
fn initialization_uses_context_when_obb_filtering_is_enabled() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&arguments());
    filler.options_mut().set_use_obb(true);
    filler.initialize().unwrap();
    assert!(filler.context().is_some());
    let iterator = filler.iterator_mut().unwrap();
    iterator.initialize(ShapeEnum::Vertex, ShapeEnum::Face);
    let (first, second) = iterator.value().unwrap();
    assert_eq!(
        format!("obb_vf {} {first}-{second}", iterator.expected_length()),
        expected()[5]
    );
}

#[test]
fn clear_retains_pave_filler_configuration_and_arguments() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(&arguments());
    filler.set_non_destructive(true);
    filler.set_glue(Glue::Shift);
    filler.set_avoid_build_pcurve(true);
    filler.set_is_primary(false);
    filler.options_mut().set_run_parallel(true);
    filler.options_mut().set_fuzzy_value(0.25);
    filler.options_mut().set_use_obb(true);
    filler.clear();
    let actual = format!(
        "config {} {} {} {} {} {} {} {}",
        filler.arguments().len(),
        filler.non_destructive() as i32,
        filler.glue() as i32,
        filler.is_avoid_build_pcurve() as i32,
        filler.is_primary() as i32,
        filler.options().run_parallel() as i32,
        filler.options().fuzzy_value(),
        filler.options().use_obb() as i32
    );
    assert_eq!(actual, expected()[7]);
    assert!(filler.data_structure().is_none());
    assert!(filler.iterator().is_none());
}

fn arguments() -> List<Shape> {
    List::from_iter([
        box_shape(Point3::default()),
        box_shape(Point3::new(0.5, 0.0, 0.0)),
    ])
}

fn box_shape(point: Point3) -> Shape {
    let mut maker = BRepPrimAPIMakeBox::at_point(point, 1.0, 1.0, 1.0).unwrap();
    maker.shape().unwrap().clone()
}

fn expected() -> Vec<&'static str> {
    include_str!("../../../tools/oracle/tkbo_pave_filler_init.expected")
        .lines()
        .collect()
}
