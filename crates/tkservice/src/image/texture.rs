//! `Image_Texture`: a texture image defined by a file, a file range or a memory buffer.
//!
//! The class has virtual members (exchange formats derive from it), so it is the trait
//! [`ImageTextureSource`] (virtuals with their native bodies) over the embedded
//! [`ImageTextureBase`]; [`ImageTexture`] is the native class itself.
use super::texture_io::{copy_file_range, fail, open_input, probe_format, read_probe};
use super::{
    ImageAlienPixMap, ImageCompressedPixMap, ImageDdsParser, ImagePixMap, ImageSupportedFormats,
};
use crate::json_dump::{class_begin, field_dumped, field_real, field_string};
use rustycad_tkernel::ncollection::{Buffer, HashPolicy};
use rustycad_tkernel::osd::SeekDir;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, StandardTransient, Transient};
use rustycad_tkernel::tcollection::AsciiString;
use std::io::{self, Write};

/// The protected fields of `Image_Texture`.
#[derive(Clone, Default)]
pub struct ImageTextureBase {
    /// `myTextureId`: the generated id ("texture://path", "texturebuf://id").
    pub texture_id: AsciiString,
    /// `myImagePath`.
    pub image_path: AsciiString,
    /// `myBuffer`: the image file bytes in memory.
    pub buffer: Handle<Buffer>,
    /// `myOffset`: the offset within the file, -1 for none.
    pub offset: i64,
    /// `myLength`: the length within the file, -1 for none.
    pub length: i64,
}

impl ImageTextureBase {
    /// `Image_Texture(theFileName)`; `ImageTextureBase::from_file(&path)`.
    pub fn from_file(file_name: &AsciiString) -> Self {
        let mut base = Self {
            image_path: file_name.clone(),
            offset: -1,
            length: -1,
            ..Default::default()
        };
        if !file_name.as_bytes().is_empty() {
            base.texture_id = AsciiString::from("texture://");
            base.texture_id.append(file_name);
        }
        base
    }
    /// `Image_Texture(theFileName, theOffset, theLength)`; the id gets ";offset,length".
    ///
    /// Example: `ImageTextureBase::from_file_range(&path, 16, 512)`.
    pub fn from_file_range(file_name: &AsciiString, offset: i64, length: i64) -> Self {
        let mut base = Self {
            image_path: file_name.clone(),
            offset,
            length,
            ..Default::default()
        };
        if !file_name.as_bytes().is_empty() {
            base.texture_id = AsciiString::from("texture://");
            base.texture_id.append(file_name);
            base.texture_id
                .append(&AsciiString::from(format!(";{offset},{length}").as_str()));
        }
        base
    }
    /// `Image_Texture(theBuffer, theId)`; `ImageTextureBase::from_buffer(buffer, &id)`.
    pub fn from_buffer(buffer: Handle<Buffer>, id: &AsciiString) -> Self {
        let mut base = Self {
            buffer,
            offset: -1,
            length: -1,
            ..Default::default()
        };
        if !id.as_bytes().is_empty() {
            base.texture_id = AsciiString::from("texturebuf://");
            base.texture_id.append(id);
        }
        base
    }
}

/// A texture image source; implementers return their embedded base.
pub trait ImageTextureSource: Transient {
    /// The embedded base; `texture.texture_base().texture_id`.
    fn texture_base(&self) -> &ImageTextureBase;
    /// The compressed (DDS) image when `supported` accepts compressed formats.
    ///
    /// Example: `texture.read_compressed_image(&formats)`.
    fn read_compressed_image(
        &self,
        supported: &Handle<ImageSupportedFormats>,
    ) -> Handle<ImageCompressedPixMap> {
        let base = self.texture_base();
        if !supported
            .get()
            .is_some_and(ImageSupportedFormats::has_compressed)
        {
            return Handle::default();
        }
        if base.buffer.is_some() {
            return ImageDdsParser::load_buffer(supported, &base.buffer, 0);
        }
        if base.offset >= 0 {
            return ImageDdsParser::load(supported, &base.image_path, 0, base.offset);
        }
        if !base
            .image_path
            .as_bytes()
            .to_ascii_lowercase()
            .ends_with(b".dds")
        {
            return Handle::default();
        }
        ImageDdsParser::load(supported, &base.image_path, 0, 0)
    }
    /// The decoded image (always null without an image library); `texture.read_image(&f)`.
    fn read_image(&self, _supported: &Handle<ImageSupportedFormats>) -> Handle<ImagePixMap> {
        let base = self.texture_base();
        if base.buffer.is_some() {
            return self.load_image_buffer(&base.buffer, &base.texture_id);
        }
        if base.offset >= 0 {
            return self.load_image_offset(&base.image_path, base.offset, base.length);
        }
        self.load_image_file(&base.image_path)
    }
    /// Copies the image file (or buffer) to `file`; `texture.write_image(&path)`.
    fn write_image(&self, file: &AsciiString) -> bool {
        let path = String::from_utf8_lossy(file.as_bytes()).into_owned();
        let Ok(mut output) = std::fs::File::create(&path) else {
            fail(&[b"Error: Unable to create file '", file.as_bytes(), b"'"]);
            return false;
        };
        if !self.write_image_stream(&mut output, file) {
            return false;
        }
        if output.flush().is_err() {
            fail(&[b"Error: Unable to write file '", file.as_bytes(), b"'"]);
            return false;
        }
        true
    }
    /// Writes the image bytes to a stream; `texture.write_image_stream(&mut out, &name)`.
    fn write_image_stream(&self, stream: &mut dyn Write, file: &AsciiString) -> bool {
        let base = self.texture_base();
        if let Some(buffer) = base.buffer.get() {
            if stream.write_all(buffer.as_slice()).is_err() {
                fail(&[b"File '", file.as_bytes(), b"' cannot be written"]);
                return false;
            }
            return true;
        }
        copy_file_range(base, stream, file)
    }
    /// `DumpJson`: id, path, buffer, offset and length; `texture.dump_json(&mut s, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let base = self.texture_base();
        class_begin(stream, "Image_Texture")?;
        field_string(stream, "myTextureId", base.texture_id.as_bytes())?;
        field_string(stream, "myImagePath", base.image_path.as_bytes())?;
        if let Some(buffer) = base.buffer.get() {
            field_dumped(stream, depth, "myBuffer.get()", &|s, d| {
                buffer.dump_json(s, d)
            })?;
        }
        field_real(stream, "myOffset", base.offset as f64)?;
        field_real(stream, "myLength", base.length as f64)
    }
    /// Loads an image file (fails without an image library); `texture.load_image_file(&p)`.
    fn load_image_file(&self, path: &AsciiString) -> Handle<ImagePixMap> {
        let image = ImageAlienPixMap::default();
        if !image.load(path) {
            return Handle::default();
        }
        Handle::new(image).cast()
    }
    /// Loads an image from a file range; `texture.load_image_offset(&p, 16, 512)`.
    fn load_image_offset(
        &self,
        path: &AsciiString,
        offset: i64,
        length: i64,
    ) -> Handle<ImagePixMap> {
        if length > i64::from(i32::MAX) {
            fail(&[
                b"Error: Image file size is too big '",
                path.as_bytes(),
                b"'",
            ]);
            return Handle::default();
        }
        let Some(stream) = open_input(path) else {
            fail(&[
                b"Error: Image file '",
                path.as_bytes(),
                b"' cannot be opened",
            ]);
            return Handle::default();
        };
        let mut stream = stream
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stream.seekg(offset, SeekDir::Beg);
        if !stream.good() {
            fail(&[
                b"Error: Image is defined with invalid file offset '",
                path.as_bytes(),
                b"'",
            ]);
            return Handle::default();
        }
        let image = ImageAlienPixMap::default();
        if !image.load_stream(&mut io::empty(), path) {
            return Handle::default();
        }
        Handle::new(image).cast()
    }
    /// Loads an image from memory; `texture.load_image_buffer(&buffer, &id)`.
    fn load_image_buffer(&self, buffer: &Handle<Buffer>, id: &AsciiString) -> Handle<ImagePixMap> {
        let Some(bytes) = buffer.get().map(Buffer::as_slice) else {
            return Handle::default();
        };
        if bytes.len() > i32::MAX as usize {
            fail(&[b"Error: Image file size is too big '", id.as_bytes(), b"'"]);
            return Handle::default();
        }
        let image = ImageAlienPixMap::default();
        if !image.load_bytes(Some(bytes), id) {
            return Handle::default();
        }
        Handle::new(image).cast()
    }
}

/// The non-virtual members of `Image_Texture`.
pub trait ImageTextureMembers: ImageTextureSource {
    /// The generated id; `texture.texture_id()`.
    fn texture_id(&self) -> AsciiString {
        self.texture_base().texture_id.clone()
    }
    /// The image file path; `texture.file_path()`.
    fn file_path(&self) -> AsciiString {
        self.texture_base().image_path.clone()
    }
    /// The offset within the file, -1 for none; `texture.file_offset()`.
    fn file_offset(&self) -> i64 {
        self.texture_base().offset
    }
    /// The length within the file, -1 for none; `texture.file_length()`.
    fn file_length(&self) -> i64 {
        self.texture_base().length
    }
    /// The image bytes in memory; `texture.data_buffer()`.
    fn data_buffer(&self) -> Handle<Buffer> {
        self.texture_base().buffer.clone()
    }
    /// The MIME type of the probed format, empty when unknown; `texture.mime_type()`.
    fn mime_type(&self) -> AsciiString {
        let kind = self.probe_image_file_format();
        let kind = String::from_utf8_lossy(kind.as_bytes()).into_owned();
        let mime = match kind.as_str() {
            "jpg" => "image/jpeg".to_string(),
            "png" | "bmp" | "webp" | "gif" | "tiff" => format!("image/{kind}"),
            "dds" => "image/vnd-ms.dds".to_string(),
            "" => String::new(),
            other => format!("image/x-{other}"),
        };
        AsciiString::from(mime.as_str())
    }
    /// The format of the first 20 bytes ("png", "jpg", "gif", "tiff", "bmp", "webp",
    /// "dds" or ""). Native quirks kept: a short buffer is compared with uninitialised
    /// bytes natively (zeros here), and a file error returns "0" (`return false` converted
    /// to a string). Example: `texture.probe_image_file_format()`.
    fn probe_image_file_format(&self) -> AsciiString {
        let mut header = [0u8; 20];
        let base = self.texture_base();
        if let Some(buffer) = base.buffer.get() {
            let bytes = buffer.as_slice();
            let count = bytes.len().min(20);
            header[..count].copy_from_slice(&bytes[..count]);
        } else if let Err(text) = read_probe(base, &mut header) {
            fail(text.as_slice());
            return AsciiString::from("0");
        }
        AsciiString::from(probe_format(&header))
    }
}
impl<T: ImageTextureSource + ?Sized> ImageTextureMembers for T {}

/// The native `Image_Texture`; `ImageTexture::new(&AsciiString::from("wood.png"))`.
pub struct ImageTexture {
    transient: StandardTransient,
    base: ImageTextureBase,
}
rustycad_tkernel::impl_transient!(ImageTexture, "Image_Texture", dyn ImageTextureSource, transient,
    interfaces: [dyn ImageTextureSource]);
rustycad_tkernel::impl_transient_interface!(
    dyn ImageTextureSource,
    "Image_Texture",
    StandardTransient
);

impl ImageTexture {
    /// A texture of an image file; `ImageTexture::new(&path)`.
    pub fn new(file_name: &AsciiString) -> Self {
        Self::with_base(ImageTextureBase::from_file(file_name))
    }
    /// A texture of a file range; `ImageTexture::with_range(&path, 16, 512)`.
    pub fn with_range(file_name: &AsciiString, offset: i64, length: i64) -> Self {
        Self::with_base(ImageTextureBase::from_file_range(file_name, offset, length))
    }
    /// A texture of image bytes in memory; `ImageTexture::with_buffer(buffer, &id)`.
    pub fn with_buffer(buffer: Handle<Buffer>, id: &AsciiString) -> Self {
        Self::with_base(ImageTextureBase::from_buffer(buffer, id))
    }
    fn with_base(base: ImageTextureBase) -> Self {
        Self {
            transient: StandardTransient::default(),
            base,
        }
    }
}
impl ImageTextureSource for ImageTexture {
    fn texture_base(&self) -> &ImageTextureBase {
        &self.base
    }
}

/// `std::hash` and `std::equal_to` of `occ::handle<Image_Texture>`: by texture id, null
/// handles hashing to 0 and equal only to each other.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImageTextureHashPolicy;
impl HashPolicy<Handle<dyn ImageTextureSource>> for ImageTextureHashPolicy {
    fn hash(&self, key: &Handle<dyn ImageTextureSource>) -> usize {
        key.get()
            .map_or(0, |texture| texture.texture_id().hash_code())
    }
    fn equal(
        &self,
        first: &Handle<dyn ImageTextureSource>,
        second: &Handle<dyn ImageTextureSource>,
    ) -> bool {
        match (first.get(), second.get()) {
            (None, None) => true,
            (Some(a), Some(b)) => a.texture_id() == b.texture_id(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_follow_the_source() {
        let path = AsciiString::from("/tmp/a.png");
        assert_eq!(
            ImageTexture::new(&path).texture_id(),
            AsciiString::from("texture:///tmp/a.png")
        );
        let ranged = ImageTexture::with_range(&path, 16, -1);
        assert_eq!(
            ranged.texture_id(),
            AsciiString::from("texture:///tmp/a.png;16,-1")
        );
        assert_eq!(
            ImageTexture::new(&AsciiString::default()).texture_id(),
            AsciiString::default()
        );
        let mut buffer = Buffer::new(
            Some(rustycad_tkernel::ncollection::BaseAllocator::common()),
            8,
        )
        .unwrap();
        buffer.as_mut_slice().copy_from_slice(b"GIF89a\0\0");
        let texture = ImageTexture::with_buffer(Handle::new(buffer), &AsciiString::from("id"));
        assert_eq!(texture.texture_id(), AsciiString::from("texturebuf://id"));
        assert_eq!(texture.mime_type(), AsciiString::from("image/gif"));
        assert!(texture.read_image(&Handle::default()).is_null());
        let mut out = Vec::new();
        assert!(texture.write_image_stream(&mut out, &AsciiString::from("x")) && out.len() == 8);
    }

    #[test]
    fn probing_a_missing_file_returns_zero_text() {
        let texture = ImageTexture::new(&AsciiString::from("/nonexistent/x.png"));
        assert_eq!(texture.probe_image_file_format(), AsciiString::from("0"));
        let first: Handle<dyn ImageTextureSource> =
            Handle::new(ImageTexture::new(&AsciiString::from("a"))).cast();
        let second: Handle<dyn ImageTextureSource> =
            Handle::new(ImageTexture::new(&AsciiString::from("a"))).cast();
        assert!(ImageTextureHashPolicy.equal(&first, &second));
        assert!(!ImageTextureHashPolicy.equal(&first, &Handle::default()));
    }
}
