//! `Graphic3d_MarkerImage::StandardMarker`: the predefined marker sprites built from the
//! bitmap tables (simple markers, circled markers, rings and the shaded ball).
use super::Graphic3dMarkerImage;
use super::marker_image::{bitmap_row_bytes, bytes_array};
use super::marker_tables::{
    MARKER_RASTER, PM_FONT_INFO, TEL_NO_OF_SIZES, TEL_PM_END_SIZE, TEL_PM_START_SIZE,
};
use crate::aspect::AspectTypeOfMarker;
use crate::cast::{int_from_f32, int_from_f64};
use crate::image::{ImageColorRgba, ImageFormat, ImagePixMap};
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::exceptions::null_value;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;

/// `THE_MARKER_NAMES`, by `Aspect_TypeOfMarker` from `POINT` to `BALL`.
const MARKER_NAMES: [&str; 13] = [
    ".", "+", "*", "x", "o", "o.", "o+", "o*", "ox", "ring1", "ring2", "ring3", "ball",
];

/// `getMarkerBitMapParam`: width, height, raster offset and byte count of a marker bitmap.
fn marker_bitmap_param(kind: AspectTypeOfMarker, scale: f32) -> (i32, i32, usize, i32) {
    let table = (kind as i32).min(AspectTypeOfMarker::O as i32) as usize;
    let index = (TEL_NO_OF_SIZES - 1) as f64 * (f64::from(scale) - TEL_PM_START_SIZE)
        / (TEL_PM_END_SIZE - TEL_PM_START_SIZE);
    let id = int_from_f64(index + 0.5).clamp(0, TEL_NO_OF_SIZES as i32 - 1) as usize;
    let info = PM_FONT_INFO[table][id];
    let (width, height) = (int_from_f32(info.width), int_from_f32(info.height));
    (width, height, info.offset, height * bitmap_row_bytes(width))
}

/// `getTextureImage`: the bitmap marker of a simple type and scale.
fn texture_image(kind: AspectTypeOfMarker, scale: f32) -> Graphic3dMarkerImage {
    let (width, height, offset, count) = marker_bitmap_param(kind, scale);
    let bytes = MARKER_RASTER[offset..offset + count.max(0) as usize].to_vec();
    Graphic3dMarkerImage::from_bitmap(bytes_array(bytes), width, height)
}

/// `fillPointBitmap`: a full square bitmap of `size` pixels.
fn point_bitmap(size: i32) -> Graphic3dMarkerImage {
    let count = (bitmap_row_bytes(size) * size).max(0) as usize;
    Graphic3dMarkerImage::from_bitmap(bytes_array(vec![255; count]), size, size)
}

/// `mergeImages`: the two Alpha images OR-ed, centred in a square of the larger size.
fn merge_images(first: &Handle<ImagePixMap>, second: &Handle<ImagePixMap>) -> Handle<ImagePixMap> {
    if first.is_null() && second.is_null() {
        return Handle::default();
    }
    let size_of = |image: &Handle<ImagePixMap>| {
        image
            .get()
            .map_or((0, 0), |i| (i.width() as i32, i.height() as i32))
    };
    let ((w1, h1), (w2, h2)) = (size_of(first), size_of(second));
    let (max_width, max_height) = (w1.max(w2), h1.max(h2));
    let side = max_width.max(max_height) as usize;
    let result = ImagePixMap::default();
    result.init_zero(ImageFormat::Alpha, side, side, 0, 0);
    for image in [first, second] {
        if let Some(pixmap) = image.get() {
            or_into(&result, pixmap, (max_width, max_height));
        }
    }
    Handle::new(result)
}

/// ORs one image into the merge result at its centring offset.
fn or_into(result: &ImagePixMap, image: &ImagePixMap, (max_width, max_height): (i32, i32)) {
    let (width, height) = (image.width() as i32, image.height() as i32);
    let (x_offset, y_offset) = (
        (width - max_width).abs() / 2,
        (height - max_height).abs() / 2,
    );
    let source = image.data();
    let mut target = result.change_data();
    for y in 0..height {
        let (Some(from), Some(to)) = (
            source.data.row(y as usize),
            target.data.change_row((y_offset + y) as usize),
        ) else {
            continue;
        };
        for x in 0..width as usize {
            to[x_offset as usize + x] |= from[x];
        }
    }
}

/// The key `Graphic3d_MarkerImage[Alpha]_<name>_<scale*10>`.
fn marker_key(prefix: &str, kind: AspectTypeOfMarker, scale: f32) -> AsciiString {
    let scale_int = int_from_f32(scale * 10.0 + 0.5);
    AsciiString::from(format!("{prefix}{}_{scale_int}", MARKER_NAMES[kind as usize]).as_str())
}

/// `(unsigned int)(255.0f * c)` printed as two hex digits (x86-64: a 64-bit truncation).
fn hex_channel(channel: f32) -> String {
    format!("{:02X}", (f64::from(255.0f32 * channel)) as i64 as u32)
}

impl Graphic3dMarkerImage {
    /// `StandardMarker`: the sprite of a predefined marker type, scale and (ball) color;
    /// null for `USERDEFINED` and `EMPTY`. A ring scale below 1 has no image, which the native
    /// constructor dereferences; here it is `Standard_NullObject`. The ball color goes through
    /// `Quantity_Color::HlsRgb`, whose "Color out" failure (a NaN color) is returned.
    ///
    /// Example: `Graphic3dMarkerImage::standard_marker(AspectTypeOfMarker::Plus, 3.0, color)?`.
    pub fn standard_marker(
        kind: AspectTypeOfMarker,
        scale: f32,
        color: Vec4<f32>,
    ) -> Result<Handle<Self>, Failure> {
        if matches!(
            kind,
            AspectTypeOfMarker::Userdefined | AspectTypeOfMarker::Empty
        ) {
            return Ok(Handle::default());
        }
        let mut key = marker_key("Graphic3d_MarkerImage_", kind, scale);
        let key_alpha = marker_key("Graphic3d_MarkerImageAlpha_", kind, scale);
        if kind == AspectTypeOfMarker::Ball {
            key.assign_cat(
                [color.x, color.y, color.z]
                    .map(hex_channel)
                    .concat()
                    .as_str(),
            );
        }
        use AspectTypeOfMarker as T;
        let marker = match kind {
            T::OPoint | T::OPlus | T::OStar | T::OX => circled_marker(kind, scale, &key)?,
            T::Ring1 | T::Ring2 | T::Ring3 => ring_marker(kind, scale, &key)?,
            T::Ball => ball_marker(scale, color, key, key_alpha)?,
            _ => {
                let marker = texture_image(kind, scale);
                *marker.ids.write() = (key.clone(), key);
                marker
            }
        };
        Ok(Handle::new(marker))
    }
}

/// A circle merged with a point, plus, star or x.
fn circled_marker(
    kind: AspectTypeOfMarker,
    scale: f32,
    key: &AsciiString,
) -> Result<Graphic3dMarkerImage, Failure> {
    let circle = texture_image(AspectTypeOfMarker::O, scale);
    let inner = if kind == AspectTypeOfMarker::OPoint {
        point_bitmap(if scale > 7.0 {
            7
        } else {
            int_from_f32(scale + 0.5)
        })
    } else {
        let simple = (kind as i32) - (AspectTypeOfMarker::OPoint as i32);
        texture_image(marker_of(simple), scale)
    };
    let image = merge_images(&circle.get_image(), &inner.get_image());
    Graphic3dMarkerImage::with_ids(key.clone(), key.clone(), image, Handle::default())
}

/// The marker type of an integer value (only simple types are asked for).
fn marker_of(value: i32) -> AspectTypeOfMarker {
    use AspectTypeOfMarker as T;
    [T::Point, T::Plus, T::Star, T::X, T::O]
        .into_iter()
        .find(|kind| *kind as i32 == value)
        .unwrap_or(T::Point)
}

/// Concentric circles from `scale` down to the ring's limit in steps of 0.1 (float).
fn ring_marker(
    kind: AspectTypeOfMarker,
    scale: f32,
    key: &AsciiString,
) -> Result<Graphic3dMarkerImage, Failure> {
    let limit = match kind {
        AspectTypeOfMarker::Ring1 => scale * 0.2,
        AspectTypeOfMarker::Ring2 => scale * 0.5,
        _ => scale * 0.8,
    };
    let (mut current, mut image) = (scale, Handle::<ImagePixMap>::default());
    while current > limit && current >= 1.0 {
        image = merge_images(
            &image,
            &texture_image(AspectTypeOfMarker::O, current).get_image(),
        );
        current -= 0.1;
    }
    if image.is_null() {
        return Err(null_value(&format!(
            "Graphic3d_MarkerImage: no ring image below scale 1; got scale {scale}"
        )));
    }
    Graphic3dMarkerImage::with_ids(key.clone(), key.clone(), image, Handle::default())
}

/// The shaded ball: filled circles from `scale` down to 1, each 5% less saturated.
fn ball_marker(
    scale: f32,
    color: Vec4<f32>,
    key: AsciiString,
    key_alpha: AsciiString,
) -> Result<Graphic3dMarkerImage, Failure> {
    let (width, height, _, _) = marker_bitmap_param(AspectTypeOfMarker::O, scale);
    let side = (width + 2).max(height + 2) as usize;
    let (image, image_alpha) = (ImagePixMap::default(), ImagePixMap::default());
    image.init_zero(ImageFormat::Rgba, side, side, 0, 0);
    image_alpha.init_zero(ImageFormat::Alpha, side, side, 0, 0);
    let mut rgb = [f64::from(color.x), f64::from(color.y), f64::from(color.z)];
    let mut current = scale;
    while current >= 1.0 {
        let [hue, light, saturation] = Color::rgb_hls(rgb[0], rgb[1], rgb[2]);
        rgb = Color::hls_rgb(hue, light, saturation * 0.95)?;
        let [r, g, b] = rgb.map(|c| int_from_f64(255.0 * c) as u8);
        let pixel = ImageColorRgba { v: [r, g, b, 255] };
        stamp_circle(&image, &image_alpha, pixel, current);
        current -= 0.1;
    }
    Graphic3dMarkerImage::with_ids(key, key_alpha, Handle::new(image), Handle::new(image_alpha))
}

/// Paints the circle of `scale` centred in the ball images. Native quirk: the row offset is
/// computed from the widths and the column offset from the heights (the images are square).
fn stamp_circle(image: &ImagePixMap, image_alpha: &ImagePixMap, pixel: ImageColorRgba, scale: f32) {
    let circle = texture_image(AspectTypeOfMarker::O, scale).get_image();
    let Some(circle) = circle.get() else { return };
    let diff_x = image.width().wrapping_sub(circle.width()) / 2;
    let diff_y = image.height().wrapping_sub(circle.height()) / 2;
    let source = circle.data();
    let (mut colors, mut alphas) = (image.change_data(), image_alpha.change_data());
    for row in 0..circle.height() {
        let Some(bits) = source.data.row(row) else {
            continue;
        };
        for column in (0..circle.width()).filter(|column| bits[*column] != 0) {
            colors.data.set_typed(diff_x + row, diff_y + column, pixel);
            alphas.data.set_typed(diff_x + row, diff_y + column, 255u8);
        }
    }
}
