//! Fixed GL stipple counters belong to actual line draws, not global fragment coordinates.
use crate::{PrimitiveDraw, RenderError, error::rejected};
pub(crate) fn fixed_line_stipple(draw: &PrimitiveDraw) -> Option<(u32, u16)> {
    if !draw.fixed_function_lighting {
        return None;
    }
    let style = draw.line_style.or(draw.polygon_edge_style)?;
    (style.pattern != 0xffff).then_some((u32::from(style.factor), style.pattern))
}
pub(crate) fn require_fixed_line_stipple(
    requested: bool,
    supported: bool,
) -> Result<(), RenderError> {
    if requested && !supported {
        return Err(rejected(
            (requested, supported),
            "enabled stippled Bresenham support for native fixed line counters",
        ));
    }
    Ok(())
}
pub(crate) fn restore_fixed_line_strip(draw: &mut PrimitiveDraw) {
    if !draw.source_line_strip || draw.line_style.is_none() || draw.indices.is_empty() {
        return;
    }
    let mut strip = Vec::with_capacity(draw.indices.len() / 2 + 1);
    strip.push(draw.indices[0]);
    strip.extend(draw.indices.chunks_exact(2).map(|segment| segment[1]));
    draw.indices = strip;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported_hardware_rejects_requested_counters_only() {
        assert!(require_fixed_line_stipple(true, false).is_err());
        assert!(require_fixed_line_stipple(false, false).is_ok());
        assert!(require_fixed_line_stipple(true, true).is_ok());
    }
    #[test]
    fn ordinary_lines_keep_shader_policy_and_fixed_strips_retain_continuity() {
        let vertices = vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3];
        let mut draw = PrimitiveDraw::segments(
            vertices,
            vec![0, 1, 1, 2],
            crate::identity_clip(),
            crate::LineStyle::new(1., 0x00ff, 1).unwrap(),
        )
        .unwrap();
        assert_eq!(fixed_line_stipple(&draw), None);
        draw.fixed_function_lighting = true;
        draw.source_line_strip = true;
        assert_eq!(fixed_line_stipple(&draw), Some((1, 0x00ff)));
        restore_fixed_line_strip(&mut draw);
        assert_eq!(draw.indices, vec![0, 1, 2]);
    }
}
