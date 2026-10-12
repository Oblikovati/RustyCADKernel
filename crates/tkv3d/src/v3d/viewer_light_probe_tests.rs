//! Pinned native replay bounds each case so a callback-lock regression cannot hang its toolkit.
use super::viewer_light_probe_replay::capture_light_case;
#[test]
fn real_activation_live_lists_callback_replacement_and_failures_replay_native() {
    let rows: Vec<String> = (0..=14)
        .filter(|policy| *policy != 12)
        .flat_map(bounded_light_case)
        .collect();
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/viewer-live-lights-native.txt")
    );
}
fn bounded_light_case(policy: usize) -> Vec<String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || sender.send(capture_light_case(policy)).unwrap());
    let result=receiver.recv_timeout(std::time::Duration::from_secs(3)).unwrap_or_else(|error| {
        panic!("Native light case={policy} completion={error}; expected completion within 3 seconds")
    });
    worker.join().unwrap();
    result
}
