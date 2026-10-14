//! Pixel-bound GUI packets cannot inherit ordinary model presentation scaling.
use crate::{RenderError, RenderFrame, error::rejected};
pub(super) fn validate(frame: &RenderFrame, extent: ash::vk::Extent2D) -> Result<(), RenderError> {
    let actual = [extent.width, extent.height];
    let expected = [frame.width, frame.height];
    if frame.gui_draw_start.is_some() && actual != expected {
        return Err(rejected(
            actual,
            &format!("native GUI extent {expected:?}; resize and attach a fresh packet"),
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_gui_rejects_scaling_but_model_presentation_keeps_it() {
        let mut frame = RenderFrame::new(64, 64, [0.; 4], vec![]).unwrap();
        let matching = ash::vk::Extent2D {
            width: 64,
            height: 64,
        };
        let resized = ash::vk::Extent2D {
            width: 80,
            height: 72,
        };
        assert!(validate(&frame, resized).is_ok());
        frame.gui_draw_start = Some(0);
        assert!(validate(&frame, resized).is_err());
        assert!(validate(&frame, matching).is_ok());
    }
}
