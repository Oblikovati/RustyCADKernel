//! `Graphic3d_MarkerImage`: a marker sprite given by a 1-bit bitmap or by color and alpha
//! images, converted lazily between the two forms.
use crate::cast::int_from_f64;
use crate::image::{ImageFormat, ImagePixMap};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::exceptions::{null_value, program_error};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::atomic::{AtomicI32, Ordering};

/// `THE_MARKER_IMAGE_COUNTER`: the source of the `Graphic3d_MarkerImage_<n>` ids.
static MARKER_IMAGE_COUNTER: AtomicI32 = AtomicI32::new(0);

/// The images, built lazily from the bitmap.
#[derive(Clone, Default)]
struct MarkerImages {
    image: Handle<ImagePixMap>,
    image_alpha: Handle<ImagePixMap>,
}

/// A marker sprite; `Graphic3dMarkerImage::from_bitmap(bitmap, 8, 8)`.
pub struct Graphic3dMarkerImage {
    transient: StandardTransient,
    pub(crate) ids: SharedState<(AsciiString, AsciiString)>,
    bit_map: Handle<HArray1<u8>>,
    images: SharedState<MarkerImages>,
    margin: i32,
    width: i32,
    height: i32,
}
rustycad_tkernel::impl_transient!(
    Graphic3dMarkerImage,
    "Graphic3d_MarkerImage",
    StandardTransient,
    transient
);

/// The image and alpha ids of a new counter value.
fn counter_ids() -> (AsciiString, AsciiString) {
    let number = MARKER_IMAGE_COUNTER
        .fetch_add(1, Ordering::SeqCst)
        .wrapping_add(1);
    let mut image = AsciiString::from("Graphic3d_MarkerImage_");
    image.assign_cat(number);
    let mut alpha = AsciiString::from("Graphic3d_MarkerImageAlpha_");
    alpha.assign_cat(number);
    (image, alpha)
}

/// The size of the image (natively dereferenced: a null image crashes), and the alpha checks.
fn checked_image_size(
    image: &Handle<ImagePixMap>,
    alpha: &Handle<ImagePixMap>,
) -> Result<(i32, i32), Failure> {
    let pixmap = image
        .get()
        .ok_or_else(|| null_value("Graphic3d_MarkerImage: null image; expected an image"))?;
    if let Some(alpha) = alpha.get() {
        if alpha.format() != ImageFormat::Alpha && alpha.format() != ImageFormat::Gray {
            return Err(program_error(&format!(
                "Graphic3d_MarkerImage, wrong color format of alpha image; got {:?}, expected Alpha or Gray",
                alpha.format()
            )));
        }
        if alpha.width() != pixmap.width() || alpha.height() != pixmap.height() {
            let got = (alpha.width(), alpha.height());
            return Err(program_error(&format!(
                "Graphic3d_MarkerImage, wrong dimensions of alpha image; got {got:?}, expected {:?}",
                (pixmap.width(), pixmap.height())
            )));
        }
    }
    Ok((pixmap.width() as i32, pixmap.height() as i32))
}

impl Graphic3dMarkerImage {
    fn assemble(
        ids: (AsciiString, AsciiString),
        bit_map: Handle<HArray1<u8>>,
        images: MarkerImages,
        (width, height): (i32, i32),
    ) -> Self {
        Self {
            transient: StandardTransient::default(),
            ids: SharedState::new(ids),
            bit_map,
            images: SharedState::new(images),
            margin: 1,
            width,
            height,
        }
    }
    /// A color image and an optional alpha image (Alpha or Gray, same size); a wrong alpha
    /// image raises `Standard_ProgramError`. The ids come from the counter, drawn before the
    /// checks as natively. Example: `Graphic3dMarkerImage::from_images(image, Handle::default())?`.
    pub fn from_images(
        image: Handle<ImagePixMap>,
        image_alpha: Handle<ImagePixMap>,
    ) -> Result<Self, Failure> {
        let ids = counter_ids();
        let size = checked_image_size(&image, &image_alpha)?;
        Ok(Self::assemble(
            ids,
            Handle::default(),
            MarkerImages { image, image_alpha },
            size,
        ))
    }
    /// The protected constructor with explicit ids; `Graphic3dMarkerImage::with_ids(id, alpha_id, image, alpha)?`.
    pub fn with_ids(
        id: AsciiString,
        alpha_id: AsciiString,
        image: Handle<ImagePixMap>,
        image_alpha: Handle<ImagePixMap>,
    ) -> Result<Self, Failure> {
        let size = checked_image_size(&image, &image_alpha)?;
        Ok(Self::assemble(
            (id, alpha_id),
            Handle::default(),
            MarkerImages { image, image_alpha },
            size,
        ))
    }
    /// A 1-bit bitmap of `width` x `height`, rows padded to whole bytes, top row first.
    /// Example: `Graphic3dMarkerImage::from_bitmap(bitmap, 8, 8)`.
    pub fn from_bitmap(bit_map: Handle<HArray1<u8>>, width: i32, height: i32) -> Self {
        Self::assemble(
            counter_ids(),
            bit_map,
            MarkerImages::default(),
            (width, height),
        )
    }
    /// The image id; `marker.get_image_id()`.
    pub fn get_image_id(&self) -> AsciiString {
        self.ids.read().0.clone()
    }
    /// The alpha image id; `marker.get_image_alpha_id()`.
    pub fn get_image_alpha_id(&self) -> AsciiString {
        self.ids.read().1.clone()
    }
    /// `GetTextureSize`: `(width, height)`; `marker.get_texture_size()`.
    pub fn get_texture_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }
    /// Whether the image has colors (not Alpha or Gray); `marker.is_colored_image()`.
    pub fn is_colored_image(&self) -> bool {
        let images = self.images.read();
        images.image.get().is_some_and(|image| {
            image.format() != ImageFormat::Alpha && image.format() != ImageFormat::Gray
        })
    }
}

/// `(width / 8)` rounded up: the bytes of one bitmap row.
pub(crate) fn bitmap_row_bytes(width: i32) -> i32 {
    width / 8 + i32::from(width % 8 != 0)
}

/// Lazy conversions.
impl Graphic3dMarkerImage {
    /// `GetBitMapArray`: the bitmap, or the image thresholded at `alpha_value` (red channel of
    /// a Gray image, alpha otherwise), bottom row first unless `is_top_down`.
    /// Example: `marker.get_bit_map_array(0.5, false)`.
    pub fn get_bit_map_array(&self, alpha_value: f64, is_top_down: bool) -> Handle<HArray1<u8>> {
        let image = self.images.read().image.clone();
        let Some(pixmap) = image.get().filter(|_| self.bit_map.is_null()) else {
            return self.bit_map.clone();
        };
        bytes_array(threshold_bits(pixmap, alpha_value, is_top_down))
    }
    /// `GetImage`: the image, or the bitmap drawn into a square Alpha image with a margin.
    /// Example: `marker.get_image()`.
    pub fn get_image(&self) -> Handle<ImagePixMap> {
        let current = self.images.read().image.clone();
        if current.is_some() || self.bit_map.is_null() {
            return current;
        }
        let image = Handle::new(self.bitmap_image());
        self.images.write().image = image.clone();
        image
    }
    /// The bitmap as an Alpha image of `max(width, height) + 2 * margin` pixels.
    fn bitmap_image(&self) -> ImagePixMap {
        let row_bytes = bitmap_row_bytes(self.width);
        let size = self.width.max(self.height);
        let (row_offset, column_offset) = (
            (size - self.height) / 2 + self.margin,
            (size - self.width) / 2 + self.margin,
        );
        let bits = self.bit_map.array1();
        let lower = bits.lower();
        let image = ImagePixMap::default();
        let side = (size + self.margin * 2) as usize;
        image.init_zero(ImageFormat::Alpha, side, side, 0, 0);
        let mut state = image.change_data();
        for row in 0..self.height {
            let Some(target) = state.data.change_row((row + row_offset) as usize) else {
                continue;
            };
            for column in 0..self.width {
                let byte = bits
                    .value(lower + row_bytes * row + column / 8)
                    .map_or(0, |byte| *byte);
                let bit_on = byte & (0x80u8 >> (column % 8)) != 0;
                target[(column + column_offset) as usize] = if bit_on { 255 } else { 0 };
            }
        }
        drop(state);
        image
    }
    /// `GetImageAlpha`: the alpha image, the image itself when Gray or Alpha, or the alpha
    /// channel of a color image (bottom-up rows). Example: `marker.get_image_alpha()`.
    pub fn get_image_alpha(&self) -> Handle<ImagePixMap> {
        let images = self.images.read().clone();
        if images.image_alpha.is_some() {
            return images.image_alpha;
        }
        let Some(image) = images.image.get() else {
            return Handle::default();
        };
        let alpha = if matches!(image.format(), ImageFormat::Gray | ImageFormat::Alpha) {
            images.image.clone()
        } else {
            Handle::new(alpha_channel(image))
        };
        self.images.write().image_alpha = alpha.clone();
        alpha
    }
}

/// The thresholded rows of `GetBitMapArray` (bit `0x80 >> (x % 8)` per pixel, added to the
/// byte as natively).
fn threshold_bits(pixmap: &ImagePixMap, alpha_value: f64, is_top_down: bool) -> Vec<u8> {
    let (width, height) = (pixmap.width() as i32, pixmap.height() as i32);
    let row_bytes = bitmap_row_bytes(width);
    let mut bytes = vec![0u8; (row_bytes * height).max(0) as usize];
    for row in 0..height {
        let target_row = if is_top_down { row } else { height - row - 1 };
        for column in (0..width).filter(|column| pixel_bit(pixmap, *column, row, alpha_value)) {
            let index = (row_bytes * target_row + column / 8) as usize;
            bytes[index] = bytes[index].wrapping_add(0x80u8 >> (column % 8));
        }
    }
    bytes
}

/// Whether a pixel is on for `GetBitMapArray`.
fn pixel_bit(image: &ImagePixMap, column: i32, row: i32, alpha_value: f64) -> bool {
    let Ok(color) = image.pixel_color(column, row, false) else {
        return false;
    };
    if image.format() == ImageFormat::Gray {
        return color.rgb().red() > alpha_value;
    }
    f64::from(color.alpha()) > alpha_value
}

/// The alpha channel of a color image as a bottom-up Alpha image.
fn alpha_channel(image: &ImagePixMap) -> ImagePixMap {
    let alpha = ImagePixMap::default();
    alpha.init_zero(ImageFormat::Alpha, image.width(), image.height(), 0, 0);
    alpha.set_top_down(false);
    for row in 0..image.height() {
        for column in 0..image.width() {
            let value = image
                .pixel_color(column as i32, row as i32, false)
                .map_or(0.0, |c| c.alpha());
            let byte = int_from_f64(255.0 * f64::from(value)) as u8;
            if let Some(target) = alpha.change_data().data.change_row(row) {
                target[column] = byte;
            }
        }
    }
    alpha
}

/// A zero-based byte array (`NCollection_HArray1<uint8_t>(0, n - 1)`).
pub(crate) fn bytes_array(bytes: Vec<u8>) -> Handle<HArray1<u8>> {
    let array = HArray1::<u8>::with_bounds(0, bytes.len() as i32 - 1).unwrap_or_default();
    array.change_array1().as_mut_slice().copy_from_slice(&bytes);
    Handle::new(array)
}
