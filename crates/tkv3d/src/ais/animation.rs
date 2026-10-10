//! Martensite animation timelines preserve AIS_Animation at OCCT 3d097a0.
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
    tcollection::AsciiString,
};
use rustycad_tkservice::media::MediaTimer;

/// Native presentation timestamps; `AisAnimationProgress::default()` is undefined (-1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AisAnimationProgress {
    /// Global presentation timestamp.
    pub pts: f64,
    /// Timestamp relative to this animation's start.
    pub local_pts: f64,
    /// Own-duration fraction, clamped in native comparison order.
    pub local_normalized: f64,
}
impl Default for AisAnimationProgress {
    fn default() -> Self {
        Self {
            pts: -1.,
            local_pts: -1.,
            local_normalized: -1.,
        }
    }
}
/// Native protected playback state; `AisAnimationState::Stopped`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AisAnimationState {
    /// Playback is active.
    Started,
    /// Playback has ended or has not started.
    Stopped,
    /// Playback is paused; child animations have been stopped.
    Paused,
}
/// Protected timeline storage available to derived animations.
pub struct AisAnimationFields {
    /// Native name, unaffected by CopyFrom.
    pub name: AsciiString,
    /// Children retain stable sequence nodes across callback insertions.
    pub children: Sequence<Handle<dyn AisAnimationSource>>,
    /// Optional injected playback timer.
    pub timer: Handle<MediaTimer>,
    /// Native playback state.
    pub state: AisAnimationState,
    /// Start time relative to the containing timeline.
    pub start_pts: f64,
    /// Duration of the animation's own callback.
    pub own_duration: f64,
    /// Cached maximum child end time.
    pub children_duration: f64,
}
/// Embedded base for derived animation atoms; `AisAnimationBase::new("move")`.
pub struct AisAnimationBase {
    /// Native protected fields, shared by handles.
    pub fields: SharedState<AisAnimationFields>,
}
impl AisAnimationBase {
    /// Initialize a stopped, empty timeline; `AisAnimationBase::new("move")`.
    pub fn new(name: impl Into<AsciiString>) -> Self {
        Self {
            fields: SharedState::new(AisAnimationFields {
                name: name.into(),
                children: Sequence::default(),
                timer: Handle::default(),
                state: AisAnimationState::Stopped,
                start_pts: 0.,
                own_duration: 0.,
                children_duration: 0.,
            }),
        }
    }
}
/// Virtual lifecycle and frame callbacks; overrides participate in parent traversal.
pub trait AisAnimationSource: Transient {
    /// Embedded native state; `animation.animation_base()`.
    fn animation_base(&self) -> &AisAnimationBase;
    /// Start children and optionally apply the current frame; `animation.start(true)?`.
    fn start(&self, update: bool) -> Result<(), Failure> {
        super::animation_playback::ais_animation_start(self, update)
    }
    /// Pause the timer and stop children; `animation.pause()?`.
    fn pause(&self) -> Result<(), Failure> {
        super::animation_playback::ais_animation_pause(self)
    }
    /// Stop and clamp the timer to total duration; `animation.stop()?`.
    fn stop(&self) -> Result<(), Failure> {
        super::animation_playback::ais_animation_stop(self)
    }
    /// Apply a global timestamp and return whether it precedes the end; `animation.update(0.5)?`.
    fn update(&self, pts: f64) -> Result<bool, Failure> {
        super::animation_playback::ais_animation_update(self, pts)
    }
    /// Start or reuse the timer; `animation.start_timer(0., 1., true, false)?`.
    fn start_timer(&self, pts: f64, speed: f64, update: bool, paused: bool) -> Result<(), Failure> {
        super::animation_playback::ais_animation_start_timer(self, pts, speed, update, paused)
    }
    /// Apply the timer's timestamp; an absent timer is ProgramError; `animation.update_timer()?`.
    fn update_timer(&self) -> Result<f64, Failure> {
        super::animation_playback::ais_animation_update_timer(self)
    }
    /// Process children then the own frame, including the final stopped frame; `animation.update_with_children(&progress)?`.
    fn update_with_children(&self, progress: &AisAnimationProgress) -> Result<(), Failure> {
        super::animation_playback::ais_animation_update_with_children(self, progress)
    }
    /// Animation atom hook, called without holding timeline locks; `animation.update_own(&progress)?`.
    fn update_own(&self, _progress: &AisAnimationProgress) -> Result<(), Failure> {
        Ok(())
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisAnimationSource,
    "AIS_Animation",
    StandardTransient
);
/// Native concrete animation container; `Handle::new(AisAnimation::new("timeline"))`.
pub struct AisAnimation {
    transient: StandardTransient,
    base: AisAnimationBase,
}
rustycad_tkernel::impl_transient!(AisAnimation,"AIS_Animation",StandardTransient,transient,
    interfaces:[dyn AisAnimationSource]);
impl AisAnimation {
    /// Construct an empty named timeline; `AisAnimation::new("timeline")`.
    pub fn new(name: impl Into<AsciiString>) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisAnimationBase::new(name),
        }
    }
}
impl AisAnimationSource for AisAnimation {
    fn animation_base(&self) -> &AisAnimationBase {
        &self.base
    }
}
/// Native AIS_AnimationTimer typedef; `AisAnimationTimer::default()`.
pub type AisAnimationTimer = MediaTimer;
