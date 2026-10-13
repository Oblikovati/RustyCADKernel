use super::*;
use crate::{
    RenderLayer,
    stencil_program::{StencilCommand, StencilProgram},
};
fn triangle(blend: bool) -> crate::PrimitiveDraw {
    let mut draw = crate::PrimitiveDraw::new(
        vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1, 2],
        crate::identity_clip(),
    )
    .unwrap();
    draw.surface_blend = blend;
    draw
}
#[test]
fn state_only_and_trailing_events_survive_opaque_and_deferred_layer_sorting() {
    let mut program = StencilProgram::default();
    program.command(StencilCommand::Options(true));
    program.geometry(vec![triangle(true)]);
    program.command(StencilCommand::Options(false));
    program.geometry(vec![triangle(false)]);
    program.command(StencilCommand::Options(true));
    let layer = RenderLayer::from_stencil_program(program, true, true, true);
    let frame = RenderFrame::with_layers(2, 2, [0.; 4], vec![layer]).unwrap();
    let plan = StencilPlan::new(&frame, Default::default());
    assert_eq!(
        plan.during
            .iter()
            .map(|state| state.is_enabled())
            .collect::<Vec<_>>(),
        [false, true]
    );
    assert!(plan.ending.is_enabled());
    assert_eq!(plan.ending.native_comparison(), (0x205, 1, 255));
}
#[test]
fn empty_layers_execute_commands_and_cap_restore_changes_inherited_context() {
    let mut program = StencilProgram::default();
    program.command(StencilCommand::Options(true));
    program.command(StencilCommand::CappingRestore);
    let frame = RenderFrame::with_layers(
        2,
        2,
        [0.; 4],
        vec![RenderLayer::from_stencil_program(program, true, true, true)],
    )
    .unwrap();
    let plan = StencilPlan::new(&frame, Default::default());
    assert!(plan.during.is_empty());
    assert!(!plan.ending.is_enabled());
    assert_eq!(plan.ending.native_comparison(), (0x207, 0, 255));
    assert_eq!(plan.after_layer, [plan.ending]);
    assert_eq!(plan.clear_after_layer, [true]);
}
