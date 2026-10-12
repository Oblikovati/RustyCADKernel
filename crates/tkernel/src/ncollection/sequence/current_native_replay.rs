//! Rust-only replay uses real stable nodes, saved-next clearing and actual allocator callbacks.
#![allow(unsafe_code)]
use super::current_native_fixture::{NativeSequenceScene, create_sequence_resources};
use crate::ncollection::SequencePreparedNode;
use std::sync::Arc;
fn append_real_native_node(scene: &NativeSequenceScene) {
    let lease = scene.sequence.read().allocator().storage_lease();
    let node = SequencePreparedNode::new(&lease, || 19);
    scene.sequence.write().append_prepared(node);
}
fn clear_real_native_node(scene: &NativeSequenceScene) {
    let mut progress = scene.sequence.write().begin_clear_progress();
    while visit_native_node(scene, &mut progress) {
        retire_native_node(scene, &mut progress);
    }
    scene
        .sequence
        .write()
        .finish_clear_progress(progress)
        .unwrap();
}
fn visit_native_node(
    scene: &NativeSequenceScene,
    progress: &mut super::SequenceClearProgress,
) -> bool {
    // Capturing Next occurs before callbacks, while actual node metadata remains native.
    scene
        .sequence
        .read()
        .visit_clear_node(progress)
        .unwrap()
        .is_some()
}
fn retire_native_node(scene: &NativeSequenceScene, progress: &mut super::SequenceClearProgress) {
    let node = scene.sequence.write().detach_clear_node(progress).unwrap();
    // SAFETY: native fixture resources use the same compatible Standard heap backend.
    unsafe { node.retire_using(|| scene.sequence.read().allocator().storage_lease()) };
}
fn record_scene_endpoint(scene: &NativeSequenceScene, phase: &str) {
    let row = format!(
        "{phase} {} {}",
        scene.sequence.read().length(),
        i32::from(*scene.retired.read())
    );
    scene.rows.write().push(row);
}
#[test]
fn actual_sequence_nodes_replay_native_owner_loss_and_current_free_metadata() {
    let scene = Arc::new(NativeSequenceScene::default());
    let (_current, original) = create_sequence_resources(&scene);
    append_real_native_node(&scene);
    assert_eq!(scene.sequence.read().first(), Some(&19));
    drop(original);
    record_scene_endpoint(&scene, "after-owner-loss");
    clear_real_native_node(&scene);
    record_scene_endpoint(&scene, "cleared");
    let actual = scene.rows.read().join("\n") + "\n";
    assert_eq!(
        actual,
        include_str!("../../../tests/fixtures/allocator-retirement-native.txt")
    );
}
