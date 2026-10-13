//! OCCT3d097a0 stencil elements change only enable/compare, not operations or write masks.
use crate::{RenderCappingPass, stencil_program::StencilCommand};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum StencilCompare {
    #[default]
    Always,
    NotEqual,
    Equal,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum StencilOperation {
    #[default]
    Keep,
    Invert,
}
/// Captured native stencil context; `frame.stencil_state().is_enabled()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderStencilState {
    pub(crate) enabled: bool,
    pub(crate) compare: StencilCompare,
    pub(crate) reference: u32,
    pub(crate) compare_mask: u32,
    pub(crate) write_mask: u32,
    pub(crate) operations: [StencilOperation; 3],
}
impl Default for RenderStencilState {
    fn default() -> Self {
        Self {
            enabled: false,
            compare: StencilCompare::Always,
            reference: 0,
            compare_mask: 0xFF,
            write_mask: 0xFF,
            operations: [StencilOperation::Keep; 3],
        }
    }
}
impl RenderStencilState {
    /// Inspect executed enable state without mutating the context; `state.is_enabled()`.
    pub fn is_enabled(self) -> bool {
        self.enabled
    }
    /// Read the native compare/reference/mask tuple; `state.native_comparison()`.
    pub fn native_comparison(self) -> (u32, u32, u32) {
        let comparison = match self.compare {
            StencilCompare::Always => 0x207,
            StencilCompare::NotEqual => 0x205,
            StencilCompare::Equal => 0x202,
        };
        (comparison, self.reference, self.compare_mask)
    }
    /// Read executed state in native GL query order; `state.native_state_fields()`.
    pub fn native_state_fields(self) -> [u32; 8] {
        let (comparison, reference, mask) = self.native_comparison();
        let [fail, depth_fail, pass] = self.operations.map(|op| match op {
            StencilOperation::Keep => 0x1E00,
            StencilOperation::Invert => 0x150A,
        });
        [
            u32::from(self.enabled),
            comparison,
            reference,
            mask,
            self.write_mask,
            fail,
            depth_fail,
            pass,
        ]
    }
    pub(crate) fn command(&mut self, command: StencilCommand) {
        match command {
            StencilCommand::Options(enabled) => {
                self.enabled = enabled;
                if enabled {
                    self.comparison(StencilCompare::NotEqual, 1, 0xFF);
                }
            }
            StencilCommand::CappingRestore => {
                self.enabled = false;
                self.comparison(StencilCompare::Always, 0, 0xFF);
            }
        }
    }
    fn comparison(&mut self, compare: StencilCompare, reference: u32, mask: u32) {
        self.compare = compare;
        self.reference = reference;
        self.compare_mask = mask;
    }
    pub(crate) fn capping(&mut self, pass: RenderCappingPass) {
        if pass == RenderCappingPass::None {
            return;
        }
        self.enabled = true;
        self.comparison(
            if pass.is_mask() {
                StencilCompare::Always
            } else {
                StencilCompare::Equal
            },
            1,
            1,
        );
        self.operations = cap_operations(pass.is_mask());
    }
}
fn cap_operations(mask: bool) -> [StencilOperation; 3] {
    if mask {
        return [
            StencilOperation::Keep,
            StencilOperation::Invert,
            StencilOperation::Invert,
        ];
    }
    [StencilOperation::Keep; 3]
}
#[cfg(test)]
#[path = "stencil_context_tests.rs"]
mod tests;
