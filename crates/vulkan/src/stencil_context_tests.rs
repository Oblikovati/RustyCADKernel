use super::*;
#[test]
fn enable_disable_and_cap_restore_preserve_unrelated_operations_and_write_mask() {
    let mut state = RenderStencilState {
        write_mask: 0x5A,
        operations: [StencilOperation::Invert; 3],
        ..Default::default()
    };
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
    assert_eq!(
        state.operations,
        [
            StencilOperation::Keep,
            StencilOperation::Invert,
            StencilOperation::Invert
        ]
    );
    state.capping(RenderCappingPass::Surface);
    assert_eq!(state.native_comparison(), (0x202, 1, 1));
    assert_eq!(state.operations, [StencilOperation::Keep; 3]);
    state.capping(RenderCappingPass::None);
    assert!(state.is_enabled());
    assert_eq!(state.write_mask, 255);
}
