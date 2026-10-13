use super::*;
#[test]
fn captured_and_adjacent_view_contexts_do_not_follow_a_completed_frame() {
    let first = StencilFeedback::default();
    let second = StencilFeedback::default();
    let mut frame = RenderFrame::new(2, 2, [0.; 4], vec![]).unwrap();
    frame.attach_stencil_context(&first);
    let ending = RenderStencilState {
        enabled: true,
        ..Default::default()
    };
    frame.stencil_context.as_ref().unwrap().commit(ending);
    assert!(!frame.stencil_state().is_enabled());
    assert!(frame.stencil_draw_states().is_empty());
    frame.attach_stencil_context(&first);
    assert!(frame.stencil_state().is_enabled());
    frame.attach_stencil_context(&second);
    assert!(!frame.stencil_state().is_enabled());
}
#[test]
fn standalone_context_fallback_is_preserved_without_a_view_sink() {
    let frame = RenderFrame::new(2, 2, [0.; 4], vec![]).unwrap();
    let ending = RenderStencilState {
        enabled: true,
        ..Default::default()
    };
    assert_eq!(frame.initial_stencil(ending), ending);
    assert!(frame.stencil_context.is_none());
}

#[test]
fn controlled_clear_input_does_not_mutate_a_captured_frame_or_native_context() {
    let original = RenderFrame::new(2, 2, [0.; 4], vec![]).unwrap();
    let seeded = original.clone().with_stencil_clear_value(1);
    assert_eq!(original.stencil_clear_value, 0);
    assert_eq!(seeded.stencil_clear_value, 1);
    assert_eq!(seeded.stencil_state(), original.stencil_state());
}
