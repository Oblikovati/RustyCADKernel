//! Native volume grouping and triangulation/closure policy use actual shared BRep geometry.
#[path = "support/textured_volume_replay.rs"]
mod replay;

#[test]
fn textured_shaded_volume_groups_preserve_native_location_orientation_and_closure() {
    for scenario in 0..10 {
        replay::replay_textured_volume_case(scenario, false);
        replay::replay_textured_volume_case(scenario, true);
    }
}
