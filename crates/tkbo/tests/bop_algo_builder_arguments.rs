//! Argument and tool identity behavior for the Boolean builder foundation.
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoToolsProvider, Glue};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;

#[test]
fn builder_argument_fence_and_clear_match_native() {
    let shape = vertex_shape();
    let reversed = shape.reversed();
    let mut builder = BOPAlgoBuilder::new();
    builder.add_argument(&shape);
    builder.add_argument(&reversed);
    assert_eq!(
        builder.arguments().len(),
        expected("builder_add")[0] as usize
    );
    let arguments = List::from_iter([shape.clone()]);
    builder.set_arguments(&arguments);
    assert_eq!(
        builder.arguments().len(),
        expected("builder_set_after_add")[0] as usize
    );
    builder.clear();
    builder.set_arguments(&arguments);
    assert_eq!(
        builder.arguments().len(),
        expected("builder_set_after_clear")[0] as usize
    );
    assert!(builder.shape().is_null());
}

#[test]
fn builder_clear_preserves_options_and_boolean_modes() {
    let mut builder = BOPAlgoBuilder::new();
    builder.set_non_destructive(true);
    builder.set_glue(Glue::Shift);
    builder.set_check_inverted(false);
    builder.set_to_fill_history(false);
    builder.options_mut().set_fuzzy_value(0.25);
    builder.clear();
    let clear = expected("builder_clear");
    assert_eq!(builder.arguments().len(), clear[0] as usize);
    assert_eq!(f64::from(i32::from(builder.non_destructive())), clear[1]);
    assert_eq!(builder.glue() as i32 as f64, clear[2]);
    assert_eq!(f64::from(i32::from(builder.check_inverted())), clear[3]);
    assert_eq!(f64::from(i32::from(builder.has_history())), clear[4]);
    assert_eq!(builder.options().fuzzy_value(), clear[5]);
}

#[test]
fn tools_provider_keeps_native_stale_tool_fence_until_clear() {
    let shape = vertex_shape();
    let reversed = shape.reversed();
    let mut provider = BOPAlgoToolsProvider::new();
    provider.add_tool(&shape);
    provider.add_tool(&reversed);
    assert_eq!(provider.tools().len(), expected("tools_add")[0] as usize);
    let tools = List::from_iter([shape]);
    provider.set_tools(&tools);
    assert_eq!(provider.tools().len(), expected("tools_set")[0] as usize);
    provider.clear();
    let clear = expected("tools_clear");
    assert_eq!(provider.tools().len(), clear[0] as usize);
    assert_eq!(provider.arguments().len(), clear[1] as usize);
    provider.set_tools(&tools);
    assert_eq!(
        provider.tools().len(),
        expected("tools_set_after_clear")[0] as usize
    );
}

fn vertex_shape() -> Shape {
    BRepBuilderAPIMakeVertex::new(Point3::default())
        .unwrap()
        .vertex()
        .unwrap()
        .into_shape()
}

fn expected(label: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkbo_builder_argument_state.expected")
        .lines()
        .find(|line| line.starts_with(label))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}
