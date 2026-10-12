//! All successful pinned last-owner callbacks run against actual nodes and object counters.
use super::clip_sequence_retirement_fixture::{
    ClipRetirementAppend, ClipRetirementTranscript, append_retiring_plane,
};
use super::{
    Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane, Graphic3dSequenceOfHClipPlaneIterator,
};
use rustycad_tkernel::standard::{Handle, Transient};
use std::sync::Arc;
fn append_retirement_members(
    collection: &Arc<Graphic3dSequenceOfHClipPlane>,
    transcript: &Arc<ClipRetirementTranscript>,
    phase: &'static str,
    count: i32,
) {
    assert_eq!(collection.transient_base().ref_count(), 0);
    for ordinal in 1..=count {
        append_retiring_plane(collection, transcript, phase, ordinal);
    }
}
fn replay_clear_order(transcript: &Arc<ClipRetirementTranscript>) {
    let collection = Arc::new(Graphic3dSequenceOfHClipPlane::default());
    append_retirement_members(&collection, transcript, "clear", 3);
    collection.clear();
    transcript.rows.write().push(format!(
        "clear-end {} {}",
        collection.size(),
        i32::from(collection.is_empty())
    ));
}
fn replay_remove_order(transcript: &Arc<ClipRetirementTranscript>) {
    let collection = Arc::new(Graphic3dSequenceOfHClipPlane::default());
    append_retirement_members(&collection, transcript, "remove", 2);
    let mut cursor = Graphic3dSequenceOfHClipPlaneIterator::new(&collection);
    collection.remove_at(&mut cursor).unwrap();
    transcript.rows.write().push(format!(
        "remove-end {} {}",
        collection.size(),
        i32::from(cursor.more())
    ));
    collection.clear();
}
fn configure_retirement_append(
    transcript: &ClipRetirementTranscript,
    appended: &Handle<Graphic3dClipPlane>,
    ordinal: i32,
    direct: bool,
) {
    *transcript.append.write() = Some(ClipRetirementAppend {
        ordinal,
        plane: appended.downgrade(),
        direct,
    });
}
fn replay_append_order(transcript: &Arc<ClipRetirementTranscript>, direct: bool) {
    let collection = Arc::new(Graphic3dSequenceOfHClipPlane::default());
    let appended = Handle::new(Graphic3dClipPlane::default());
    let (phase, ordinal) = if direct {
        ("protected-append", 2)
    } else {
        ("append-clear", 1)
    };
    configure_retirement_append(transcript, &appended, ordinal, direct);
    append_retirement_members(&collection, transcript, phase, 2);
    collection.clear();
    *transcript.append.write() = None;
    record_append_end(transcript, &collection, &appended, direct);
}
fn record_append_end(
    transcript: &ClipRetirementTranscript,
    collection: &Graphic3dSequenceOfHClipPlane,
    appended: &Handle<Graphic3dClipPlane>,
    direct: bool,
) {
    let phase = if direct {
        "protected-end"
    } else {
        "append-end 1"
    };
    let row = format!("{phase} {} {}", collection.size(), appended.ref_count());
    transcript.rows.write().push(row);
}
#[test]
fn actual_collection_destruction_replays_all_thirteen_valid_native_retirement_records() {
    let transcript = Arc::new(ClipRetirementTranscript::default());
    replay_clear_order(&transcript);
    replay_remove_order(&transcript);
    replay_append_order(&transcript, false);
    replay_append_order(&transcript, true);
    assert_eq!(
        transcript.rows.read().join("\n") + "\n",
        include_str!("../../tests/fixtures/clip-collection-retirement-native.txt")
    );
}
