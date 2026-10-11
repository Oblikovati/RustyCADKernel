//! Native range ordering, hue normalization and discrete color selection.
use super::AisColorScale;
use rustycad_tkernel::{
    precision,
    quantity::{Color, ColorSpace},
    standard::Failure,
};
impl AisColorScale {
    /// Return ordered endpoints; `scale.get_range()`.
    pub fn get_range(&self) -> (f64, f64) {
        let f = self.fields.read();
        (f.range.min, f.range.max)
    }
    /// Store native std::min/std::max ordering, including NaN and signed zero; `scale.set_range(10.0,0.0)`.
    pub fn set_range(&self, min: f64, max: f64) {
        let mut f = self.fields.write();
        f.range.min = if max < min { max } else { min };
        f.range.max = if min < max { max } else { min };
    }
    /// Hue corresponding to minimum; `scale.hue_min()`.
    pub fn hue_min(&self) -> f64 {
        self.fields.read().range.hls_min[0]
    }
    /// Hue corresponding to maximum; `scale.hue_max()`.
    pub fn hue_max(&self) -> f64 {
        self.fields.read().range.hls_max[0]
    }
    /// Raw hue endpoints; `scale.hue_range()`.
    pub fn hue_range(&self) -> (f64, f64) {
        (self.hue_min(), self.hue_max())
    }
    /// Retain unnormalized hue endpoints for interpolation; `scale.set_hue_range(360.0,720.0)`.
    pub fn set_hue_range(&self, min: f64, max: f64) {
        let mut f = self.fields.write();
        f.range.hls_min[0] = min;
        f.range.hls_max[0] = max;
    }
    /// Wrap finite hue by native repeated shifts; `AisColorScale::hue_to_valid_range(-30.0)`.
    pub fn hue_to_valid_range(mut hue: f64) -> f64 {
        if !hue.is_finite() {
            return hue;
        }
        while hue < 0.0 {
            hue += 360.0;
        }
        while hue > 360.0 {
            hue -= 360.0;
        }
        hue
    }
    /// Normalized endpoint colors; `scale.color_range()?`.
    pub fn color_range(&self) -> Result<(Color, Color), Failure> {
        let f = self.fields.read();
        Ok((hls_color(f.range.hls_min)?, hls_color(f.range.hls_max)?))
    }
    /// Store all HLS components from two colors; `scale.set_color_range(blue,red)`.
    pub fn set_color_range(&self, min: Color, max: Color) {
        let mut f = self.fields.write();
        f.range.hls_min = min.values(ColorSpace::Hls);
        f.range.hls_max = max.values(ColorSpace::Hls);
    }
    /// Look up using native default blue/red mapping in auto mode; `scale.find_color(0.5,&mut color)?`.
    pub fn find_color(&self, value: f64, color: &mut Color) -> Result<bool, Failure> {
        let f = self.fields.read();
        if value < f.range.min || value > f.range.max || f.range.max < f.range.min {
            *color = Color::default();
            return Ok(false);
        }
        if f.inputs.color_type != rustycad_tkservice::aspect::AspectTypeOfColorScaleData::User {
            return Self::find_color_default(
                value,
                f.range.min,
                f.range.max,
                f.range.intervals,
                color,
            );
        }
        let index = discrete_interval(value, f.range.min, f.range.max, f.range.intervals);
        let found = custom_scale_color(&f.inputs.colors, index);
        *color = found.unwrap_or_default();
        Ok(found.is_some())
    }
    /// Static native default color ramp; invalid range leaves output unchanged. `Self::find_color_default(v,0.0,1.0,10,&mut c)?`.
    pub fn find_color_default(
        value: f64,
        min: f64,
        max: f64,
        count: i32,
        color: &mut Color,
    ) -> Result<bool, Failure> {
        Self::find_color_hls(
            value,
            min,
            max,
            count,
            [230.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
            color,
        )
    }
    /// Static discrete ramp with custom HLS endpoints; `Self::find_color_hls(v,min,max,n,hls1,hls2,&mut c)?`.
    pub fn find_color_hls(
        value: f64,
        min: f64,
        max: f64,
        count: i32,
        hls_min: [f64; 3],
        hls_max: [f64; 3],
        color: &mut Color,
    ) -> Result<bool, Failure> {
        if value < min || value > max || max < min {
            return Ok(false);
        }
        let index = discrete_interval(value, min, max, count);
        *color = color_from_hls(
            f64::from(index - 1),
            0.0,
            f64::from(count - 1),
            hls_min,
            hls_max,
        )?;
        Ok(true)
    }
    pub(super) fn interval_value(&self, index: i32) -> f64 {
        let f = self.fields.read();
        let range = &f.range;
        if range.intervals <= 0 {
            return 0.0;
        }
        if f.inputs.logarithmic {
            let min = if range.min > 0.0 { range.min } else { 1.0 };
            let divisor = (range.max / min).powf(1.0 / f64::from(range.intervals));
            return min * divisor.powf(f64::from(index));
        }
        range.min + f64::from(index) * ((range.max - range.min).abs() / f64::from(range.intervals))
    }
    pub(super) fn scale_color_from_value(
        &self,
        value: f64,
        min: f64,
        max: f64,
    ) -> Result<Color, Failure> {
        let f = self.fields.read();
        color_from_hls(value, min, max, f.range.hls_min, f.range.hls_max)
    }
}
fn hls_color(hls: [f64; 3]) -> Result<Color, Failure> {
    Color::new(
        AisColorScale::hue_to_valid_range(hls[0]),
        hls[1],
        hls[2],
        ColorSpace::Hls,
    )
}
pub(super) fn color_from_hls(
    value: f64,
    min: f64,
    max: f64,
    hls_min: [f64; 3],
    hls_max: [f64; 3],
) -> Result<Color, Failure> {
    let delta = max - min;
    let t = if delta != 0.0 {
        (value - min) / delta
    } else {
        0.0
    };
    hls_color(std::array::from_fn(|i| {
        (1.0 - t) * hls_min[i] + t * hls_max[i]
    }))
}
fn discrete_interval(value: f64, min: f64, max: f64, count: i32) -> i32 {
    if (max - min).abs() <= precision::approximation() {
        return 1;
    }
    let interval = 1 + (f64::from(count) * (value - min) / (max - min)).floor() as i32;
    interval.min(count)
}

fn custom_scale_color(colors: &[Color], index: i32) -> Option<Color> {
    usize::try_from(index - 1)
        .ok()
        .and_then(|i| colors.get(i))
        .copied()
}
