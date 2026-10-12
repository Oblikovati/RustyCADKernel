//! Native text XOR requires real normalized attachment logic operations, never alpha-blend emulation.
use crate::{RenderError, error::rejected};
/// Validate a native logic-op request before Vulkan pipeline creation; `require_text_xor(true, true)?`.
pub(crate) fn require_text_xor(requested: bool, available: bool) -> Result<(), RenderError> {
    if requested && !available {
        return Err(rejected(
            (requested, available),
            "enabled device logicOp support for native text XOR",
        ));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_logic_op_is_explicit_and_ordinary_text_stays_available() {
        assert!(require_text_xor(false, false).is_ok());
        assert!(require_text_xor(true, true).is_ok());
        let error = require_text_xor(true, false).unwrap_err().to_string();
        assert!(error.contains("false") && error.contains("logicOp"));
    }
}

/// Preserve native framebuffer conversion before logic ops; `text_target_rgba(true, rgba)` keeps alpha.
pub(crate) fn text_target_rgba(xor: bool, rgba: [f32; 4]) -> [f32; 4] {
    if xor {
        return rustycad_tkernel::quantity::ColorRgba::linear_to_srgb_vec4(rgba);
    }
    rgba
}
#[cfg(test)]
mod color_tests {
    use super::*;
    #[test]
    fn normal_input_stays_linear_and_xor_converts_rgb_without_touching_alpha() {
        let input = [0.18, 0.4, 0.7, 0.4];
        assert_eq!(text_target_rgba(false, input), input);
        let encoded = text_target_rgba(true, input);
        assert_eq!(encoded[3], input[3]);
        assert_eq!(
            encoded.map(|v| (v * 255.).round() as u8),
            [118, 170, 218, 102]
        );
    }
}
