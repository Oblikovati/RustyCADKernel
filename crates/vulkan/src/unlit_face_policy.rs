//! Versioned unlit face-color semantics, fixed for the lifetime of a driver.
use crate::{RenderError, RenderLightingPipeline, error::rejected};

/// Opt into owned unlit side colors; `RenderUnlitFaceColorPolicy::SideColorsV2`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderUnlitFaceColorPolicy {
    /// Replay the original rejection of distinguished untextured unlit surfaces.
    #[default]
    LegacyV1,
    /// Resolve untextured triangle front/back RGBA independently (ADR 0021).
    SideColorsV2,
}
impl RenderUnlitFaceColorPolicy {
    /// Identify recorded behavior; `assert_eq!(RenderUnlitFaceColorPolicy::LegacyV1.behavior_version(), 1)`.
    pub fn behavior_version(self) -> u32 {
        match self {
            Self::LegacyV1 => 1,
            Self::SideColorsV2 => 2,
        }
    }
    pub(crate) fn validate(self, lighting: RenderLightingPipeline) -> Result<(), RenderError> {
        if self == Self::SideColorsV2 && lighting == RenderLightingPipeline::FixedFunction {
            return Err(rejected(
                (self, lighting),
                "SideColorsV2 with programmable lighting",
            ));
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unlit_policy_versions_default_replay_and_reject_fixed_v2() {
        fn require_send_sync<T: Send + Sync>() {}
        require_send_sync::<RenderUnlitFaceColorPolicy>();
        let legacy = RenderUnlitFaceColorPolicy::default();
        let sides = RenderUnlitFaceColorPolicy::SideColorsV2;
        assert_eq!(legacy, RenderUnlitFaceColorPolicy::LegacyV1);
        assert_eq!(
            (legacy.behavior_version(), sides.behavior_version()),
            (1, 2)
        );
        assert!(
            legacy
                .validate(RenderLightingPipeline::FixedFunction)
                .is_ok()
        );
        assert!(sides.validate(RenderLightingPipeline::Programmable).is_ok());
    }
    #[test]
    fn unlit_policy_rejects_fixed_function_without_device_initialization() {
        let sides = RenderUnlitFaceColorPolicy::SideColorsV2;
        let Err(error) = crate::Graphic3dRenderDriver::with_pipeline_policies(
            crate::VulkanOptions::default(),
            Default::default(),
            Default::default(),
            RenderLightingPipeline::FixedFunction,
            sides,
        ) else {
            panic!("expected unsupported policy before device initialization");
        };
        assert!(error.to_string().contains("SideColorsV2, FixedFunction"));
    }
}
