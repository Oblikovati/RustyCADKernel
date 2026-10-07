//! Pixel formats (`Image_Format.hxx`, `Image_CompressedFormat.hxx`) and the format table of
//! `Image_PixMap.cxx` (name, component count, bytes per pixel).

/// This enumeration defines packed image plane formats
///
/// Native enumeration `Image_Format` (`Image_Format.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImageFormat {
    /// unsupported or unknown format.
    Unknown = 0,
    /// 1 byte per pixel, intensity of the color.
    Gray = 1,
    /// 1 byte per pixel, transparency.
    Alpha = 2,
    /// 3 bytes packed RGB image plane.
    Rgb = 3,
    /// same as RGB but with different components order.
    Bgr = 4,
    /// 4 bytes packed RGB image plane (1 extra byte for alignment, may have undefined value).
    Rgb32 = 5,
    /// same as RGB but with different components order.
    Bgr32 = 6,
    /// 4 bytes packed RGBA image plane.
    Rgba = 7,
    /// same as RGBA but with different components order.
    Bgra = 8,
    /// 1 float  (4-bytes) per pixel (1-component plane), intensity of the color.
    GrayF = 9,
    /// 1 float  (4-bytes) per pixel (1-component plane), transparency.
    AlphaF = 10,
    /// 2 floats (8-bytes) RG image plane.
    Rgf = 11,
    /// 3 floats (12-bytes) RGB image plane.
    Rgbf = 12,
    /// same as RGBF but with different components order.
    Bgrf = 13,
    /// 4 floats (16-bytes) RGBA image plane.
    Rgbaf = 14,
    /// same as RGBAF but with different components order.
    Bgraf = 15,
    /// 1 half-float  (2-bytes) intensity of color.
    GrayFHalf = 16,
    /// 2 half-floats (4-bytes) RG   image plane.
    RgfHalf = 17,
    /// 4 half-floats (8-bytes) RGBA image plane.
    RgbafHalf = 18,
    /// 2 bytes per pixel (unsigned short integer), intensity of the color.
    Gray16 = 19,
}

/// Number of uncompressed formats (`Image_Format_NB`).
pub const IMAGE_FORMAT_NB: i32 = ImageFormat::Gray16 as i32 + 1;

/// Compressed pixel formats, numbered after the uncompressed ones (`Image_CompressedFormat`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImageCompressedFormat {
    /// `Image_CompressedFormat_UNKNOWN` (= `Image_Format_UNKNOWN`).
    Unknown = 0,
    /// `Image_CompressedFormat_RGB_S3TC_DXT1` (= `Image_Format_NB`).
    RgbS3tcDxt1 = IMAGE_FORMAT_NB,
    /// `Image_CompressedFormat_RGBA_S3TC_DXT1`.
    RgbaS3tcDxt1 = IMAGE_FORMAT_NB + 1,
    /// `Image_CompressedFormat_RGBA_S3TC_DXT3`.
    RgbaS3tcDxt3 = IMAGE_FORMAT_NB + 2,
    /// `Image_CompressedFormat_RGBA_S3TC_DXT5`.
    RgbaS3tcDxt5 = IMAGE_FORMAT_NB + 3,
}

/// Number of formats, compressed ones included (`Image_CompressedFormat_NB`).
pub const IMAGE_COMPRESSED_FORMAT_NB: i32 = ImageCompressedFormat::RgbaS3tcDxt5 as i32 + 1;

/// `Image_FormatInfo`: name and bytes per pixel of one format (the native table also lists
/// the component count, which no member reads).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ImageFormatInfo {
    pub(crate) name: &'static str,
    pub(crate) pixel_size: u32,
}

const fn info(name: &'static str, _nb_components: u32, pixel_size: u32) -> ImageFormatInfo {
    ImageFormatInfo { name, pixel_size }
}

/// `Image_Table_ImageFormats`, indexed by the format value.
pub(crate) const IMAGE_FORMAT_TABLE: [ImageFormatInfo; IMAGE_COMPRESSED_FORMAT_NB as usize] = [
    info("UNKNOWN", 0, 1),
    info("Gray", 1, 1),
    info("Alpha", 1, 1),
    info("RGB", 3, 3),
    info("BGR", 3, 3),
    info("RGB32", 3, 4),
    info("BGR32", 3, 4),
    info("RGBA", 4, 4),
    info("BGRA", 4, 4),
    info("GrayF", 1, 4),
    info("AlphaF", 1, 4),
    info("RGF", 2, 8),
    info("RGBF", 3, 12),
    info("BGRF", 3, 12),
    info("RGBAF", 4, 16),
    info("BGRAF", 4, 16),
    info("GrayF_half", 1, 2),
    info("RGF_half", 2, 4),
    info("RGBAF_half", 4, 8),
    info("Gray16", 1, 2),
    // DXT1 uses circa half a byte per pixel, DXT3/5 circa 1 byte per pixel.
    info("RGB_S3TC_DXT1", 3, 1),
    info("RGBA_S3TC_DXT1", 4, 1),
    info("RGBA_S3TC_DXT3", 4, 1),
    info("RGBA_S3TC_DXT5", 4, 1),
];

impl ImageFormat {
    /// Every format in value order; `ImageFormat::ALL[3] == ImageFormat::Rgb`.
    pub const ALL: [ImageFormat; IMAGE_FORMAT_NB as usize] = [
        Self::Unknown,
        Self::Gray,
        Self::Alpha,
        Self::Rgb,
        Self::Bgr,
        Self::Rgb32,
        Self::Bgr32,
        Self::Rgba,
        Self::Bgra,
        Self::GrayF,
        Self::AlphaF,
        Self::Rgf,
        Self::Rgbf,
        Self::Bgrf,
        Self::Rgbaf,
        Self::Bgraf,
        Self::GrayFHalf,
        Self::RgfHalf,
        Self::RgbafHalf,
        Self::Gray16,
    ];
    /// The format of a value, `None` outside `[0, Image_Format_NB)`; `ImageFormat::from_value(3)`.
    pub fn from_value(value: i32) -> Option<Self> {
        usize::try_from(value)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
    }
    pub(crate) fn table_info(self) -> ImageFormatInfo {
        IMAGE_FORMAT_TABLE[self as usize]
    }
}

impl ImageCompressedFormat {
    /// The format of a value, `None` for other values; `ImageCompressedFormat::from_value(20)`.
    pub fn from_value(value: i32) -> Option<Self> {
        [
            Self::Unknown,
            Self::RgbS3tcDxt1,
            Self::RgbaS3tcDxt1,
            Self::RgbaS3tcDxt3,
            Self::RgbaS3tcDxt5,
        ]
        .into_iter()
        .find(|format| *format as i32 == value)
    }
    pub(crate) fn table_info(self) -> ImageFormatInfo {
        IMAGE_FORMAT_TABLE[self as usize]
    }
}
