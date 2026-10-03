use super::*;

#[test]
fn constructor_modes_and_transfer_tool_match_native() {
    let expected = include_str!("../../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = ShapeFixComposeShell::new();
    let tool = composer.get_transfer_param_tool();
    assert_eq!(
        format!(
            "default {} {} {} {}",
            composer.closed_mode() as u8,
            composer.status(ShapeExtendStatus::Done1) as u8,
            composer.result().is_null() as u8,
            tool.dynamic_type().unwrap().name()
        ),
        expected.lines().next().unwrap()
    );
    composer.set_closed_mode(true);
    assert_eq!(
        format!("closed_mode {}", composer.closed_mode() as u8),
        expected
            .lines()
            .find(|row| row.starts_with("closed_mode "))
            .unwrap()
    );
    let replacement = Handle::new(ShapeAnalysisTransferParameters::new()).cast();
    composer.set_transfer_param_tool(&replacement);
    assert_eq!(
        format!(
            "transfer_type {}",
            composer
                .get_transfer_param_tool()
                .dynamic_type()
                .unwrap()
                .name()
        ),
        expected
            .lines()
            .find(|row| row.starts_with("transfer_type "))
            .unwrap()
    );
    assert_init_matches_native(expected);
}

fn assert_init_matches_native(expected: &str) {
    let face = face_fixture();
    let grid = plane_grid();
    let composer = ShapeFixComposeShell::new();
    composer
        .init(&grid, &Location::default(), &face, 1.0e-7)
        .unwrap();
    assert_eq!(
        format!(
            "init {} {} {}",
            composer.closed_mode() as u8,
            composer.status(ShapeExtendStatus::Done1) as u8,
            composer.result().is_null() as u8,
        ),
        expected
            .lines()
            .find(|row| row.starts_with("init "))
            .unwrap()
    );
}
