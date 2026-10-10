//! Nonvirtual native timeline queries and child-list mutations.
use super::{AisAnimationBase, AisAnimationFields, AisAnimationSource};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{
        Failure, Handle,
        exceptions::{null_object, program_error},
    },
    tcollection::AsciiString,
};
use rustycad_tkservice::media::MediaTimer;
use std::{ops::Deref, sync::RwLockReadGuard};
/// Borrowed native child sequence; drop the guard before editing; `animation.children().length()`.
pub struct AisAnimationChildren<'a>(RwLockReadGuard<'a, AisAnimationFields>);
impl Deref for AisAnimationChildren<'_> {
    type Target = Sequence<Handle<dyn AisAnimationSource>>;
    fn deref(&self) -> &Self::Target {
        &self.0.children
    }
}
/// Shared nonvirtual animation members; `animation.set_own_duration(2.)`.
pub trait AisAnimationMembers: AisAnimationSource {
    /// Retain the original name; `animation.name()`.
    fn name(&self) -> AsciiString {
        self.animation_base().fields.read().name.clone()
    }
    /// Start position within the parent timeline; `animation.start_pts()`.
    fn start_pts(&self) -> f64 {
        self.animation_base().fields.read().start_pts
    }
    /// Set start without refreshing parent duration; `animation.set_start_pts(1.)`.
    fn set_start_pts(&self, pts: f64) {
        self.animation_base().fields.write().start_pts = pts;
    }
    /// Maximum own/cached-child duration with native NaN and signed-zero policy; `animation.duration()`.
    fn duration(&self) -> f64 {
        let fields = self.animation_base().fields.read();
        native_max(fields.own_duration, fields.children_duration)
    }
    /// Own duration is defined only when positive; `animation.has_own_duration()`.
    fn has_own_duration(&self) -> bool {
        self.own_duration() > 0.
    }
    /// Own callback duration; `animation.own_duration()`.
    fn own_duration(&self) -> f64 {
        self.animation_base().fields.read().own_duration
    }
    /// Set duration without refreshing parent duration; `animation.set_own_duration(2.)`.
    fn set_own_duration(&self, duration: f64) {
        self.animation_base().fields.write().own_duration = duration;
    }
    /// Borrow the actual stable-node child sequence; `animation.children().length()`.
    fn children(&self) -> AisAnimationChildren<'_> {
        AisAnimationChildren(self.animation_base().fields.read())
    }
    /// Append unless the handle is already present, refreshing duration in either case; `animation.add(&child)?`.
    fn add(&self, child: &Handle<dyn AisAnimationSource>) -> Result<(), Failure> {
        if child.is_null() {
            return Err(program_error(
                "AIS_Animation::Add received null; expected a non-null animation",
            ));
        }
        if !self.children().iter().any(|current| current == child) {
            self.animation_base()
                .fields
                .write()
                .children
                .append(child.clone());
        }
        self.update_total_duration()
    }
    /// Clear children and own duration, retaining the cached child duration like OCCT; `animation.clear()`.
    fn clear(&self) {
        let mut fields = self.animation_base().fields.write();
        // AIS_Animation::Clear at 3d097a0 deliberately does not refresh myChildrenDuration.
        fields.children.clear(None);
        fields.own_duration = 0.;
    }
    /// First direct child with the name, or null; `animation.find(&AsciiString::from("move"))?`.
    fn find(&self, name: &AsciiString) -> Result<Handle<dyn AisAnimationSource>, Failure> {
        for child in self.children().iter() {
            if required_animation(child)?.name() == *name {
                return Ok(child.clone());
            }
        }
        Ok(Handle::default())
    }
    /// Remove by identity, refreshing duration only when found; `animation.remove(&child)?`.
    fn remove(&self, child: &Handle<dyn AisAnimationSource>) -> Result<bool, Failure> {
        let Some(index) = child_index(self.animation_base(), child) else {
            return Ok(false);
        };
        self.animation_base()
            .fields
            .write()
            .children
            .remove(index)?;
        self.update_total_duration()?;
        Ok(true)
    }
    /// Replace the first identity match without deduplicating; `animation.replace(&old, &new)?`.
    fn replace(
        &self,
        old: &Handle<dyn AisAnimationSource>,
        new: &Handle<dyn AisAnimationSource>,
    ) -> Result<bool, Failure> {
        let Some(index) = child_index(self.animation_base(), old) else {
            return Ok(false);
        };
        required_animation(new)?;
        self.animation_base()
            .fields
            .write()
            .children
            .set_value(index, new.clone())?;
        self.update_total_duration()?;
        Ok(true)
    }
    /// Shallow-copy children and time limits, retaining name, timer and state; self-copy empties children; `animation.copy_from(&other)?`.
    fn copy_from(&self, other: &Handle<dyn AisAnimationSource>) -> Result<(), Failure> {
        self.animation_base().fields.write().children.clear(None);
        let other = required_animation(other)?;
        let children: Vec<_> = other.children().iter().cloned().collect();
        for child in children {
            self.animation_base().fields.write().children.append(child);
        }
        self.update_total_duration()?;
        self.set_start_pts(other.start_pts());
        self.set_own_duration(other.own_duration());
        Ok(())
    }
    /// Refresh the maximum child end, without recursively refreshing descendants; `animation.update_total_duration()?`.
    fn update_total_duration(&self) -> Result<(), Failure> {
        let mut duration = 0.;
        for child in self.children().iter() {
            let child = required_animation(child)?;
            duration = native_max(duration, child.start_pts() + child.duration());
        }
        self.animation_base().fields.write().children_duration = duration;
        Ok(())
    }
    /// Timer handle, including null; `animation.timer()`.
    fn timer(&self) -> Handle<MediaTimer> {
        self.animation_base().fields.read().timer.clone()
    }
    /// Inject or detach playback timing; `animation.set_timer(&timer)`.
    fn set_timer(&self, timer: &Handle<MediaTimer>) {
        self.animation_base().fields.write().timer = timer.clone();
    }
    /// Zero without a timer; `animation.elapsed_time()`.
    fn elapsed_time(&self) -> f64 {
        self.timer().get().map_or(0., MediaTimer::elapsed_time)
    }
    /// Paused animations also count as stopped; `animation.is_stopped()`.
    fn is_stopped(&self) -> bool {
        self.animation_base().fields.read().state != super::AisAnimationState::Started
    }
}
impl<T: AisAnimationSource + ?Sized> AisAnimationMembers for T {}
pub(super) fn required_animation(
    child: &Handle<dyn AisAnimationSource>,
) -> Result<&dyn AisAnimationSource, Failure> {
    child.get().ok_or_else(|| {
        null_object("AIS animation received null; expected a non-null child animation")
    })
}
fn child_index(base: &AisAnimationBase, child: &Handle<dyn AisAnimationSource>) -> Option<usize> {
    base.fields
        .read()
        .children
        .iter()
        .position(|current| current == child)
        .map(|index| index + 1)
}
pub(super) fn native_max(left: f64, right: f64) -> f64 {
    if left < right { right } else { left }
}
pub(super) fn native_min(left: f64, right: f64) -> f64 {
    if right < left { right } else { left }
}
