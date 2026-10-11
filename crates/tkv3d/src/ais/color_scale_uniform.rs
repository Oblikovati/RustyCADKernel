//! Native perceptually uniform color generation, including its float RGB interpolation.
use super::AisColorScale;
use rustycad_tkernel::{
    ncollection::Vec3,
    precision,
    quantity::{Color, ColorSpace},
    standard::{Failure, exceptions::program_error},
};
impl AisColorScale {
    /// Generate maximal-gamut CIE Lch colors with native DeltaE2000 spacing; `Self::make_uniform_colors(10,50.0,0.0,360.0)?`.
    pub fn make_uniform_colors(
        count: i32,
        lightness: f64,
        from: f64,
        to: f64,
    ) -> Result<Vec<Color>, Failure> {
        let range = uniform_hue_span(from, to);
        if count < 1 {
            return Ok(Vec::new());
        }
        if count == 1 {
            return Ok(vec![uniform_lch_color(lightness, from)?]);
        }
        let grid = uniform_color_grid(lightness, from, range)?;
        let metric = uniform_color_metric(&grid);
        let step = uniform_grid_step(&metric, count, lightness)?;
        sample_uniform_grid(&grid, &metric, step, count)
    }
    /// Set user colors and switch color mode; `scale.set_uniform_colors(50.0,0.0,360.0)?`.
    pub fn set_uniform_colors(&self, lightness: f64, from: f64, to: f64) -> Result<(), Failure> {
        let colors =
            Self::make_uniform_colors(self.get_number_of_intervals(), lightness, from, to)?;
        self.set_colors(&colors);
        self.set_color_type(rustycad_tkservice::aspect::AspectTypeOfColorScaleData::User);
        Ok(())
    }
}
fn uniform_lch_color(lightness: f64, hue: f64) -> Result<Color, Failure> {
    let mut hue = hue % 360.0;
    if hue < 0.0 {
        hue += 360.0;
    }
    Color::new(lightness, 130.0, hue, ColorSpace::Lch)
}
fn uniform_color_grid(lightness: f64, from: f64, range: f64) -> Result<Vec<Color>, Failure> {
    let count = 2 + range.abs() as usize;
    let step = range / (count - 1) as f64;
    (0..count)
        .map(|i| uniform_lch_color(lightness, from + i as f64 * step))
        .collect()
}
fn uniform_color_metric(grid: &[Color]) -> Vec<f64> {
    (0..grid.len())
        .map(|i| grid[i].delta_e2000(grid[(i + grid.len() - 1) % grid.len()]))
        .collect()
}
fn sample_uniform_grid(
    grid: &[Color],
    metric: &[f64],
    step: f64,
    count: i32,
) -> Result<Vec<Color>, Failure> {
    let mut sampler = UniformPaletteSampler {
        result: vec![grid[0]],
        previous: 0.0,
        target: step,
        step,
    };
    for index in 1..grid.len() {
        sampler.consume(grid[index - 1], grid[index], metric[index])?;
    }
    if sampler.result.len() < count as usize {
        sampler
            .result
            .push(*grid.last().expect("nonempty uniform grid"));
    }
    Ok(sampler.result)
}
fn interpolate_uniform_rgb(
    previous_color: Color,
    current_color: Color,
    previous: f64,
    parameter: f64,
    target: f64,
) -> Result<Color, Failure> {
    let previous_coefficient = ((parameter - target) / (parameter - previous)) as f32;
    let current_coefficient = ((target - previous) / (parameter - previous)) as f32;
    let a = previous_color.values(ColorSpace::Rgb).map(|v| v as f32);
    let b = current_color.values(ColorSpace::Rgb).map(|v| v as f32);
    Color::from_vec3(Vec3::new(
        b[0] * current_coefficient + a[0] * previous_coefficient,
        b[1] * current_coefficient + a[1] * previous_coefficient,
        b[2] * current_coefficient + a[2] * previous_coefficient,
    ))
}

fn uniform_hue_span(from: f64, to: f64) -> f64 {
    let range = (to - from) % 360.0;
    if range.abs() > precision::angular() * 180.0 / std::f64::consts::PI {
        return range;
    }
    if range < 0.0 { -360.0 } else { 360.0 }
}
fn uniform_grid_step(metric: &[f64], count: i32, lightness: f64) -> Result<f64, Failure> {
    let length = metric.iter().sum::<f64>();
    let mut step = (length - metric[0]) / f64::from(count - 1);
    if metric[0] < step {
        step = length / f64::from(count);
    }
    if step <= 0.0 || !step.is_finite() {
        return Err(program_error(&format!(
            "AIS_ColorScale uniform step={step}, lightness={lightness}; expected finite positive perceptual distance"
        )));
    }
    Ok(step)
}

struct UniformPaletteSampler {
    result: Vec<Color>,
    previous: f64,
    target: f64,
    step: f64,
}
impl UniformPaletteSampler {
    fn consume(
        &mut self,
        previous_color: Color,
        current_color: Color,
        distance: f64,
    ) -> Result<(), Failure> {
        let parameter = self.previous + distance;
        while self.target <= parameter {
            self.result.push(interpolate_uniform_rgb(
                previous_color,
                current_color,
                self.previous,
                parameter,
                self.target,
            )?);
            self.target += self.step;
        }
        self.previous = parameter;
        Ok(())
    }
}
