//! Native animation callback and frozen timer token streams.
use super::common::{Reader, end, flag, int, real, text};
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
#[path = "../../tests/support/animation_recorder.rs"]
mod recorder;
use recorder::{RecordedAnimationFrame, RecordingAnimation};
fn frames(values: &Arc<SharedState<Vec<RecordedAnimationFrame>>>) {
    for value in values.write().drain(..) {
        text(value.name.as_bytes());
        real(value.progress.pts);
        real(value.progress.local_pts);
        real(value.progress.local_normalized);
        flag(value.stopped);
    }
}
pub fn animation_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let values = Arc::new(SharedState::new(Vec::new()));
    let root = Handle::new(RecordingAnimation::new("root", values.clone()));
    let child = Handle::new(RecordingAnimation::new("child", values.clone()));
    root.set_start_pts(reader.real());
    root.set_own_duration(reader.real());
    child.set_start_pts(reader.real());
    child.set_own_duration(reader.real());
    root.add(&child.cast()).unwrap();
    root.add(&child.cast()).unwrap();
    int(root.children().length());
    real(root.duration());
    root.start(false).unwrap();
    for _ in 0..3 {
        let running = root.update(reader.real()).unwrap();
        frames(&values);
        flag(running);
    }
    root.pause().unwrap();
    flag(root.is_stopped());
    flag(child.is_stopped());
    root.clear();
    real(root.duration());
    root.update_total_duration().unwrap();
    real(root.duration());
    end();
}
pub fn timer_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let values = Arc::new(SharedState::new(Vec::new()));
    let root = RecordingAnimation::new("timer", values.clone());
    root.set_own_duration(reader.real());
    let pts = reader.real();
    let speed = reader.real();
    root.start_timer(pts, speed, true, true).unwrap();
    frames(&values);
    real(root.elapsed_time());
    real(root.timer().playback_speed());
    flag(root.timer().is_started());
    flag(root.is_stopped());
    let elapsed = root.update_timer().unwrap();
    frames(&values);
    real(elapsed);
    root.stop().unwrap();
    real(root.elapsed_time());
    end();
}
pub fn edits_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let root = Handle::new(AisAnimation::new("root"));
    let first = Handle::new(AisAnimation::new("same"));
    let second = Handle::new(AisAnimation::new("same"));
    first.set_own_duration(reader.real());
    second.set_own_duration(reader.real());
    root.add(&first.cast()).unwrap();
    root.add(&second.cast()).unwrap();
    flag(root.find(&"same".into()).unwrap() == first.cast::<dyn AisAnimationSource>());
    flag(root.find(&"absent".into()).unwrap().is_null());
    flag(root.replace(&first.cast(), &second.cast()).unwrap());
    int(root.children().length());
    flag(root.remove(&second.cast()).unwrap());
    real(root.duration());
    let copy = Handle::new(AisAnimation::new("copy"));
    copy.copy_from(&root.cast()).unwrap();
    text(copy.name().as_bytes());
    real(copy.duration());
    copy.copy_from(&copy.cast()).unwrap();
    int(copy.children().length());
    real(copy.duration());
    super::common::attempt(|| root.add(&Handle::default()));
    end();
}
