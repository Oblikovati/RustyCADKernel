//! Explicit odd/even mask boundaries keep native capping order visible to every backend.
use crate::{PrimitiveDraw, PrimitiveTopology, RenderError, error::rejected};
/// Capping phase for one draw; `RenderCappingPass::MaskStart` resets the stencil mask.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderCappingPass {
    /// Ordinary drawing with the layer's depth policy.
    #[default]
    None,
    /// Clear stencil and toggle its low bit without color/depth writes.
    MaskStart,
    /// Accumulate another closed group into the current mask.
    MaskContinue,
    /// Draw the section where the mask is odd, with native LESS depth testing.
    Surface,
}
impl RenderCappingPass {
    pub(crate) fn is_mask(self) -> bool {
        matches!(self, Self::MaskStart | Self::MaskContinue)
    }
}
impl PrimitiveDraw {
    /// Tag triangle geometry for capping; `draw.with_capping_pass(RenderCappingPass::Surface)?`.
    pub fn with_capping_pass(mut self, pass: RenderCappingPass) -> Result<Self, RenderError> {
        if pass != RenderCappingPass::None
            && (self.topology() != PrimitiveTopology::Triangles || self.textured_style.is_some())
        {
            return Err(rejected(
                (pass, self.topology(), self.textured_style.is_some()),
                "triangle capping geometry without a glyph atlas",
            ));
        }
        self.capping_pass = pass;
        if pass != RenderCappingPass::None {
            self.unlit_back_color = None;
        }
        // Native FillModeOnly masks use the workspace unlit aspect.
        if pass.is_mask() {
            self.lighting = None;
        }
        Ok(self)
    }
    /// Inspect the immutable phase; `draw.capping_pass()`.
    pub fn capping_pass(&self) -> RenderCappingPass {
        self.capping_pass
    }
}
pub(crate) fn validate_capping_sequence(draws: &[PrimitiveDraw]) -> Result<(), RenderError> {
    let mut phase = CappingSequencePhase::Ordinary;
    for draw in draws {
        phase = next_capping_phase(phase, draw.capping_pass)?;
    }
    if phase == CappingSequencePhase::Mask {
        return Err(rejected(
            "unterminated mask",
            "a capping surface before the layer ends",
        ));
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum CappingSequencePhase {
    Ordinary,
    Mask,
    Surface,
}
fn next_capping_phase(
    phase: CappingSequencePhase,
    pass: RenderCappingPass,
) -> Result<CappingSequencePhase, RenderError> {
    use CappingSequencePhase::*;
    let next = match (phase, pass) {
        (Ordinary | Surface, RenderCappingPass::None) => Ordinary,
        (Ordinary | Surface, RenderCappingPass::MaskStart) => Mask,
        (Mask, RenderCappingPass::MaskContinue) => Mask,
        (Mask | Surface, RenderCappingPass::Surface) => Surface,
        state => {
            return Err(rejected(
                state,
                "a complete mask-start/accumulate/surface sequence",
            ));
        }
    };
    Ok(next)
}
