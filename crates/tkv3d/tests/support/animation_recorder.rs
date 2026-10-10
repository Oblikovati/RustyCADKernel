//! Named animation atoms exercise virtual calls and callback edits without held locks.
#![allow(dead_code)]
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkv3d::ais::*;
use std::sync::Arc;
#[derive(Clone, Debug, PartialEq)]
pub struct RecordedAnimationFrame {
    pub name: String,
    pub progress: AisAnimationProgress,
    pub stopped: bool,
}
pub struct RecordingAnimation {
    transient: StandardTransient,
    base: AisAnimationBase,
    frames: Arc<SharedState<Vec<RecordedAnimationFrame>>>,
    pub stop_calls: SharedState<usize>,
    pub append_parent: SharedState<Handle<dyn AisAnimationSource>>,
    pub append_child: SharedState<Handle<dyn AisAnimationSource>>,
}
rustycad_tkernel::impl_transient!(RecordingAnimation,"RecordingAnimation",dyn AisAnimationSource,transient,
    interfaces:[dyn AisAnimationSource]);
impl RecordingAnimation {
    pub fn new(name: &str, frames: Arc<SharedState<Vec<RecordedAnimationFrame>>>) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisAnimationBase::new(name),
            frames,
            stop_calls: SharedState::new(0),
            append_parent: SharedState::new(Handle::default()),
            append_child: SharedState::new(Handle::default()),
        }
    }
}
impl AisAnimationSource for RecordingAnimation {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base
    }
    fn update_own(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        self.frames.write().push(RecordedAnimationFrame {
            name: String::from_utf8_lossy(self.name().as_bytes()).into_owned(),
            progress: *progress,
            stopped: self.is_stopped(),
        });
        let child = self.append_child.write().clone();
        *self.append_child.write() = Handle::default();
        if !child.is_null() {
            self.append_parent.read().add(&child)?;
        }
        Ok(())
    }
    fn stop(&self) -> Result<(), Failure> {
        *self.stop_calls.write() += 1;
        ais_animation_stop(self)
    }
}
