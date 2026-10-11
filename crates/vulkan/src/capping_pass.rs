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
    /// Tag opaque triangle geometry for capping; `draw.with_capping_pass(RenderCappingPass::Surface)?`.
    pub fn with_capping_pass(mut self, pass: RenderCappingPass) -> Result<Self, RenderError> {
        if pass != RenderCappingPass::None
            && (self.topology() != PrimitiveTopology::Triangles
                || self.textured_style.is_some()
                || self.surface_blend
                || self
                    .lighting
                    .as_ref()
                    .is_some_and(crate::RenderLighting::uses_translucent_material))
        {
            return Err(rejected(
                (
                    pass,
                    self.topology(),
                    self.surface_blend,
                    self.lighting
                        .as_ref()
                        .map(|state| state.material.common.map(|face| face.diffuse[3])),
                ),
                "opaque triangle capping geometry",
            ));
        }
        self.capping_pass = pass;
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
    let mut mask = false;
    for draw in draws {
        match (mask, draw.capping_pass) {
            (false, RenderCappingPass::None) => {}
            (false, RenderCappingPass::MaskStart) => mask = true,
            (true, RenderCappingPass::MaskContinue) => {}
            (true, RenderCappingPass::Surface) => mask = false,
            state => {
                return Err(rejected(
                    state,
                    "a complete mask-start/accumulate/surface sequence",
                ));
            }
        }
    }
    if mask {
        return Err(rejected(
            "unterminated mask",
            "a capping surface before the layer ends",
        ));
    }
    Ok(())
}
