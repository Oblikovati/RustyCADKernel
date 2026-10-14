//! Original87 accepted native cases replay through actual retained Rust groups, without OCCT.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "support/ordered_stencil_replay.rs"]
mod replay;

#[test]
fn all87_native_stencil_cases_replay_with_exact_color_state_and_original_depth_gate() {
    let mut count = 0;
    for case in replay::native_cases() {
        assert!(replay::replay_case(&case, device_options::validation_options(), false).is_empty());
        count += 1;
    }
    assert_eq!(count, 87);
}
