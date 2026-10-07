//! Image_PixMap pixel conversions, the twin of tools/oracle/tkservice_probe_pixel.cxx.
use super::common::{Reader, attempt, end, int, real};
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::image::{
    ImageFormat, image_color_from_raw_pixel, image_color_to_raw_pixel,
    image_convert_from_half_float, image_convert_to_half_float,
};

fn print_rgba(color: ColorRgba) {
    let v = color.to_vec4();
    [v.x, v.y, v.z, v.w]
        .into_iter()
        .for_each(|c| real(f64::from(c)));
}

fn format(r: &mut Reader<'_>) -> ImageFormat {
    ImageFormat::from_value(r.integer()).expect("probe format")
}

/// `sv_pixel Format ToLinear R G B A`.
pub fn pixel_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let format = format(&mut r);
    let linear = r.flag();
    let c: Vec<f32> = (0..4).map(|_| r.real() as f32).collect();
    attempt(|| {
        let color = ColorRgba::from_vec4(Vec4::new(c[0], c[1], c[2], c[3]))?;
        let mut raw = [0u8; 16];
        image_color_to_raw_pixel(&mut raw, format, &color, linear);
        raw.iter().for_each(|b| int(i32::from(*b)));
        print_rgba(image_color_from_raw_pixel(&raw, format, linear)?);
        Ok(())
    });
    end();
}

/// `sv_rawpixel Format ToLinear byte...16`.
pub fn raw_pixel_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let format = format(&mut r);
    let linear = r.flag();
    let raw: Vec<u8> = (0..16).map(|_| r.integer() as u8).collect();
    attempt(|| {
        print_rgba(image_color_from_raw_pixel(&raw, format, linear)?);
        Ok(())
    });
    end();
}

/// `sv_half Half Float`.
pub fn half_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let half = r.integer() as u16;
    let value = r.real() as f32;
    real(f64::from(image_convert_from_half_float(half)));
    int(i32::from(image_convert_to_half_float(value)));
    end();
}
