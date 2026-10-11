//! Native color-band tessellation: discrete intervals and two distinct smoothing policies.
use super::AisColorScale;
use rustycad_tkernel::{quantity::Color, standard::Failure};
use rustycad_tkservice::aspect::AspectTypeOfColorScaleData;
#[derive(Clone, Copy)]
pub(super) struct ColorScaleQuad {
    pub y: i32,
    pub height: i32,
    pub bottom: Color,
    pub top: Color,
}
impl AisColorScale {
    pub(super) fn color_bar_quads(
        &self,
        bottom: i32,
        height: i32,
    ) -> Result<Vec<ColorScaleQuad>, Failure> {
        let step = f64::from(height) / f64::from(self.get_number_of_intervals());
        if step <= 0.0 {
            return Ok(Vec::new());
        }
        if self.is_smooth_transition() && self.get_color_type() == AspectTypeOfColorScaleData::Auto
        {
            return self.smooth_automatic_quads(bottom, height);
        }
        let mut colors = (1..=self.get_number_of_intervals())
            .map(|i| self.get_interval_color(i))
            .collect::<Result<Vec<_>, _>>()?;
        if self.is_reversed() {
            colors.reverse();
        }
        if self.is_smooth_transition() {
            return Ok(smooth_custom_quads(&colors, bottom, height, step));
        }
        Ok(discrete_quads(&colors, bottom, step))
    }
    fn smooth_automatic_quads(
        &self,
        bottom: i32,
        height: i32,
    ) -> Result<Vec<ColorScaleQuad>, Failure> {
        let colors = (0..5)
            .map(|i| self.scale_color_from_value(f64::from(i), 0.0, 4.0))
            .collect::<Result<Vec<_>, _>>()?;
        let mut y = bottom;
        let mut quads = Vec::new();
        for index in 0..4 {
            let size = if index == 3 {
                bottom + height - y
            } else {
                height / 4
            };
            quads.push(scale_quad(y, size, colors[index], colors[index + 1]));
            y += size;
        }
        Ok(quads)
    }
}
fn discrete_quads(colors: &[Color], bottom: i32, step: f64) -> Vec<ColorScaleQuad> {
    let mut y = bottom;
    let mut quads = Vec::new();
    for (index, &color) in colors.iter().enumerate() {
        let size = bottom + ((index + 1) as f64 * step) as i32 - y;
        quads.push(scale_quad(y, size, color, color));
        y += size;
    }
    quads
}
fn smooth_custom_quads(
    colors: &[Color],
    bottom: i32,
    height: i32,
    step: f64,
) -> Vec<ColorScaleQuad> {
    let half = (step / 2.0) as i32;
    let first = bottom + half;
    let mut y = first;
    let mut quads = vec![scale_quad(bottom, half, colors[0], colors[0])];
    for index in 0..colors.len() - 1 {
        let size = first + ((index + 1) as f64 * step) as i32 - y;
        quads.push(scale_quad(y, size, colors[index], colors[index + 1]));
        y += size;
    }
    let last = *colors.last().expect("positive interval count");
    quads.push(scale_quad(y, bottom + height - y, last, last));
    quads
}

fn scale_quad(y: i32, height: i32, bottom: Color, top: Color) -> ColorScaleQuad {
    ColorScaleQuad {
        y,
        height,
        bottom,
        top,
    }
}
