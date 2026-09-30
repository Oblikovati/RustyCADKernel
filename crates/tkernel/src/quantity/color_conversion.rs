//! OCCT color conversions, retaining double intermediates and float outputs.
use super::Color;
use crate::standard::Failure;
const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
const LAB_EPSILON: f64 = 0.008856451679035631;
const LAB_KAPPA: f64 = 7.787037037037037;
impl Color {
    /// Gamma-encodes a double channel; `Color::linear_to_srgb(1.0) == 1.0` within rounding.
    pub fn linear_to_srgb(value: f64) -> f64 {
        if value <= 0.0031308 {
            return value * 12.92;
        }
        value.powf(1.0 / 2.4) * 1.055 - 0.055
    }
    /// Decodes a double channel; `Color::srgb_to_linear(0.0) == 0.0`.
    pub fn srgb_to_linear(value: f64) -> f64 {
        if value <= 0.04045 {
            return value / 12.92;
        }
        ((value + 0.055) / 1.055).powf(2.4)
    }
    /// Gamma-encodes a float channel; `Color::linear_to_srgb_f32(0.0) == 0.0`.
    pub fn linear_to_srgb_f32(value: f32) -> f32 {
        if value <= 0.0031308 {
            return value * 12.92;
        }
        value.powf(1.0 / 2.4) * 1.055 - 0.055
    }
    /// Decodes a float channel; `Color::srgb_to_linear_f32(0.0) == 0.0`.
    pub fn srgb_to_linear_f32(value: f32) -> f32 {
        if value <= 0.04045 {
            return value / 12.92;
        }
        ((value + 0.055) / 1.055).powf(2.4)
    }
    /// Converts gamma-encoded channels to OCCT HLS; `Color::srgb_to_hls([1.0,0.0,0.0])`.
    pub fn srgb_to_hls([r, g, b]: [f32; 3]) -> [f32; 3] {
        let (mut maximum, mut plus, mut difference) = (r, 0.0, g - b);
        if g > maximum {
            maximum = g;
            plus = 2.0;
            difference = b - r;
        }
        if b > maximum {
            maximum = b;
            plus = 4.0;
            difference = r - g;
        }
        let mut minimum = r;
        if g < minimum {
            minimum = g;
        }
        if b < minimum {
            minimum = b;
        }
        let delta = maximum - minimum;
        let saturation = if maximum != 0.0 { delta / maximum } else { 0.0 };
        let mut hue = -1.0;
        if saturation != 0.0 {
            hue = 60.0 * (plus + difference / delta);
        }
        if hue < 0.0 && saturation != 0.0 {
            hue += 360.0;
        }
        [hue, maximum, saturation]
    }
    /// Converts OCCT HLS to gamma-encoded channels; `Color::hls_to_srgb([0.0,1.0,1.0])?`.
    pub fn hls_to_srgb([hue, light, saturation]: [f32; 3]) -> Result<[f32; 3], Failure> {
        if saturation == 0.0 && hue == -1.0 {
            return Ok([light; 3]);
        }
        let hue = if hue == 360.0 { 0.0 } else { hue / 60.0 };
        let product = light * saturation;
        // A NaN hue makes the native `(int)aHue` undefined; x86-64 cvttss2si yields INT_MIN,
        // which reaches the "Color out" throw. Rust's saturating cast would give 0 instead.
        let index = if hue.is_nan() { i32::MIN } else { hue as i32 };
        let rgb = match index {
            0 => [light, light - product + product * hue, light - product],
            1 => [light + product - product * hue, light, light - product],
            2 => [
                light - product,
                light,
                light - 3.0 * product + product * hue,
            ],
            3 => [
                light - product,
                light + 3.0 * product - product * hue,
                light,
            ],
            4 => [
                light - 5.0 * product + product * hue,
                light - product,
                light,
            ],
            5 => [
                light,
                light - product,
                light + 5.0 * product - product * hue,
            ],
            _ => return Err(Failure::out_of_range("Color out")),
        };
        Ok(rgb)
    }
    /// Converts linear RGB to Lab with D65 white; `Color::linear_rgb_to_lab([1.0;3])`.
    pub fn linear_rgb_to_lab(rgb: [f32; 3]) -> [f32; 3] {
        let [r, g, b] = rgb.map(f64::from);
        let x = (r * 0.4124564 + g * 0.3575761 + b * 0.1804375) * 100.0 / 95.047;
        let y = r * 0.2126729 + g * 0.7151522 + b * 0.0721750;
        let z = (r * 0.0193339 + g * 0.1191920 + b * 0.9503041) * 100.0 / 108.883;
        let [fx, fy, fz] = [x, y, z].map(lab_curve);
        [
            (116.0 * fy - 16.0) as f32,
            (500.0 * (fx - fy)) as f32,
            (200.0 * (fy - fz)) as f32,
        ]
    }
    /// Converts Lab, reducing chroma by 0.1% steps to fit gamut; `Color::lab_to_linear_rgb([50.0,0.0,0.0])`.
    pub fn lab_to_linear_rgb(lab: [f32; 3]) -> [f32; 3] {
        for rate in (0..=1000).rev() {
            let channels = lab_rgb_at_rate(lab, rate);
            if rate == 0 || channels.iter().all(|v| *v >= 0.0 && *v <= 1.0) {
                return channels.map(|v| v as f32);
            }
        }
        unreachable!("rate zero always returns")
    }
    /// Converts Lab to cylindrical components; `Color::lab_to_lch([50.0,25.0,25.0])`.
    pub fn lab_to_lch([light, a, b]: [f32; 3]) -> [f32; 3] {
        let (a, b) = (f64::from(a), f64::from(b));
        let chroma = (a * a + b * b).sqrt();
        let mut hue = if chroma > Self::epsilon() {
            b.atan2(a) * RAD_TO_DEG
        } else {
            0.0
        };
        if hue < 0.0 {
            hue += 360.0;
        }
        [light, chroma as f32, hue as f32]
    }
    /// Converts cylindrical components to Lab; `Color::lch_to_lab([50.0,25.0,45.0])`.
    pub fn lch_to_lab([light, chroma, hue]: [f32; 3]) -> [f32; 3] {
        let (chroma, hue) = (f64::from(chroma), f64::from(hue) * DEG_TO_RAD);
        [
            light,
            (chroma * hue.cos()) as f32,
            (chroma * hue.sin()) as f32,
        ]
    }
}
fn lab_curve(value: f64) -> f64 {
    if value > LAB_EPSILON {
        return value.powf(1.0 / 3.0);
    }
    LAB_KAPPA * value + 16.0 / 116.0
}
fn inverse_lab_curve(value: f64) -> f64 {
    let cube = value * value * value;
    if cube > LAB_EPSILON {
        return cube;
    }
    (value - 16.0 / 116.0) / LAB_KAPPA
}
fn lab_rgb_at_rate(lab: [f32; 3], rate: i32) -> [f64; 3] {
    let [light, a, b] = lab.map(f64::from);
    let chroma = f64::from(rate) / 1000.0;
    let fy = (light + 16.0) / 116.0;
    let fx = chroma * a / 500.0 + fy;
    let fz = fy - chroma * b / 200.0;
    let x = inverse_lab_curve(fx) * 95.047;
    let y = inverse_lab_curve(fy) * 100.0;
    let z = inverse_lab_curve(fz) * 108.883;
    [
        (x * 3.2404542 + y * (-1.5371385) + z * (-0.4985314)) / 100.0,
        (x * (-0.9692660) + y * 1.8760108 + z * 0.0415560) / 100.0,
        (x * 0.0556434 + y * (-0.2040259) + z * 1.0572252) / 100.0,
    ]
}
