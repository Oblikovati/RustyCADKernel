use super::{allocation_trace::*, native_mode};
use rustycad_tkernel::ncollection::{
    BaseAllocator, BoundingTreeInsertion, BoundingVolume, UBTree, UBTreeFiller,
};
use std::process::Command;
struct Dropping<const KIND: char>(Payload<KIND>);
impl<const KIND: char> Default for Dropping<KIND> {
    fn default() -> Self {
        Self(Payload::default())
    }
}
impl<const KIND: char> Clone for Dropping<KIND> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}
impl<const KIND: char> Drop for Dropping<KIND> {
    fn drop(&mut self) {
        eprintln!("payload_drop {KIND}");
    }
}
impl BoundingVolume for Dropping<'B'> {
    fn add(&mut self, other: &Self) {
        self.0.add(&other.0);
    }
    fn is_out(&self, other: &Self) -> bool {
        self.0.is_out(&other.0)
    }
    fn square_extent(&self) -> f64 {
        self.0.square_extent()
    }
}
type ChildTree = UBTree<Dropping<'O'>, Dropping<'B'>>;
struct RecordingInsertion(ChildTree);
impl BoundingTreeInsertion<Dropping<'O'>, Dropping<'B'>> for RecordingInsertion {
    fn insert_bound(&mut self, object: Dropping<'O'>, bounds: Dropping<'B'>) -> bool {
        self.insert_bound_ref(&object, &bounds)
    }
    fn insert_bound_ref(&mut self, object: &Dropping<'O'>, bounds: &Dropping<'B'>) -> bool {
        eprintln!("insertion");
        self.0.add_ref(object, bounds)
    }
    fn tree_view(&self) -> &ChildTree {
        &self.0
    }
}
struct PolicyCapture;
impl PolicyCapture {
    fn observe(&self) {
        eprintln!("policy");
    }
}
impl Drop for PolicyCapture {
    fn drop(&mut self) {
        eprintln!("capture_drop");
    }
}
fn install_child_hook() {
    std::panic::set_hook(Box::new(|info| {
        if info.payload().downcast_ref::<&str>() == Some(&"outer rejection") {
            eprintln!("outer_unwind");
            return;
        }
        eprintln!("terminate {}", finish());
    }));
}
#[test]
fn child_drop_failure_probe() {
    let Ok(mode) = std::env::var("RUSTYCAD_FILLER_DROP_PROBE") else {
        return;
    };
    run_child_drop(&mode);
}
fn run_child_drop(mode: &str) {
    install_child_hook();
    let arena = BaseAllocator::accumulating(65536).unwrap();
    let mut target = RecordingInsertion(ChildTree::with_allocator(arena.clone()));
    let capture = PolicyCapture;
    let mut filler =
        UBTreeFiller::new(&mut target, None, true).with_self_assignment_policy(move |pair| {
            capture.observe();
            pair.assign_self_with(
                |object| object.0.assign_self(),
                |bounds| bounds.0.assign_self(),
            );
        });
    filler.add(Dropping(Payload(1)), Dropping(Payload(10)));
    start(&arena, ('A', mode.chars().next().unwrap(), 1));
    if mode.ends_with('u') {
        panic!("outer rejection");
    }
    drop(filler);
}
fn child_output(mode: &str) -> std::process::Output {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "termination::child_drop_failure_probe",
            "--nocapture",
        ])
        .env("RUSTYCAD_FILLER_DROP_PROBE", mode)
        .output()
        .unwrap()
}
fn assert_aborted(mode: &str) {
    let output = child_output(mode);
    let stderr = String::from_utf8(output.stderr).unwrap();
    let native = native_mode(mode);
    let prefix = native[0].strip_suffix(" 1 1").unwrap();
    assert!(stderr.lines().any(|line| line == prefix), "{stderr}");
    assert_eq!(stderr.matches("insertion\n").count(), 1, "{stderr}");
    assert_eq!(stderr.matches("policy\n").count(), 1, "{stderr}");
    assert!(!stderr.contains("capture_drop"), "{stderr}");
    assert!(!stderr.contains("payload_drop"), "{stderr}");
    assert!(!output.status.success());
    #[cfg(unix)]
    assert_eq!(
        std::os::unix::process::ExitStatusExt::signal(&output.status),
        Some(6)
    );
}
#[test]
fn automatic_failure_aborts_before_inner_replay_or_capture_cleanup() {
    for mode in ["Od", "Bd", "Ou", "Bu"] {
        assert_aborted(mode);
    }
}
