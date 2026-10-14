//! Checked GUI work refuses observed change without changing model presentation.
use crate::{
    RenderError, RenderFrame, RenderGuiSurfaceChange, RenderGuiSurfacePhase, RenderGuiSurfacePolicy,
};
#[derive(Clone, Copy)]
pub(super) struct WindowGuiContract {
    accepted: Option<[u32; 2]>,
}
impl WindowGuiContract {
    pub fn new(frame: &RenderFrame, policy: RenderGuiSurfacePolicy) -> Self {
        let checked = policy == RenderGuiSurfacePolicy::CheckedV2 && frame.gui_draw_start.is_some();
        Self {
            accepted: checked.then_some(frame.extent()),
        }
    }
    pub fn validate(
        self,
        actual: [u32; 2],
        phase: RenderGuiSurfacePhase,
    ) -> Result<(), RenderError> {
        let Some(accepted) = self.accepted else {
            return Ok(());
        };
        if accepted == actual {
            return Ok(());
        }
        Err(RenderError::surface_changed(
            RenderGuiSurfaceChange::extent_mismatch(phase, accepted, actual),
        ))
    }
    pub fn present_result(self, result: Result<bool, ash::vk::Result>) -> Result<(), RenderError> {
        if result == Err(ash::vk::Result::ERROR_OUT_OF_DATE_KHR)
            && let Some(change) = self.out_of_date(RenderGuiSurfacePhase::Present)
        {
            return Err(change);
        }
        super::window_chain::legacy_present_result(result)
    }
    pub fn out_of_date(self, phase: RenderGuiSurfacePhase) -> Option<RenderError> {
        let accepted = self.accepted?;
        Some(RenderError::surface_changed(
            RenderGuiSurfaceChange::out_of_date(phase, accepted),
        ))
    }
    pub fn observe(
        self,
        presenter: &super::WindowPresenter,
        phase: RenderGuiSurfacePhase,
    ) -> Result<(), RenderError> {
        let Some(_) = self.accepted else {
            return Ok(());
        };
        let actual = presenter.observed_extent()?;
        self.validate(actual, phase)
    }
    pub fn retain_fault<T>(
        self,
        result: Result<T, RenderError>,
        presenter: &super::WindowPresenter,
        phase: RenderGuiSurfacePhase,
    ) -> Result<T, RenderError> {
        let observation = self.observe(presenter, phase);
        combine_native_observation(result, observation)
    }
}
pub(super) fn combine_native_observation<T>(
    result: Result<T, RenderError>,
    observation: Result<(), RenderError>,
) -> Result<T, RenderError> {
    crate::error::gui_observed_failure::combine_observed_result(result, observation)
}
pub(super) trait WindowPresentationProbe {
    fn observe(&mut self, phase: RenderGuiSurfacePhase) -> Result<(), RenderError>;
}
pub(super) struct NoPresentationProbe;
impl WindowPresentationProbe for NoPresentationProbe {
    fn observe(&mut self, phase: RenderGuiSurfacePhase) -> Result<(), RenderError> {
        let _ = phase;
        Ok(())
    }
}
