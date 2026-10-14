//! `Image_PixMap`: a shared 2D or 3D pixel image of one pixel format.
//!
//! The transient keeps its buffer and format in one lock: [`ImagePixMap::data`] and
//! [`ImagePixMap::change_data`] give the low-level row and pixel accessors of
//! [`ImagePixMapData`] (the native `Row`, `Value`, `ChangeValueXYZ`... members).
use super::pix_map_data::ImagePixMapData;
use super::pixel_convert::{image_color_from_raw_pixel, image_color_to_raw_pixel};
use super::{ImageCompressedFormat, ImageFormat};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::quantity::{Color, ColorRgba};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// The locked fields: the buffer (`myData`) and the format (`myImgFormat`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImagePixMapState {
    /// `myData`.
    pub data: ImagePixMapData,
    /// `myImgFormat`.
    pub format: ImageFormat,
}
impl Default for ImageFormat {
    /// `Image_Format_Gray`, the format of a new `Image_PixMap`.
    fn default() -> Self {
        ImageFormat::Gray
    }
}

/// A pixel image shared by handles; `ImagePixMap::default().init_trash(ImageFormat::Rgba, 4, 4, 0)`.
pub struct ImagePixMap {
    transient: StandardTransient,
    state: SharedState<ImagePixMapState>,
}
impl Default for ImagePixMap {
    /// An empty gray image, as `Image_PixMap()`.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(ImagePixMapState::default()),
        }
    }
}
rustycad_tkernel::impl_transient!(ImagePixMap, "Image_PixMap", StandardTransient, transient,
    interfaces:[dyn super::ImagePixMapSource]);

impl ImagePixMap {
    /// True on a big-endian host; `ImagePixMap::is_big_endian_host()`.
    pub fn is_big_endian_host() -> bool {
        cfg!(target_endian = "big")
    }
    /// Bytes per pixel of a format; `ImagePixMap::size_pixel_bytes_of(ImageFormat::Rgb) == 3`.
    pub fn size_pixel_bytes_of(format: ImageFormat) -> usize {
        format.table_info().pixel_size as usize
    }
    /// The name of a format; `ImagePixMap::image_format_to_string(ImageFormat::Rgba) == "RGBA"`.
    pub fn image_format_to_string(format: ImageFormat) -> &'static str {
        format.table_info().name
    }
    /// The name of a compressed format; `image_compressed_format_to_string(..)`.
    pub fn image_compressed_format_to_string(format: ImageCompressedFormat) -> &'static str {
        format.table_info().name
    }
    /// The locked buffer and format, for reading; `pixmap.data().data.row(0)`.
    pub fn data(&self) -> RwLockReadGuard<'_, ImagePixMapState> {
        self.state.read()
    }
    /// The locked buffer and format, for writing; `pixmap.change_data().data.change_row(0)`.
    pub fn change_data(&self) -> RwLockWriteGuard<'_, ImagePixMapState> {
        self.state.write()
    }
    /// The pixel format; `pixmap.format()`.
    pub fn format(&self) -> ImageFormat {
        self.state.read().format
    }
    /// Changes the format; `Standard_ProgramError` when the image is not empty and the
    /// pixel sizes differ. Example: `pixmap.set_format(ImageFormat::Bgra)?`.
    pub fn set_format(&self, format: ImageFormat) -> Result<(), Failure> {
        let mut state = self.state.write();
        if state.format == format {
            return Ok(());
        }
        let incompatible =
            Self::size_pixel_bytes_of(state.format) != Self::size_pixel_bytes_of(format);
        if !state.data.is_empty() && incompatible {
            return Err(program_error(&format!(
                "Image_PixMap::SetFormat() - incompatible pixel format {format:?} for {:?}",
                state.format
            )));
        }
        state.format = format;
        Ok(())
    }
    /// The width; `pixmap.width()` (also `SizeX`).
    pub fn width(&self) -> usize {
        self.state.read().data.size_x
    }
    /// The height; `pixmap.height()` (also `SizeY`).
    pub fn height(&self) -> usize {
        self.state.read().data.size_y
    }
    /// The depth; `pixmap.depth()` (also `SizeZ`).
    pub fn depth(&self) -> usize {
        self.state.read().data.size_z
    }
    /// `(SizeX, SizeY, SizeZ)`; `pixmap.size_xyz()`.
    pub fn size_xyz(&self) -> Vec3<usize> {
        let data = &self.state.read().data;
        Vec3::new(data.size_x, data.size_y, data.size_z)
    }
    /// Width over height, 1 for an empty height; `pixmap.ratio()`.
    pub fn ratio(&self) -> f64 {
        let data = &self.state.read().data;
        if data.size_y > 0 {
            data.size_x as f64 / data.size_y as f64
        } else {
            1.0
        }
    }
    /// True for a null buffer; `pixmap.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.state.read().data.is_empty()
    }
    /// The color of a pixel, transparent black outside the image; out-of-range float
    /// channels raise `Standard_OutOfRange` natively. Example: `pixmap.pixel_color(0, 0, false)?`.
    pub fn pixel_color(&self, x: i32, y: i32, to_linearize: bool) -> Result<ColorRgba, Failure> {
        let state = self.state.read();
        let Some(raw) = pixel_xy(&state.data, x, y).and_then(|(x, y)| state.data.value_xy(x, y))
        else {
            return ColorRgba::new(0.0, 0.0, 0.0, 0.0);
        };
        image_color_from_raw_pixel(raw, state.format, to_linearize)
    }
    /// `SetPixelColor(x, y, const Quantity_Color&)`: an opaque color; `pixmap.set_pixel_color(0, 0, red, false)`.
    pub fn set_pixel_color(&self, x: i32, y: i32, color: Color, to_delinearize: bool) {
        self.set_pixel_color_rgba(x, y, &ColorRgba::from_rgb(color, 1.0), to_delinearize);
    }
    /// `SetPixelColor(x, y, const Quantity_ColorRGBA&)`; ignored outside the image.
    ///
    /// Example: `pixmap.set_pixel_color_rgba(0, 0, &rgba, false)`.
    pub fn set_pixel_color_rgba(&self, x: i32, y: i32, color: &ColorRgba, to_delinearize: bool) {
        let mut state = self.state.write();
        let format = state.format;
        let Some((x, y)) = pixel_xy(&state.data, x, y) else {
            return;
        };
        if let Some(raw) = state.data.change_value_xy(x, y) {
            image_color_to_raw_pixel(raw, format, color, to_delinearize);
        }
    }
}

/// The pixel of `(x, y)` when the image is not empty and the point lies inside it.
fn pixel_xy(data: &ImagePixMapData, x: i32, y: i32) -> Option<(usize, usize)> {
    let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
    (!data.is_empty() && x < data.size_x && y < data.size_y).then_some((x, y))
}
