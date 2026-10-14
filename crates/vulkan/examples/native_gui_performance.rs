//! Build this executable first; accepted timings require the coordinator's quiet-host GO.
// The shared fixture retains its viewer owner even when this timing probe does not inspect it.
#[allow(dead_code)]
#[path = "../tests/support/ais_scene.rs"]
mod ais_scene;
#[path = "support/device_options.rs"]
mod device_options;
// Mutation and native-byte observers are exercised by the separate acceptance harness.
#[allow(dead_code)]
#[path = "support/gui_acceptance_scene.rs"]
mod gui_acceptance_scene;
#[path = "../tests/support/gui_packets.rs"]
mod gui_packets;
#[path = "support/gui_performance.rs"]
mod gui_performance;
#[path = "support/native_window.rs"]
mod native_window;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[path = "support/window_scene.rs"]
mod window_scene;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let warmup = parse_gui_count(arguments.get(1), 20)?;
    let samples = parse_gui_count(arguments.get(2), 100)?;
    let policy = parse_gui_policy(arguments.get(3))?;
    gui_performance::benchmark_gui(warmup, samples, policy);
    Ok(())
}
fn parse_gui_policy(
    value: Option<&String>,
) -> Result<martensite_vulkan::RenderGuiSurfacePolicy, Box<dyn std::error::Error>> {
    use martensite_vulkan::RenderGuiSurfacePolicy::{CheckedV2, LegacyV1};
    match value.map(String::as_str) {
        None | Some("legacy") => Ok(LegacyV1),
        Some("checked") => Ok(CheckedV2),
        Some(other) => {
            Err(format!("GUI surface policy {other:?}; expected legacy or checked").into())
        }
    }
}
fn parse_gui_count(
    value: Option<&String>,
    fallback: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let Some(value) = value else {
        return Ok(fallback);
    };
    let count = value
        .parse::<usize>()
        .map_err(|error| format!("GUI count {value:?}: {error}; expected a positive integer"))?;
    if count == 0 {
        return Err(format!("GUI count {value:?}; expected a positive sample/warmup count").into());
    }
    Ok(count)
}
#[cfg(test)]
mod tests {
    #[test]
    fn gui_performance_policy_is_explicit_and_legacy_by_default() {
        use martensite_vulkan::RenderGuiSurfacePolicy::{CheckedV2, LegacyV1};
        assert_eq!(super::parse_gui_policy(None).unwrap(), LegacyV1);
        assert_eq!(
            super::parse_gui_policy(Some(&"legacy".into())).unwrap(),
            LegacyV1
        );
        assert_eq!(
            super::parse_gui_policy(Some(&"checked".into())).unwrap(),
            CheckedV2
        );
        assert!(super::parse_gui_policy(Some(&"unknown".into())).is_err());
    }
    #[test]
    fn gui_performance_counts_preserve_defaults_and_refuse_zero() {
        assert_eq!(super::parse_gui_count(None, 20).unwrap(), 20);
        assert_eq!(
            super::parse_gui_count(Some(&"100".into()), 20).unwrap(),
            100
        );
        assert!(super::parse_gui_count(Some(&"0".into()), 20).is_err());
        assert!(super::parse_gui_count(Some(&"broken".into()), 20).is_err());
    }
}
