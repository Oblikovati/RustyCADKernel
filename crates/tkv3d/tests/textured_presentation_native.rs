//! Rust-only replay of actual pinned native presentation callbacks.
#[path = "support/textured_compute_replay.rs"]
mod compute_replay;
#[path = "support/textured_wireframe_replay.rs"]
mod wireframe_replay;

#[test]
fn textured_compute_replays_native_modes_and_mapping_flags() {
    for scenario in 0..8 {
        for mode in 0..5 {
            compute_replay::replay_textured_compute_case(scenario, mode);
        }
    }
}

#[test]
fn textured_wireframe_replays_native_isoline_aspect_routes() {
    for scenario in 0..16 {
        wireframe_replay::replay_textured_wireframe_case(scenario);
    }
}
