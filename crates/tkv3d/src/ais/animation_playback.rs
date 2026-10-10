//! Timeline traversal releases locks for virtual callbacks and follows live node links.
use super::animation_members::{native_max, native_min, required_animation};
use super::{AisAnimationMembers, AisAnimationProgress, AisAnimationSource, AisAnimationState};
use rustycad_tkernel::standard::{
    Failure, Handle,
    exceptions::{null_object, program_error},
};
use rustycad_tkservice::media::MediaTimer;

fn walk_children<A, F>(animation: &A, mut visit: F) -> Result<(), Failure>
where
    A: AisAnimationSource + ?Sized,
    F: FnMut(&dyn AisAnimationSource) -> Result<(), Failure>,
{
    let mut cursor = animation.children().cursor();
    while cursor.more() {
        let child = cursor
            .value(&animation.children())
            .cloned()
            .ok_or_else(|| {
                null_object(
                    "AIS animation cursor is no longer live; expected an existing child node",
                )
            })?;
        visit(required_animation(&child)?)?;
        cursor.next(&animation.children());
    }
    Ok(())
}
/// Native base lifecycle body for explicit delegation from overrides; `ais_animation_start(&animation, false)?`.
pub fn ais_animation_start<A: AisAnimationSource + ?Sized>(
    animation: &A,
    update: bool,
) -> Result<(), Failure> {
    animation.update_total_duration()?;
    animation.animation_base().fields.write().state = AisAnimationState::Started;
    walk_children(animation, |child| child.start(false))?;
    if update {
        animation.update(animation.elapsed_time())?;
    }
    if let Some(timer) = animation.timer().get() {
        timer.start();
    }
    Ok(())
}
/// Native base lifecycle body for explicit delegation from overrides; `ais_animation_pause(&animation)?`.
pub fn ais_animation_pause<A: AisAnimationSource + ?Sized>(animation: &A) -> Result<(), Failure> {
    animation.animation_base().fields.write().state = AisAnimationState::Paused;
    if let Some(timer) = animation.timer().get() {
        timer.pause();
    }
    walk_children(animation, |child| child.stop())
}
/// Native base lifecycle body for explicit delegation from overrides; `ais_animation_stop(&animation)?`.
pub fn ais_animation_stop<A: AisAnimationSource + ?Sized>(animation: &A) -> Result<(), Failure> {
    animation.animation_base().fields.write().state = AisAnimationState::Stopped;
    if let Some(timer) = animation.timer().get() {
        let elapsed = animation.elapsed_time();
        timer.stop();
        timer.seek(native_min(animation.duration(), elapsed));
    }
    walk_children(animation, |child| child.stop())
}
/// Native base lifecycle body for explicit delegation from overrides; `ais_animation_start_timer(&animation, 0., 1., true, false)?`.
pub fn ais_animation_start_timer<A: AisAnimationSource + ?Sized>(
    animation: &A,
    pts: f64,
    speed: f64,
    update: bool,
    paused: bool,
) -> Result<(), Failure> {
    if animation.timer().is_null() {
        animation.set_timer(&Handle::new(MediaTimer::default()));
    }
    let timer = animation.timer();
    timer.stop();
    timer.seek(pts);
    timer.set_playback_speed(speed);
    animation.start(update)?;
    if paused {
        timer.stop();
        timer.seek(pts);
    }
    Ok(())
}
/// Native base lifecycle body for explicit delegation from overrides; `ais_animation_update_timer(&animation)?`.
pub fn ais_animation_update_timer<A: AisAnimationSource + ?Sized>(
    animation: &A,
) -> Result<f64, Failure> {
    let timer = animation.timer();
    let timer = timer.get().ok_or_else(|| {
        program_error(
            "AIS_Animation::UpdateTimer received null timer; expected a created playback timer",
        )
    })?;
    let elapsed = timer.elapsed_time();
    animation.update(elapsed)?;
    Ok(elapsed)
}
fn animation_position<A: AisAnimationSource + ?Sized>(
    animation: &A,
    pts: f64,
    local_pts: f64,
) -> AisAnimationProgress {
    let fraction = if animation.has_own_duration() {
        local_pts / animation.own_duration()
    } else {
        0.
    };
    AisAnimationProgress {
        pts,
        local_pts,
        local_normalized: native_min(1., native_max(0., fraction)),
    }
}
/// Native base lifecycle body for explicit delegation from overrides; `ais_animation_update(&animation, 0.5)?`.
pub fn ais_animation_update<A: AisAnimationSource + ?Sized>(
    animation: &A,
    pts: f64,
) -> Result<bool, Failure> {
    let progress = animation_position(animation, pts, pts - animation.start_pts());
    animation.update_with_children(&progress)?;
    Ok(pts < animation.start_pts() + animation.duration())
}
/// Native base lifecycle body for explicit delegation from overrides; `ais_animation_update_with_children(&animation, &progress)?`.
pub fn ais_animation_update_with_children<A: AisAnimationSource + ?Sized>(
    animation: &A,
    progress: &AisAnimationProgress,
) -> Result<(), Failure> {
    if progress.local_pts < 0. || animation.is_stopped() {
        return Ok(());
    }
    walk_children(animation, |child| {
        let nested =
            animation_position(child, progress.pts, progress.local_pts - child.start_pts());
        child.update_with_children(&nested)
    })?;
    if progress.local_pts >= animation.duration() {
        animation.stop()?;
    }
    animation.update_own(progress)
}
