//! Native child callbacks edit other nodes while DumpJson keeps a live traversal and no owners.
use super::{Graphic3dClipPlane, Graphic3dClipPlaneSource, Graphic3dSequenceOfHClipPlane};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, Transient, WeakHandle};
use std::{io, sync::Arc};
struct ClipDumpLiveEdit {
    remove: WeakHandle<dyn Graphic3dClipPlaneSource>,
    append: WeakHandle<dyn Graphic3dClipPlaneSource>,
}
struct ClipDumpLivePlaybackPlane {
    base: Graphic3dClipPlane,
    tag: i32,
    collection: WeakHandle<Graphic3dSequenceOfHClipPlane>,
    rows: Arc<SharedState<Vec<String>>>,
    edit: SharedState<Option<ClipDumpLiveEdit>>,
}
rustycad_tkernel::impl_transient!(ClipDumpLivePlaybackPlane,
    "Martensite_ClipDumpLivePlaybackPlane", Graphic3dClipPlane, base,
    interfaces: [dyn Graphic3dClipPlaneSource]);
impl Graphic3dClipPlaneSource for ClipDumpLivePlaybackPlane {
    fn clip_plane_base(&self) -> &Graphic3dClipPlane {
        &self.base
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let sequence = self.collection.observe();
        self.rows.write().push(format!(
            "visit {} {depth} {} {}",
            self.tag,
            sequence.size(),
            self.transient_base().ref_count()
        ));
        let edit = self.edit.write().take();
        if let Some(edit) = edit {
            sequence.remove_reference(&*edit.remove.observe());
            sequence.append_reference(&*edit.append.observe());
        }
        stream.append(&format!("\"tag\": {}", self.tag));
        Ok(())
    }
}
fn live_dump_plane(
    sequence: &Handle<Graphic3dSequenceOfHClipPlane>,
    rows: &Arc<SharedState<Vec<String>>>,
    tag: i32,
) -> Handle<ClipDumpLivePlaybackPlane> {
    Handle::new(ClipDumpLivePlaybackPlane {
        base: Graphic3dClipPlane::default(),
        tag,
        collection: sequence.downgrade(),
        rows: rows.clone(),
        edit: SharedState::default(),
    })
}
#[test]
fn null_dump_children_and_depth_zero_replay_native_json_without_projection() {
    let sequence = Graphic3dSequenceOfHClipPlane::default();
    sequence.append_reference(&Handle::<dyn Graphic3dClipPlaneSource>::default());
    let rows: Vec<String> = [-1, 0]
        .into_iter()
        .map(|depth| {
            let mut stream = DumpStream::default();
            sequence.dump_json(&mut stream, depth).unwrap();
            format!("null {depth} {} {}", sequence.size(), stream.text())
        })
        .collect();
    let expected: Vec<&str> = include_str!("../../tests/fixtures/clip-dump-live-native.txt")
        .lines()
        .take(2)
        .collect();
    assert_eq!(rows, expected);
}
struct ClipDumpLiveActors {
    sequence: Handle<Graphic3dSequenceOfHClipPlane>,
    rows: Arc<SharedState<Vec<String>>>,
    first: Handle<ClipDumpLivePlaybackPlane>,
    second: Handle<dyn Graphic3dClipPlaneSource>,
    third: Handle<dyn Graphic3dClipPlaneSource>,
}
fn live_dump_actors() -> ClipDumpLiveActors {
    let sequence = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    let rows = Arc::new(SharedState::default());
    let first = live_dump_plane(&sequence, &rows, 1);
    let second = live_dump_plane(&sequence, &rows, 2).into_cast();
    let third = live_dump_plane(&sequence, &rows, 3).into_cast();
    ClipDumpLiveActors {
        sequence,
        rows,
        first,
        second,
        third,
    }
}
fn prepare_live_dump(actors: &ClipDumpLiveActors) {
    *actors.first.edit.write() = Some(ClipDumpLiveEdit {
        remove: actors.second.downgrade(),
        append: actors.third.downgrade(),
    });
    actors
        .sequence
        .append_reference(&*actors.first.observe_as::<dyn Graphic3dClipPlaneSource>());
    actors.sequence.append_reference(&actors.second);
    actors.sequence.set_override_global(true);
}
fn live_dump_count_row(actors: &ClipDumpLiveActors) -> String {
    format!(
        "counts {} {} {} {}",
        actors.sequence.ref_count(),
        actors.first.ref_count(),
        actors.second.ref_count(),
        actors.third.ref_count()
    )
}
#[test]
fn virtual_dump_edits_follow_real_next_nodes_and_preserve_native_counts() {
    let actors = live_dump_actors();
    prepare_live_dump(&actors);
    let mut stream = DumpStream::default();
    actors.sequence.dump_json(&mut stream, -1).unwrap();
    actors.rows.write().push(format!("json {}", stream.text()));
    actors.rows.write().push(live_dump_count_row(&actors));
    let expected: Vec<&str> = include_str!("../../tests/fixtures/clip-dump-live-native.txt")
        .lines()
        .skip(2)
        .collect();
    assert_eq!(*actors.rows.read(), expected);
}
