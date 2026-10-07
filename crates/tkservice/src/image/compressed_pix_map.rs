//! `Image_CompressedPixMap` and `Image_SupportedFormats`: a compressed (S3TC) image and the
//! set of formats a graphic driver accepts.
use super::{IMAGE_COMPRESSED_FORMAT_NB, ImageCompressedFormat, ImageFormat};
use rustycad_tkernel::ncollection::Buffer;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// The protected fields of `Image_CompressedPixMap`.
#[derive(Clone)]
pub struct ImageCompressedPixMapFields {
    /// `myMipMaps`: the byte length of every mip level, base level first.
    pub mip_maps: Vec<i32>,
    /// `myFaceData`: the raw compressed data.
    pub face_data: Handle<Buffer>,
    /// `myFaceBytes`: the bytes of one face.
    pub face_bytes: usize,
    /// `myNbFaces`.
    pub nb_faces: i32,
    /// `mySizeX`.
    pub size_x: i32,
    /// `mySizeY`.
    pub size_y: i32,
    /// `myBaseFormat`: the uncompressed format.
    pub base_format: ImageFormat,
    /// `myFormat`.
    pub format: ImageCompressedFormat,
    /// `myIsCompleteMips`: the mip levels go down to 1x1.
    pub is_complete_mips: bool,
}

/// A compressed image; `ImageCompressedPixMap::default().is_top_down()`.
pub struct ImageCompressedPixMap {
    transient: StandardTransient,
    fields: SharedState<ImageCompressedPixMapFields>,
}
impl Default for ImageCompressedPixMap {
    /// Unknown formats, no data, no faces.
    fn default() -> Self {
        let fields = ImageCompressedPixMapFields {
            mip_maps: Vec::new(),
            face_data: Handle::default(),
            face_bytes: 0,
            nb_faces: 0,
            size_x: 0,
            size_y: 0,
            base_format: ImageFormat::Unknown,
            format: ImageCompressedFormat::Unknown,
            is_complete_mips: false,
        };
        Self {
            transient: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}
rustycad_tkernel::impl_transient!(
    ImageCompressedPixMap,
    "Image_CompressedPixMap",
    StandardTransient,
    transient
);

impl ImageCompressedPixMap {
    /// The protected fields, for reading; `pixmap.fields().mip_maps.len()`.
    pub fn fields(&self) -> RwLockReadGuard<'_, ImageCompressedPixMapFields> {
        self.fields.read()
    }
    /// The protected fields, for writing; `pixmap.change_fields().nb_faces = 6`.
    pub fn change_fields(&self) -> RwLockWriteGuard<'_, ImageCompressedPixMapFields> {
        self.fields.write()
    }
    /// The uncompressed format; `pixmap.base_format()`.
    pub fn base_format(&self) -> ImageFormat {
        self.fields.read().base_format
    }
    /// Sets the uncompressed format; `pixmap.set_base_format(ImageFormat::Rgba)`.
    pub fn set_base_format(&self, format: ImageFormat) {
        self.fields.write().base_format = format;
    }
    /// The compressed format; `pixmap.compressed_format()`.
    pub fn compressed_format(&self) -> ImageCompressedFormat {
        self.fields.read().format
    }
    /// Sets the compressed format; `pixmap.set_compressed_format(format)`.
    pub fn set_compressed_format(&self, format: ImageCompressedFormat) {
        self.fields.write().format = format;
    }
    /// The raw data; `pixmap.face_data()`.
    pub fn face_data(&self) -> Handle<Buffer> {
        self.fields.read().face_data.clone()
    }
    /// Sets the raw data; `pixmap.set_face_data(buffer)`.
    pub fn set_face_data(&self, buffer: Handle<Buffer>) {
        self.fields.write().face_data = buffer;
    }
    /// The mip level lengths (`MipMaps`); `pixmap.mip_maps()`.
    pub fn mip_maps(&self) -> Vec<i32> {
        self.fields.read().mip_maps.clone()
    }
    /// The mip level lengths, mutable (`ChangeMipMaps`); `pixmap.change_fields().mip_maps`.
    pub fn change_mip_maps(&self) -> RwLockWriteGuard<'_, ImageCompressedPixMapFields> {
        self.fields.write()
    }
    /// True when the mip levels go down to 1x1; `pixmap.is_complete_mip_map_set()`.
    pub fn is_complete_mip_map_set(&self) -> bool {
        self.fields.read().is_complete_mips
    }
    /// Sets the complete-mips flag; `pixmap.set_complete_mip_map_set(true)`.
    pub fn set_complete_mip_map_set(&self, is_complete: bool) {
        self.fields.write().is_complete_mips = is_complete;
    }
    /// The bytes of one face; `pixmap.face_bytes()`.
    pub fn face_bytes(&self) -> usize {
        self.fields.read().face_bytes
    }
    /// Sets the bytes of one face; `pixmap.set_face_bytes(64)`.
    pub fn set_face_bytes(&self, size: usize) {
        self.fields.write().face_bytes = size;
    }
    /// The width; `pixmap.size_x()`.
    pub fn size_x(&self) -> i32 {
        self.fields.read().size_x
    }
    /// The height; `pixmap.size_y()`.
    pub fn size_y(&self) -> i32 {
        self.fields.read().size_y
    }
    /// Sets the size; `pixmap.set_size(4, 4)`.
    pub fn set_size(&self, size_x: i32, size_y: i32) {
        let mut fields = self.fields.write();
        (fields.size_x, fields.size_y) = (size_x, size_y);
    }
    /// Always true: compressed images are top-down; `pixmap.is_top_down()`.
    pub fn is_top_down(&self) -> bool {
        true
    }
    /// The number of faces (6 for a cube map); `pixmap.nb_faces()`.
    pub fn nb_faces(&self) -> i32 {
        self.fields.read().nb_faces
    }
    /// Sets the number of faces; `pixmap.set_nb_faces(6)`.
    pub fn set_nb_faces(&self, count: i32) {
        self.fields.write().nb_faces = count;
    }
}

/// The formats a driver supports, uncompressed and compressed (`Image_SupportedFormats`).
///
/// Example: `let formats = ImageSupportedFormats::default(); formats.add(ImageFormat::Rgba);`.
pub struct ImageSupportedFormats {
    transient: StandardTransient,
    state: SharedState<(Vec<bool>, bool)>,
}
impl Default for ImageSupportedFormats {
    /// No format supported.
    fn default() -> Self {
        let formats = vec![false; IMAGE_COMPRESSED_FORMAT_NB as usize];
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new((formats, false)),
        }
    }
}
rustycad_tkernel::impl_transient!(
    ImageSupportedFormats,
    "Image_SupportedFormats",
    StandardTransient,
    transient
);

impl ImageSupportedFormats {
    /// True when the format is supported; `formats.is_supported(ImageFormat::Rgba)`.
    pub fn is_supported(&self, format: ImageFormat) -> bool {
        self.state.read().0[format as usize]
    }
    /// Marks a format supported; `formats.add(ImageFormat::Rgba)`.
    pub fn add(&self, format: ImageFormat) {
        self.state.write().0[format as usize] = true;
    }
    /// True when a compressed format was added; `formats.has_compressed()`.
    pub fn has_compressed(&self) -> bool {
        self.state.read().1
    }
    /// True when the compressed format is supported; `formats.is_supported_compressed(f)`.
    pub fn is_supported_compressed(&self, format: ImageCompressedFormat) -> bool {
        self.state.read().0[format as usize]
    }
    /// Marks a compressed format supported; `formats.add_compressed(f)`.
    pub fn add_compressed(&self, format: ImageCompressedFormat) {
        let mut state = self.state.write();
        state.0[format as usize] = true;
        state.1 = true;
    }
    /// Removes every format; `formats.clear()`.
    pub fn clear(&self) {
        let mut state = self.state.write();
        state.0.fill(false);
        state.1 = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_formats_track_the_compressed_flag() {
        let formats = ImageSupportedFormats::default();
        formats.add(ImageFormat::Rgba);
        assert!(formats.is_supported(ImageFormat::Rgba) && !formats.has_compressed());
        formats.add_compressed(ImageCompressedFormat::RgbaS3tcDxt5);
        assert!(formats.has_compressed());
        assert!(formats.is_supported_compressed(ImageCompressedFormat::RgbaS3tcDxt5));
        formats.clear();
        assert!(!formats.is_supported(ImageFormat::Rgba) && !formats.has_compressed());
    }

    #[test]
    fn compressed_pixmap_setters() {
        let pixmap = ImageCompressedPixMap::default();
        pixmap.set_size(8, 4);
        pixmap.set_nb_faces(6);
        pixmap.change_mip_maps().mip_maps = vec![32, 8];
        assert_eq!(
            (pixmap.size_x(), pixmap.size_y(), pixmap.nb_faces()),
            (8, 4, 6)
        );
        assert_eq!(pixmap.mip_maps(), vec![32, 8]);
        assert!(pixmap.is_top_down() && pixmap.face_data().is_null());
    }
}
