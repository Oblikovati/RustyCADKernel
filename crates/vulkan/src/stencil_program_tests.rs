use super::*;
fn triangle() -> PrimitiveDraw {
    PrimitiveDraw::new(
        vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
        vec![0, 1, 2],
        crate::identity_clip(),
    )
    .unwrap()
}
#[test]
fn empty_and_trailing_commands_append_without_manufacturing_geometry() {
    let mut first = StencilProgram::default();
    first.geometry(vec![triangle()]);
    first.command(StencilCommand::Options(true));
    let mut second = StencilProgram::default();
    second.command(StencilCommand::Options(false));
    first.append(second);
    assert_eq!(first.draws.len(), 1);
    assert_eq!(
        first
            .boundaries
            .iter()
            .map(|event| event.before_draw)
            .collect::<Vec<_>>(),
        [1, 1]
    );
    assert_eq!(first.boundaries[1].command, StencilCommand::Options(false));
}
#[test]
fn native_fill_filter_rejects_options_but_keeps_cap_restoration() {
    let program = native_fill_filter_program();
    let geometry = [true, false, true];
    let opaque = filtered_boundaries(&program, &geometry, false);
    assert_eq!(
        opaque
            .iter()
            .map(|event| event.before_draw)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let transparent = filtered_boundaries(&program, &geometry, true);
    assert_eq!(
        transparent,
        [StencilBoundary {
            before_draw: 2,
            command: StencilCommand::CappingRestore
        }]
    );
}

#[test]
fn low_level_cap_program_restores_at_actual_ordinary_boundaries_and_layer_end() {
    let ordinary = triangle();
    let mut cap = triangle();
    cap.capping_pass = crate::RenderCappingPass::Surface;
    let boundaries = implicit_cap_restores(&[cap.clone(), ordinary, cap]);
    assert_eq!(
        boundaries
            .iter()
            .map(|event| event.before_draw)
            .collect::<Vec<_>>(),
        [1, 3]
    );
    assert!(
        boundaries
            .iter()
            .all(|event| event.command == StencilCommand::CappingRestore)
    );
}
#[test]
fn valid_text_appends_reset_after_glyphs_but_empty_text_and_primitives_do_not() {
    let mut program = StencilProgram::default();
    program.ordered_geometry(vec![], true);
    program.ordered_geometry(vec![triangle()], false);
    assert!(program.boundaries.is_empty());
    program.ordered_geometry(vec![triangle()], true);
    assert_eq!(
        program.boundaries,
        [StencilBoundary {
            before_draw: 2,
            command: StencilCommand::TextReset,
        }]
    );
    assert!(filtered_boundaries(&program.boundaries, &[false, false], true).is_empty());
    assert_eq!(
        filtered_boundaries(&program.boundaries, &[false, false], false),
        program.boundaries
    );
}

fn native_fill_filter_program() -> [StencilBoundary; 2] {
    [
        StencilBoundary {
            before_draw: 1,
            command: StencilCommand::Options(true),
        },
        StencilBoundary {
            before_draw: 3,
            command: StencilCommand::CappingRestore,
        },
    ]
}
