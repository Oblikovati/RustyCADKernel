//! `Image_Diff`: compares two images pixel by pixel, optionally ignoring thin border lines.
use super::pixel_convert::image_color_from_raw_pixel;
use super::{ImageAlienPixMap, ImageFormat, ImagePixMap, ImagePixMapData};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::PackedMap;
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;

/// The eight neighbours of a pixel, as `(dx, dy)`.
const NEIGHBOR_PIXELS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// The protected fields of `Image_Diff`.
#[derive(Default)]
pub struct ImageDiffFields {
    /// `myImageRef`: the reference image.
    pub image_ref: Handle<ImagePixMap>,
    /// `myImageNew`: the image compared with it.
    pub image_new: Handle<ImagePixMap>,
    /// `myColorTolerance`: 0 (any difference) to 1 (opposite colours).
    pub color_tolerance: f64,
    /// `myIsBorderFilterOn`.
    pub is_border_filter_on: bool,
    /// `myDiffPixels`: the packed `(x, y)` of every different pixel.
    pub diff_pixels: Vec<i32>,
    /// `myLinearGroups`: the 1-based ids of the groups that are thin lines.
    pub linear_groups: PackedMap<i32>,
    /// `myGroupsOfDiffPixels`: the groups of adjacent different pixels.
    pub groups_of_diff_pixels: Vec<PackedMap<i32>>,
}

/// An image comparison; `let diff = ImageDiff::default(); diff.init(&a, &b, false);`.
pub struct ImageDiff {
    transient: StandardTransient,
    fields: SharedState<ImageDiffFields>,
}
impl Default for ImageDiff {
    /// No images, zero tolerance, no border filter.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(ImageDiffFields::default()),
        }
    }
}
rustycad_tkernel::impl_transient!(ImageDiff, "Image_Diff", StandardTransient, transient);

/// `PackXY`: x in the high 16 bits, y in the low ones.
pub fn image_diff_pack_xy(x: u16, y: u16) -> i32 {
    (u32::from(y) | (u32::from(x) << 16)) as i32
}
/// `UnpackX`; `image_diff_unpack_x(image_diff_pack_xy(3, 4)) == 3`.
pub fn image_diff_unpack_x(xy: i32) -> u16 {
    ((xy as u32 & 0xffff_0000) >> 16) as u16
}
/// `UnpackY`; `image_diff_unpack_y(image_diff_pack_xy(3, 4)) == 4`.
pub fn image_diff_unpack_y(xy: i32) -> u16 {
    (xy as u32 & 0xffff) as u16
}

fn fail(text: &str) {
    Message::send_fail_text(&AsciiString::from(text));
}

/// True for the 8-bit RGB-like formats whose first three bytes are the colour.
fn is_rgb8(format: ImageFormat) -> bool {
    use ImageFormat as F;
    matches!(
        format,
        F::Rgb | F::Bgr | F::Rgb32 | F::Bgr32 | F::Rgba | F::Bgra
    )
}

/// `isBlackPixel`.
fn is_black_pixel(
    image: &ImagePixMapData,
    format: ImageFormat,
    y: usize,
    x: usize,
) -> Result<bool, Failure> {
    let raw = image.value(y, x).unwrap_or_default();
    if matches!(format, ImageFormat::Gray | ImageFormat::Alpha) {
        return Ok(raw.first() == Some(&0));
    }
    if is_rgb8(format) {
        return Ok(raw[..3] == [0, 0, 0]);
    }
    // Native quirk kept: PixelColor is called with (theY, theX) swapped.
    let color = pixel_color(image, format, y, x)?.to_vec4();
    Ok(color.x == 0.0 && color.y == 0.0 && color.z == 0.0)
}

/// `Image_PixMap::PixelColor(x, y)` on locked data: transparent black outside the image.
fn pixel_color(
    image: &ImagePixMapData,
    format: ImageFormat,
    x: usize,
    y: usize,
) -> Result<ColorRgba, Failure> {
    match image
        .value_xy(x, y)
        .filter(|_| x < image.size_x && y < image.size_y)
    {
        Some(raw) => image_color_from_raw_pixel(raw, format, false),
        None => ColorRgba::new(0.0, 0.0, 0.0, 0.0),
    }
}

/// The pixel difference measure of `Compare` for one pixel, and its threshold, compared
/// in the format's own arithmetic (`int` bytes or `float` channels).
fn pixel_differs(
    reference: &ImagePixMapData,
    new: &ImagePixMapData,
    format: ImageFormat,
    row: usize,
    col: usize,
    tolerance: f64,
) -> Result<bool, Failure> {
    if matches!(format, ImageFormat::Gray | ImageFormat::Alpha) || is_rgb8(format) {
        let threshold = crate::cast::int_from_f64(255.0 * tolerance);
        let (a, b) = (
            reference.value(row, col).unwrap_or_default(),
            new.value(row, col).unwrap_or_default(),
        );
        let channels = if is_rgb8(format) { 3 } else { 1 };
        let diff = (0..channels)
            .map(|i| (i32::from(a[i]) - i32::from(b[i])).abs())
            .max()
            .unwrap_or(0);
        return Ok(diff > threshold);
    }
    let first = *pixel_color(reference, format, col, row)?.rgb();
    let second = *pixel_color(new, format, col, row)?.rgb();
    let channel = |c: fn(&rustycad_tkernel::quantity::Color) -> f64| {
        (c(&second) as f32 - c(&first) as f32).abs()
    };
    let diff = channel(|c| c.red())
        .max(channel(|c| c.green()))
        .max(channel(|c| c.blue()));
    Ok(diff > tolerance as f32)
}

impl ImageDiff {
    /// The protected fields; `diff.fields().diff_pixels.len()`.
    pub fn fields(&self) -> std::sync::RwLockReadGuard<'_, ImageDiffFields> {
        self.fields.read()
    }
    /// `Init(images)`: keeps both images when they have the same size and format (below
    /// 65535 pixels per side), optionally turned black and white; reports otherwise.
    ///
    /// Example: `diff.init(&reference, &new, false)?`.
    pub fn init(
        &self,
        image_ref: &Handle<ImagePixMap>,
        image_new: &Handle<ImagePixMap>,
        to_black_white: bool,
    ) -> Result<bool, Failure> {
        {
            let mut fields = self.fields.write();
            (fields.image_ref, fields.image_new) = (Handle::default(), Handle::default());
            fields.diff_pixels.clear();
            release_groups(&mut fields);
        }
        let (Some(reference), Some(new)) = (image_ref.get(), image_new.get()) else {
            fail("Error: Images have different format or dimensions");
            return Ok(false);
        };
        let mismatch = reference.is_empty()
            || new.is_empty()
            || reference.size_xyz().x != new.size_xyz().x
            || reference.height() != new.height()
            || reference.format() != new.format();
        if mismatch {
            fail("Error: Images have different format or dimensions");
            return Ok(false);
        }
        if reference.width() >= 0xFFFF || reference.height() >= 0xFFFF {
            fail("Error: Images are too large");
            return Ok(false);
        }
        {
            let mut fields = self.fields.write();
            (fields.image_ref, fields.image_new) = (image_ref.clone(), image_new.clone());
        }
        if to_black_white {
            ImagePixMap::to_black_white(reference)?;
            ImagePixMap::to_black_white(new)?;
        }
        Ok(true)
    }
    /// `Init(paths)`: loads both files; without an image library this always reports
    /// "Error: Failed to load image(s) file(s)". Example: `diff.init_files(&a, &b, false)?`.
    pub fn init_files(
        &self,
        path_ref: &AsciiString,
        path_new: &AsciiString,
        to_black_white: bool,
    ) -> Result<bool, Failure> {
        let (image_ref, image_new) = (ImageAlienPixMap::default(), ImageAlienPixMap::default());
        if !image_ref.load(path_ref) || !image_new.load(path_new) {
            fail("Error: Failed to load image(s) file(s)");
            return Ok(false);
        }
        let image_ref: Handle<ImagePixMap> = Handle::new(image_ref).cast();
        let image_new: Handle<ImagePixMap> = Handle::new(image_new).cast();
        self.init(&image_ref, &image_new, to_black_white)
    }
    /// Sets the colour tolerance; `diff.set_color_tolerance(0.1)`.
    pub fn set_color_tolerance(&self, tolerance: f64) {
        self.fields.write().color_tolerance = tolerance;
    }
    /// The colour tolerance; `diff.color_tolerance()`.
    pub fn color_tolerance(&self) -> f64 {
        self.fields.read().color_tolerance
    }
    /// Turns the border filter on or off; `diff.set_border_filter_on(true)`.
    pub fn set_border_filter_on(&self, to_ignore: bool) {
        self.fields.write().is_border_filter_on = to_ignore;
    }
    /// True when the border filter is on; `diff.is_border_filter_on()`.
    pub fn is_border_filter_on(&self) -> bool {
        self.fields.read().is_border_filter_on
    }
    /// The number of different pixels (or, with the border filter, of non-linear groups),
    /// 0 for identical bytes, -1 without images. Example: `diff.compare()?`.
    pub fn compare(&self) -> Result<i32, Failure> {
        let mut fields = self.fields.write();
        fields.diff_pixels.clear();
        let (Some(reference), Some(new)) = (fields.image_ref.get(), fields.image_new.get()) else {
            return Ok(-1);
        };
        let (reference, new) = (reference.data().clone(), new.data().clone());
        if new.data.size() == reference.data.size() && new.data.data() == reference.data.data() {
            return Ok(0);
        }
        let tolerance = fields.color_tolerance;
        let mut pixels = Vec::new();
        for row in 0..reference.data.size_y {
            for col in 0..reference.data.size_x {
                if pixel_differs(
                    &reference.data,
                    &new.data,
                    reference.format,
                    row,
                    col,
                    tolerance,
                )? {
                    pixels.push(image_diff_pack_xy(col as u16, row as u16));
                }
            }
        }
        let count = pixels.len() as i32;
        fields.diff_pixels = pixels;
        if fields.is_border_filter_on && !fields.diff_pixels.is_empty() {
            return ignore_border_effect(&mut fields, &reference.data, reference.format);
        }
        Ok(count)
    }
}

/// `releaseGroupsOfDiffPixels`.
fn release_groups(fields: &mut ImageDiffFields) {
    fields.groups_of_diff_pixels.clear();
    fields.linear_groups.clear();
}

/// `ignoreBorderEffect`: groups adjacent different pixels, marks the thin-line groups
/// lying next to non-black reference pixels, and counts the other groups.
fn ignore_border_effect(
    fields: &mut ImageDiffFields,
    reference: &ImagePixMapData,
    format: ImageFormat,
) -> Result<i32, Failure> {
    release_groups(fields);
    group_adjacent_pixels(fields);
    for (index, group) in fields.groups_of_diff_pixels.iter().enumerate() {
        if group.is_empty() {
            continue;
        }
        let Some(last) = line_end_pixel(group, reference) else {
            continue;
        };
        if count_neighbours(last, reference, |x, y| {
            Ok(!is_black_pixel(reference, format, y, x)?)
        })? > 1
        {
            fields.linear_groups.add(index as i32 + 1);
        }
    }
    let total = fields.groups_of_diff_pixels.len() as i32;
    let linear = (1..=total)
        .filter(|id| fields.linear_groups.contains(*id))
        .count() as i32;
    Ok(total - linear)
}

/// Builds the groups: each adjacent pair joins the first group holding its first pixel,
/// else starts a new group (native order and quirks kept).
fn group_adjacent_pixels(fields: &mut ImageDiffFields) {
    let pixels = fields.diff_pixels.clone();
    for (index, &first) in pixels.iter().enumerate() {
        for &second in &pixels[index + 1..] {
            let dx = (i32::from(image_diff_unpack_x(first))
                - i32::from(image_diff_unpack_x(second)))
            .abs();
            let dy = (i32::from(image_diff_unpack_y(first))
                - i32::from(image_diff_unpack_y(second)))
            .abs();
            if dx > 1 || dy > 1 {
                continue;
            }
            match fields
                .groups_of_diff_pixels
                .iter_mut()
                .find(|group| group.contains(first))
            {
                Some(group) => {
                    group.add(second);
                }
                None => {
                    let mut group = PackedMap::default();
                    group.add(first);
                    group.add(second);
                    fields.groups_of_diff_pixels.push(group);
                }
            }
        }
    }
}

/// The last pixel the line test visits, or `None` when some pixel of the group has more
/// than two neighbours inside it (not a line).
fn line_end_pixel(group: &PackedMap<i32>, reference: &ImagePixMapData) -> Option<i32> {
    let mut last = None;
    for pixel in group.iter() {
        last = Some(pixel);
        let inside = count_neighbours(pixel, reference, |x, y| {
            Ok(group.contains(image_diff_pack_xy(x as u16, y as u16)))
        })
        .ok()?;
        if inside > 2 {
            return None;
        }
    }
    last
}

/// The neighbours of `pixel` inside the image for which `test` holds.
fn count_neighbours(
    pixel: i32,
    image: &ImagePixMapData,
    test: impl Fn(usize, usize) -> Result<bool, Failure>,
) -> Result<i32, Failure> {
    let mut count = 0;
    for (dx, dy) in NEIGHBOR_PIXELS {
        let x = i32::from(image_diff_unpack_x(pixel)) + dx;
        let y = i32::from(image_diff_unpack_y(pixel)) + dy;
        // size_t(-1) is beyond every size, as the native unsigned comparison.
        let (x, y) = (x as usize, y as usize);
        if x < image.size_x && y < image.size_y && test(x, y)? {
            count += 1;
        }
    }
    Ok(count)
}
