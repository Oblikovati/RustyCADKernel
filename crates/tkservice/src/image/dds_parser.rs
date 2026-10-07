//! `Image_DDSParser`: reads the S3TC (DXT1/3/5) faces of a DirectDraw Surface.
use super::{ImageCompressedFormat, ImageCompressedPixMap, ImageFormat, ImageSupportedFormats};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
use rustycad_tkernel::osd::{IosOpenMode, SeekDir, default_file_system};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;

/// Size of the "DDS " magic plus the 124-byte header.
const DDS_HEADER_BYTES: usize = 128;

/// The header fields `parseHeader` reads (`DDSFileHeader`), native-endian at their offsets.
struct DdsHeader<'a>(&'a [u8]);
impl DdsHeader<'_> {
    fn word(&self, index: usize) -> u32 {
        u32::from_ne_bytes(
            self.0[4 * index..4 * index + 4]
                .try_into()
                .expect("header word"),
        )
    }
    fn four_cc(&self) -> &[u8] {
        &self.0[80..84]
    }
}

/// The formats and block size of a FourCC, `None` for an unsupported one.
fn dds_formats(
    four_cc: &[u8],
    has_alpha: bool,
) -> Option<(ImageFormat, ImageCompressedFormat, i32)> {
    use ImageCompressedFormat as C;
    match four_cc {
        b"DXT5" => Some((ImageFormat::Rgba, C::RgbaS3tcDxt5, 16)),
        b"DXT3" => Some((ImageFormat::Rgba, C::RgbaS3tcDxt3, 16)),
        b"DXT1" if has_alpha => Some((ImageFormat::Rgba, C::RgbaS3tcDxt1, 8)),
        b"DXT1" => Some((ImageFormat::Rgb, C::RgbS3tcDxt1, 8)),
        _ => None,
    }
}

/// `Image_DDSParser`: the two `Load` entry points; `ImageDdsParser::load_buffer(&f, &b, 0)`.
pub struct ImageDdsParser;

impl ImageDdsParser {
    /// `parseHeader`: the definition (no face data) of the 124-byte header after the magic.
    fn parse_header(bytes: &[u8]) -> Handle<ImageCompressedPixMap> {
        let header = DdsHeader(bytes);
        // Size, Height, Width, MipMapCount, PixelFormat.Size/Flags and Caps2 by word index.
        let (size, height, width, mip_count) = (
            header.word(0),
            header.word(2),
            header.word(3),
            header.word(6),
        );
        if size != 124 || width == 0 || height == 0 || header.word(18) != 32 {
            return Handle::default();
        }
        let has_alpha = header.word(19) & 0x1 != 0;
        let Some((base, format, block)) = dds_formats(header.four_cc(), has_alpha) else {
            return Handle::default();
        };
        let definition = ImageCompressedPixMap::default();
        definition.set_size(width as i32, height as i32);
        let is_cube = header.word(27) & 0xFE00 == 0xFE00;
        definition.set_nb_faces(if is_cube { 6 } else { 1 });
        definition.set_base_format(base);
        definition.set_compressed_format(format);
        fill_mip_maps(&definition, (mip_count as i32).max(1), block);
        Handle::new(definition)
    }
    /// The definition, or with `face_index >= 0` also that face's data, of a DDS file at
    /// `file_offset`; null for a missing file, a bad header, an unsupported format or a
    /// face that cannot be read (the last two are reported to the default messenger).
    ///
    /// Example: `ImageDdsParser::load(&supported, &path, 0, 0)`.
    pub fn load(
        supported: &Handle<ImageSupportedFormats>,
        file: &AsciiString,
        face_index: i32,
        file_offset: i64,
    ) -> Handle<ImageCompressedPixMap> {
        let mode = IosOpenMode::IN | IosOpenMode::BINARY;
        let Ok(Some(stream)) = default_file_system().open_istream(file, mode, 0, None) else {
            return Handle::default();
        };
        let mut stream = stream
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !stream.good() {
            return Handle::default();
        }
        if file_offset != 0 {
            stream.seekg(file_offset, SeekDir::Beg);
        }
        let mut header = [0u8; DDS_HEADER_BYTES];
        let read = stream.read(&mut header);
        let Some(definition) = Self::checked_definition(supported, &header[..read]) else {
            return Handle::default();
        };
        if face_index < 0 {
            return definition;
        }
        let name = String::from_utf8_lossy(file.as_bytes()).into_owned();
        let Some(face_bytes) =
            Self::face_size(&definition, face_index, &format!("within file\n{name}"))
        else {
            return Handle::default();
        };
        let offset = face_bytes * face_index as usize;
        if offset != 0 {
            stream.seekg(offset as i64, SeekDir::Cur);
        }
        let mut face = vec![0u8; face_bytes];
        if stream.read(&mut face) < face_bytes {
            fail(&format!(
                "DDS Reader error - unable to read face #{face_index} data from file\n{name}"
            ));
            return Handle::default();
        }
        Self::with_face(definition, &face)
    }
    /// The definition, or with `face_index >= 0` also that face's data, of a DDS image in
    /// memory. Example: `ImageDdsParser::load_buffer(&supported, &buffer, -1)`.
    pub fn load_buffer(
        supported: &Handle<ImageSupportedFormats>,
        buffer: &Handle<Buffer>,
        face_index: i32,
    ) -> Handle<ImageCompressedPixMap> {
        let Some(bytes) = buffer.get().map(Buffer::as_slice) else {
            return Handle::default();
        };
        let Some(definition) = Self::checked_definition(supported, bytes) else {
            return Handle::default();
        };
        if face_index < 0 {
            return definition;
        }
        let Some(face_bytes) = Self::face_size(&definition, face_index, "within buffer") else {
            return Handle::default();
        };
        let offset = face_bytes * face_index as usize + DDS_HEADER_BYTES;
        let Some(face) = bytes.get(offset..offset + face_bytes) else {
            fail(&format!(
                "DDS Reader error - unable to read face #{face_index} data from buffer"
            ));
            return Handle::default();
        };
        Self::with_face(definition, face)
    }
    /// Checks the magic, parses the header and checks the format against `supported`.
    fn checked_definition(
        supported: &Handle<ImageSupportedFormats>,
        bytes: &[u8],
    ) -> Option<Handle<ImageCompressedPixMap>> {
        if bytes.len() < DDS_HEADER_BYTES || &bytes[..4] != b"DDS " {
            return None;
        }
        let definition = Self::parse_header(&bytes[4..DDS_HEADER_BYTES]);
        let format = definition.get()?.compressed_format();
        let rejected = supported
            .get()
            .is_some_and(|formats| !formats.is_supported_compressed(format));
        (!rejected).then_some(definition)
    }
    /// The face size, or `None` (reported) for an invalid face index.
    fn face_size(
        definition: &Handle<ImageCompressedPixMap>,
        face_index: i32,
        place: &str,
    ) -> Option<usize> {
        let definition = definition.get()?;
        if face_index >= definition.nb_faces() || definition.face_bytes() == 0 {
            fail(&format!(
                "DDS Reader error - invalid face index #{face_index} {place}"
            ));
            return None;
        }
        Some(definition.face_bytes())
    }
    /// Attaches a copy of the face bytes in a 16-byte aligned buffer.
    fn with_face(
        definition: Handle<ImageCompressedPixMap>,
        face: &[u8],
    ) -> Handle<ImageCompressedPixMap> {
        let Ok(mut buffer) = Buffer::new(Some(BaseAllocator::aligned(16)), face.len()) else {
            return Handle::default();
        };
        buffer.as_mut_slice()[..face.len()].copy_from_slice(face);
        if let Some(pixmap) = definition.get() {
            pixmap.set_face_data(Handle::new(buffer));
        }
        definition
    }
}

/// The mip level lengths of `count` levels in `block`-byte 4x4 blocks, the face size, and
/// whether the last level is 1x1.
fn fill_mip_maps(definition: &ImageCompressedPixMap, count: i32, block: i32) {
    let (mut x, mut y) = (definition.size_x(), definition.size_y());
    let mut lengths = Vec::with_capacity(count as usize);
    let mut face_size = 0usize;
    loop {
        let length = ((x + 3) / 4) * ((y + 3) / 4) * block;
        face_size = face_size.wrapping_add(length as usize);
        lengths.push(length);
        if lengths.len() as i32 >= count {
            break;
        }
        (x, y) = ((x / 2).max(1), (y / 2).max(1));
    }
    let mut fields = definition.change_fields();
    fields.mip_maps = lengths;
    fields.is_complete_mips = x == 1 && y == 1;
    fields.face_bytes = face_size;
}

/// `Message::SendFail` of one line.
fn fail(text: &str) {
    Message::send_fail_text(&AsciiString::from(text));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A DDS image: magic, header and `faces` of zero bytes.
    fn dds(four_cc: &[u8; 4], width: u32, height: u32, mips: u32, extra: usize) -> Vec<u8> {
        let mut words = [0u32; 31];
        (words[0], words[2], words[3], words[6], words[18]) = (124, height, width, mips, 32);
        words[20] = u32::from_ne_bytes(*four_cc);
        let mut bytes = b"DDS ".to_vec();
        words
            .iter()
            .for_each(|w| bytes.extend_from_slice(&w.to_ne_bytes()));
        bytes.resize(128 + extra, 0);
        bytes
    }

    #[test]
    fn header_gives_mip_levels_and_face_size() {
        let buffer = Handle::new(Buffer::new(Some(BaseAllocator::common()), 0).unwrap());
        assert!(ImageDdsParser::load_buffer(&Handle::default(), &buffer, -1).is_null());
        let mut data = Buffer::new(Some(BaseAllocator::common()), 0).unwrap();
        let bytes = dds(b"DXT5", 8, 4, 3, 64);
        data.allocate(bytes.len()).unwrap();
        data.as_mut_slice().copy_from_slice(&bytes);
        let data = Handle::new(data);
        let definition = ImageDdsParser::load_buffer(&Handle::default(), &data, 0);
        let pixmap = definition.get().unwrap();
        assert_eq!(pixmap.mip_maps(), vec![32, 16, 16]);
        assert_eq!(
            (pixmap.face_bytes(), pixmap.is_complete_mip_map_set()),
            (64, false)
        );
        assert_eq!(
            pixmap.compressed_format(),
            ImageCompressedFormat::RgbaS3tcDxt5
        );
        assert_eq!(pixmap.face_data().get().unwrap().size(), 64);
        assert!(ImageDdsParser::load_buffer(&Handle::default(), &data, 1).is_null());
        let formats = Handle::new(ImageSupportedFormats::default());
        assert!(ImageDdsParser::load_buffer(&formats, &data, -1).is_null());
    }

    #[test]
    fn missing_files_and_unknown_formats_give_null() {
        let path = AsciiString::from("/nonexistent/texture.dds");
        assert!(ImageDdsParser::load(&Handle::default(), &path, 0, 0).is_null());
        assert!(ImageDdsParser::parse_header(&dds(b"ATI2", 4, 4, 1, 0)[4..]).is_null());
    }
}
