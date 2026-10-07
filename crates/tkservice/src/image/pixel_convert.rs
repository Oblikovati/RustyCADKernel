//! `Image_PixMap::ColorFromRawPixel`, `ColorToRawPixel` and the half-float conversions.
use super::ImageFormat;
use super::colors::*;
use super::pixel::ImagePixel;
use rustycad_tkernel::ncollection::{Vec2, Vec4};
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::Failure;

/// `ConvertFromHalfFloat`: the float of an IEEE half (bit-exact native bit hack).
///
/// Example: `image_convert_from_half_float(0x3C00) == 1.0`.
pub fn image_convert_from_half_float(half: u16) -> f32 {
    let half = u32::from(half);
    let e = (half & 0x7C00) >> 10;
    let m = (half & 0x03FF) << 13;
    // The bit pattern of (float)m: an evil log2 to count leading zeros of a denormal.
    let v = (m as f32).to_bits() >> 23;
    let normal = u32::from(e != 0) * (((e + 112) << 23) | m);
    let denormal = u32::from(e == 0 && m != 0)
        * ((v.wrapping_sub(37) << 23) | (m.wrapping_shl(150u32.wrapping_sub(v)) & 0x007F_E000));
    f32::from_bits(((half & 0x8000) << 16) | normal | denormal)
}

/// `ConvertToHalfFloat`: the IEEE half of a float, rounded to nearest even, saturated.
///
/// Example: `image_convert_to_half_float(1.0) == 0x3C00`.
pub fn image_convert_to_half_float(value: f32) -> u16 {
    let b = value.to_bits().wrapping_add(0x0000_1000);
    let e = (b & 0x7F80_0000) >> 23;
    let m = b & 0x007F_FFFF;
    let normal = u32::from(e > 112) * ((((e.wrapping_sub(112)) << 10) & 0x7C00) | (m >> 13));
    let denormal = u32::from(e < 113 && e > 101)
        * ((((0x007F_F000 + m) >> (125u32.wrapping_sub(e) & 31)) + 1) >> 1);
    let saturate = u32::from(e > 143) * 0x7FFF;
    (((b & 0x8000_0000) >> 16) | normal | denormal | saturate) as u16
}

/// `Quantity_ColorRGBA(NCollection_Vec4<float>)`, which validates the RGB range natively.
fn rgba(r: f32, g: f32, b: f32, a: f32) -> Result<ColorRgba, Failure> {
    ColorRgba::from_vec4(Vec4::new(r, g, b, a))
}

/// An 8-bit channel in [0, 1], linearized from sRGB when asked.
fn channel8(value: u8, to_linearize: bool) -> f32 {
    let value = f32::from(value) / 255.0;
    if to_linearize {
        Color::srgb_to_linear_f32(value)
    } else {
        value
    }
}

/// Three 8-bit channels and an alpha as a color (the RGB, BGR, RGBA... branches).
fn color8(rgb: [u8; 3], alpha: f32, to_linearize: bool) -> Result<ColorRgba, Failure> {
    let [r, g, b] = rgb.map(|value| channel8(value, to_linearize));
    rgba(r, g, b, alpha)
}

/// `ColorFromRawPixel`: the color of a pixel of `format`, transparent black for an unknown
/// format. Out-of-range float channels raise `Standard_OutOfRange`, as the color constructor
/// does natively. Example: `image_color_from_raw_pixel(&[255], ImageFormat::Gray, false)?`.
pub fn image_color_from_raw_pixel(
    raw: &[u8],
    format: ImageFormat,
    to_linearize: bool,
) -> Result<ColorRgba, Failure> {
    use ImageFormat as F;
    match format {
        F::GrayF => {
            let p = f32::read_pixel(raw);
            rgba(p, p, p, 1.0)
        }
        F::AlphaF => rgba(1.0, 1.0, 1.0, f32::read_pixel(raw)),
        F::Rgf => {
            let p = ImageColorRgf::read_pixel(raw);
            rgba(p.r(), p.g(), 0.0, 1.0)
        }
        F::Rgbaf => {
            let p = ImageColorRgbaf::read_pixel(raw);
            rgba(p.r(), p.g(), p.b(), p.a())
        }
        F::Bgraf => {
            let p = ImageColorBgraf::read_pixel(raw);
            rgba(p.r(), p.g(), p.b(), p.a())
        }
        F::Rgbf => {
            let p = ImageColorRgbf::read_pixel(raw);
            rgba(p.r(), p.g(), p.b(), 1.0)
        }
        F::Bgrf => {
            let p = ImageColorBgrf::read_pixel(raw);
            rgba(p.r(), p.g(), p.b(), 1.0)
        }
        F::GrayFHalf => rgba(
            image_convert_from_half_float(u16::read_pixel(raw)),
            0.0,
            0.0,
            1.0,
        ),
        F::RgfHalf => {
            let p = <Vec2<u16>>::read_pixel(raw);
            rgba(
                image_convert_from_half_float(p.x),
                image_convert_from_half_float(p.y),
                0.0,
                1.0,
            )
        }
        F::RgbafHalf => {
            let p = <Vec4<u16>>::read_pixel(raw);
            let [r, g, b, a] = [p.x, p.y, p.z, p.w].map(image_convert_from_half_float);
            rgba(r, g, b, a)
        }
        _ => color_from_integer_pixel(raw, format, to_linearize),
    }
}

/// The 8- and 16-bit integer branches of `ColorFromRawPixel`.
fn color_from_integer_pixel(
    raw: &[u8],
    format: ImageFormat,
    to_linearize: bool,
) -> Result<ColorRgba, Failure> {
    use ImageFormat as F;
    match format {
        F::Rgba => {
            let p = ImageColorRgba::read_pixel(raw);
            color8(
                [p.r(), p.g(), p.b()],
                f32::from(p.a()) / 255.0,
                to_linearize,
            )
        }
        F::Bgra => {
            let p = ImageColorBgra::read_pixel(raw);
            color8(
                [p.r(), p.g(), p.b()],
                f32::from(p.a()) / 255.0,
                to_linearize,
            )
        }
        F::Rgb32 => {
            let p = ImageColorRgb32::read_pixel(raw);
            color8([p.r(), p.g(), p.b()], 1.0, to_linearize)
        }
        F::Bgr32 => {
            let p = ImageColorBgr32::read_pixel(raw);
            color8([p.r(), p.g(), p.b()], 1.0, to_linearize)
        }
        F::Rgb => {
            let p = ImageColorRgb::read_pixel(raw);
            color8([p.r(), p.g(), p.b()], 1.0, to_linearize)
        }
        F::Bgr => {
            let p = ImageColorBgr::read_pixel(raw);
            color8([p.r(), p.g(), p.b()], 1.0, to_linearize)
        }
        F::Gray => {
            let i = f32::from(raw[0]) / 255.0;
            ColorRgba::new(i, i, i, 1.0)
        }
        F::Alpha => ColorRgba::new(1.0, 1.0, 1.0, f32::from(raw[0]) / 255.0),
        F::Gray16 => {
            let i = f32::from(u16::read_pixel(raw)) / 65535.0;
            ColorRgba::new(i, i, i, 1.0)
        }
        _ => ColorRgba::new(0.0, 0.0, 0.0, 0.0),
    }
}

/// `static_cast<uint8_t>(value * 255.0f)` as the x86-64 compiler does it (truncation of
/// the float through a 32-bit int, then the low byte).
fn byte_of(value: f32) -> u8 {
    crate::cast::int_from_f32(value * 255.0) as u8
}

/// A channel in [0, 1] as a byte, de-linearized to sRGB when asked.
fn byte_channel(value: f32, to_delinearize: bool) -> u8 {
    byte_of(if to_delinearize {
        Color::linear_to_srgb_f32(value)
    } else {
        value
    })
}

/// The RGB bytes of a color (the 8-bit branches of `ColorToRawPixel`).
fn rgb_bytes(color: Vec4<f32>, to_delinearize: bool) -> [u8; 3] {
    [color.x, color.y, color.z].map(|value| byte_channel(value, to_delinearize))
}

/// `ColorToRawPixel`: writes a color as a pixel of `format`; nothing for an unknown format.
///
/// Example: `image_color_to_raw_pixel(&mut bytes, ImageFormat::Rgba, &color, false)`.
pub fn image_color_to_raw_pixel(
    raw: &mut [u8],
    format: ImageFormat,
    color: &ColorRgba,
    to_delinearize: bool,
) {
    use ImageFormat as F;
    let c = color.to_vec4();
    match format {
        F::GrayF => c.x.write_pixel(raw),
        F::AlphaF => c.w.write_pixel(raw),
        F::Rgf => ImageColorRgf { v: [c.x, c.y] }.write_pixel(raw),
        F::Rgbaf => ImageColorRgbaf {
            v: [c.x, c.y, c.z, c.w],
        }
        .write_pixel(raw),
        F::Bgraf => ImageColorBgraf {
            v: [c.z, c.y, c.x, c.w],
        }
        .write_pixel(raw),
        F::Rgbf => ImageColorRgbf { v: [c.x, c.y, c.z] }.write_pixel(raw),
        F::Bgrf => ImageColorBgrf { v: [c.z, c.y, c.x] }.write_pixel(raw),
        F::GrayFHalf => image_convert_to_half_float(c.x).write_pixel(raw),
        F::RgfHalf => Vec2::new(
            image_convert_to_half_float(c.x),
            image_convert_to_half_float(c.y),
        )
        .write_pixel(raw),
        F::RgbafHalf => {
            let [r, g, b, a] = [c.x, c.y, c.z, c.w].map(image_convert_to_half_float);
            Vec4::new(r, g, b, a).write_pixel(raw)
        }
        _ => integer_to_raw_pixel(raw, format, c, to_delinearize),
    }
}

/// The 8- and 16-bit integer branches of `ColorToRawPixel`; the 32-bit RGB formats keep
/// their padding byte at 255 natively.
fn integer_to_raw_pixel(raw: &mut [u8], format: ImageFormat, c: Vec4<f32>, to_delinearize: bool) {
    use ImageFormat as F;
    let [r, g, b] = rgb_bytes(c, to_delinearize);
    match format {
        F::Rgba => ImageColorRgba {
            v: [r, g, b, byte_of(c.w)],
        }
        .write_pixel(raw),
        F::Bgra => ImageColorBgra {
            v: [b, g, r, byte_of(c.w)],
        }
        .write_pixel(raw),
        F::Rgb32 => ImageColorRgb32 { v: [r, g, b, 255] }.write_pixel(raw),
        F::Bgr32 => ImageColorBgr32 { v: [b, g, r, 255] }.write_pixel(raw),
        F::Rgb => ImageColorRgb { v: [r, g, b] }.write_pixel(raw),
        F::Bgr => ImageColorBgr { v: [b, g, r] }.write_pixel(raw),
        F::Gray => raw[0] = byte_of(c.x),
        F::Alpha => raw[0] = byte_of(c.w),
        F::Gray16 => (crate::cast::int_from_f32(c.x * 65535.0) as u16).write_pixel(raw),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_floats_round_trip() {
        for half in [0x0000u16, 0x3C00, 0xC000, 0x0001, 0x03FF, 0x7BFF, 0x8001] {
            assert_eq!(
                image_convert_to_half_float(image_convert_from_half_float(half)),
                half
            );
        }
        assert_eq!(image_convert_from_half_float(0x3555), 0.333_251_95);
        assert_eq!(image_convert_to_half_float(1.0e9), 0x7FFF);
    }

    #[test]
    fn pixels_convert_both_ways() {
        let color = ColorRgba::new(1.0, 0.5, 0.0, 1.0).unwrap();
        let mut raw = [0u8; 16];
        image_color_to_raw_pixel(&mut raw, ImageFormat::Bgra, &color, false);
        assert_eq!(&raw[..4], &[0, 127, 255, 255]);
        let back = image_color_from_raw_pixel(&raw, ImageFormat::Bgra, false).unwrap();
        assert_eq!(back.to_vec4().x, 1.0);
        image_color_to_raw_pixel(&mut raw, ImageFormat::Gray16, &color, false);
        assert_eq!(u16::read_pixel(&raw), 65535);
        let transparent = image_color_from_raw_pixel(&raw, ImageFormat::Unknown, false).unwrap();
        assert_eq!(transparent.alpha(), 0.0);
        2.0f32.write_pixel(&mut raw);
        assert!(image_color_from_raw_pixel(&raw, ImageFormat::GrayF, false).is_err());
    }
}
