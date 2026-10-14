//! `Image_AlienPixMap`: an image read from and written to files through an image library.
//!
//! The oracle build has no image library (`USE_FREEIMAGE=OFF`, no WinCodec on Linux), so
//! the port is the branch that configuration compiles (visualization-design.md section 2):
//! loading fails with "Error: no image library available", saving writes binary PPM
//! (P6) whatever the extension, and gamma correction is unavailable.
use super::{ImageFormat, ImagePixMap};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::io::Write;
use std::ops::Deref;

/// An image backed by the (absent) image library; `ImageAlienPixMap::default().is_top_down()`.
pub struct ImageAlienPixMap {
    base: ImagePixMap,
}
impl Default for ImageAlienPixMap {
    /// An empty bottom-up image, the native default row order without a library.
    fn default() -> Self {
        let base = ImagePixMap::default();
        base.set_top_down(false);
        Self { base }
    }
}
impl Deref for ImageAlienPixMap {
    type Target = ImagePixMap;
    fn deref(&self) -> &ImagePixMap {
        &self.base
    }
}
rustycad_tkernel::impl_transient!(ImageAlienPixMap, "Image_AlienPixMap", ImagePixMap, base,
    interfaces:[dyn super::ImagePixMapSource]);

/// "Error: no image library available" to the default messenger.
fn no_library() {
    Message::send_fail_text(&AsciiString::from("Error: no image library available"));
}

/// `static_cast<uint8_t>(channel * 255.0)` on x86-64.
fn ppm_byte(channel: f64) -> u8 {
    crate::cast::int_from_f64(channel * 255.0) as u8
}

impl ImageAlienPixMap {
    /// False: the default row order of the configuration without image library.
    ///
    /// Example: `ImageAlienPixMap::is_top_down_default() == false`.
    pub fn is_top_down_default() -> bool {
        false
    }
    /// `Load(theFileName)`: always fails without an image library; `pixmap.load(&path)`.
    pub fn load(&self, _file_name: &AsciiString) -> bool {
        self.load_bytes(None, _file_name)
    }
    /// `Load(std::istream&, theFileName)`: clears and fails; `pixmap.load_stream(&mut s, &name)`.
    pub fn load_stream(&self, _stream: &mut dyn std::io::Read, _file_name: &AsciiString) -> bool {
        self.base.clear();
        no_library();
        false
    }
    /// `Load(theData, theLength, theFileName)`: clears and fails; `pixmap.load_bytes(None, &name)`.
    pub fn load_bytes(&self, _data: Option<&[u8]>, _file_name: &AsciiString) -> bool {
        self.base.clear();
        no_library();
        false
    }
    /// `Save(theFileName)`: writes binary PPM; `pixmap.save(&AsciiString::from("a.ppm"))?`.
    pub fn save(&self, file_name: &AsciiString) -> Result<bool, Failure> {
        self.save_bytes(None, file_name)
    }
    /// `Save(std::ostream&, theExtension)`: fails without an image library.
    ///
    /// Example: `pixmap.save_stream(&mut out, &AsciiString::from("png"))`.
    pub fn save_stream(&self, _stream: &mut dyn Write, _extension: &AsciiString) -> bool {
        no_library();
        false
    }
    /// `Save(theBuffer, theLength, theFileName)`: a buffer target fails; a file is written
    /// as PPM (a trace message tells when the name does not end in ".ppm"). Float pixels
    /// outside [0, 1] raise `Standard_OutOfRange`, as the colour constructor does natively.
    ///
    /// Example: `pixmap.save_bytes(None, &name)?`.
    pub fn save_bytes(
        &self,
        buffer: Option<&mut [u8]>,
        file_name: &AsciiString,
    ) -> Result<bool, Failure> {
        if buffer.is_some() {
            no_library();
            return Ok(false);
        }
        let bytes = file_name.as_bytes();
        let is_ppm = bytes.len() >= 4
            && bytes[bytes.len() - 4] == b'.'
            && bytes[bytes.len() - 3..].eq_ignore_ascii_case(b"ppm");
        if !is_ppm {
            let text = "Image_PixMap, no image library available! Image saved in PPM format";
            Message::send_text(
                &AsciiString::from(text),
                rustycad_tkernel::message::Gravity::Trace,
            );
        }
        self.save_ppm(file_name)
    }
    /// `savePPM`: the "P6" header, then the RGB bytes of every pixel, top row first.
    fn save_ppm(&self, file_name: &AsciiString) -> Result<bool, Failure> {
        if self.is_empty() {
            return Ok(false);
        }
        let mut contents = format!(
            "P6\n{} {}\n255\n",
            self.width() as i32,
            self.height() as i32
        )
        .into_bytes();
        for row in 0..self.height() {
            for col in 0..self.width() {
                let rgb = *self.pixel_color(col as i32, row as i32, false)?.rgb();
                contents.extend([rgb.red(), rgb.green(), rgb.blue()].map(ppm_byte));
            }
        }
        let path = String::from_utf8_lossy(file_name.as_bytes()).into_owned();
        Ok(std::fs::write(path, contents).is_ok())
    }
    /// `InitTrash`: as `Image_PixMap::InitTrash` without an image library.
    ///
    /// Example: `pixmap.init_trash(ImageFormat::Rgb, 4, 4, 0)`.
    pub fn init_trash(
        &self,
        format: ImageFormat,
        size_x: usize,
        size_y: usize,
        size_row_bytes: usize,
    ) -> bool {
        self.base.init_trash(format, size_x, size_y, size_row_bytes)
    }
    /// `InitCopy`: same format only (no conversion), copied in one block when the rows
    /// match, else row by row; a different format clears and fails.
    ///
    /// Example: `pixmap.init_copy(&other)`.
    pub fn init_copy(&self, copy: &ImagePixMap) -> bool {
        if std::ptr::eq(&self.base, copy) {
            return false;
        }
        let source = copy.data().clone();
        if !self.init_trash(
            source.format,
            source.data.size_x,
            source.data.size_y,
            source.data.size_row_bytes,
        ) {
            return false;
        }
        if self.format() != source.format {
            self.clear();
            return false;
        }
        let mut target = self.change_data();
        if target.data.size_row_bytes == source.data.size_row_bytes
            && target.data.top_to_down == source.data.top_to_down
        {
            let count = target.data.size().min(source.data.size());
            target.data.change_data()[..count].copy_from_slice(&source.data.data()[..count]);
            return true;
        }
        let row_bytes = target.data.size_row_bytes.min(source.data.size_row_bytes);
        for row in 0..target.data.size_y {
            if let (Some(to), Some(from)) = (target.data.change_row(row), source.data.row(row)) {
                to[..row_bytes].copy_from_slice(&from[..row_bytes]);
            }
        }
        true
    }
    /// `InitWrapper`: private natively, where it overrides the base member to forbid
    /// wrapping foreign memory: it clears and fails, shadowing `ImagePixMap::init_wrapper`.
    ///
    /// Example: `pixmap.init_wrapper(ImageFormat::Gray, None, 1, 1, 0) == false`.
    pub fn init_wrapper(
        &self,
        _format: ImageFormat,
        _bytes: Option<Vec<u8>>,
        _size_x: usize,
        _size_y: usize,
        _size_row_bytes: usize,
    ) -> bool {
        self.clear();
        false
    }
    /// `Clear`: releases the pixels (no library image to release).
    ///
    /// Example: `pixmap.clear()`.
    pub fn clear(&self) {
        self.base.clear();
    }
    /// `AdjustGamma`: false without an image library; `pixmap.adjust_gamma(2.2)`.
    pub fn adjust_gamma(&self, _gamma: f64) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_fails_and_saving_writes_ppm() {
        let pixmap = ImageAlienPixMap::default();
        assert!(!pixmap.is_top_down());
        assert!(!pixmap.load(&AsciiString::from("missing.png")));
        assert!(!pixmap.init_wrapper(ImageFormat::Gray, Some(vec![0]), 1, 1, 0));
        assert!(pixmap.init_trash(ImageFormat::Rgb, 2, 1, 0));
        pixmap
            .change_data()
            .data
            .change_data()
            .copy_from_slice(&[255, 0, 0, 0, 128, 255]);
        let path = std::env::temp_dir().join(format!("tkservice-alien-{}.ppm", std::process::id()));
        let name = AsciiString::from(path.to_string_lossy().as_ref());
        assert!(pixmap.save(&name).unwrap());
        let written = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(&written[..11], b"P6\n2 1\n255\n");
        assert_eq!(written.len(), 11 + 6);
        assert!(!pixmap.save_bytes(Some(&mut [0u8; 4]), &name).unwrap());
        assert!(!pixmap.adjust_gamma(2.2));
    }

    #[test]
    fn copies_need_the_same_format() {
        let source = ImagePixMap::default();
        source.init_zero(ImageFormat::Gray, 3, 2, 4, 7);
        let pixmap = ImageAlienPixMap::default();
        assert!(pixmap.init_copy(&source));
        assert_eq!(pixmap.data().data.data(), &[7; 8]);
    }
}
