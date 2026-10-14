use super::*;
#[test]
fn enable_disable_and_cap_restore_preserve_unrelated_operations_and_write_mask() {
    let mut state = unusual_stencil_state();
    state.command(StencilCommand::Options(true));
    assert!(state.is_enabled());
    assert_eq!(state.native_comparison(), (0x205, 1, 255));
    state.command(StencilCommand::Options(false));
    assert!(!state.is_enabled());
    assert_eq!(state.native_comparison(), (0x205, 1, 255));
    state.command(StencilCommand::CappingRestore);
    assert_eq!(state.native_comparison(), (0x207, 0, 255));
    assert_eq!(state.write_mask, 0x5A);
    assert_eq!(state.operations, [StencilOperation::Invert; 3]);
    assert_eq!(
        state.native_state_fields(),
        [0, 0x207, 0, 255, 0x5A, 0x150A, 0x150A, 0x150A]
    );
}
#[test]
fn native_mask_surface_and_restore_transitions_preserve_write_mask() {
    let mut state = RenderStencilState::default();
    assert!(!state.is_enabled());
    assert_eq!(state.native_comparison(), (0x207, 0, 255));
    state.capping(RenderCappingPass::MaskStart);
    assert_eq!(state.native_comparison(), (0x207, 1, 1));
    let expected = [
        StencilOperation::Keep,
        StencilOperation::Invert,
        StencilOperation::Invert,
    ];
    assert_eq!(state.operations, expected);
    state.capping(RenderCappingPass::Surface);
    assert_eq!(state.native_comparison(), (0x202, 1, 1));
    assert_eq!(state.operations, [StencilOperation::Keep; 3]);
    state.capping(RenderCappingPass::None);
    assert!(state.is_enabled());
    assert_eq!(state.write_mask, 255);
}
#[test]
fn native_text_reset_disables_only_and_preserves_inherited_masks_and_operations() {
    let mut state = unusual_stencil_state();
    state.command(StencilCommand::Options(true));
    let inherited = state;
    state.command(StencilCommand::TextReset);
    assert!(!state.enabled);
    assert_eq!(state.native_comparison(), inherited.native_comparison());
    assert_eq!(state.write_mask, inherited.write_mask);
    assert_eq!(state.operations, inherited.operations);
}

fn unusual_stencil_state() -> RenderStencilState {
    RenderStencilState {
        write_mask: 0x5A,
        operations: [StencilOperation::Invert; 3],
        ..Default::default()
    }
}
