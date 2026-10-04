//! The protected line, word and tag readers of `FSD_File`.
use super::{TextDialect, TextFile};
use crate::standard::Failure;
use crate::storage::StreamException;
use crate::tcollection::{AsciiString, ExtendedString};

/// `getline` buffer size of the line readers (`char Buffer[8193]` read with 8192).
const LINE_BUFFER: i64 = 8192;
/// `ENDOFNORMALEXTENDEDSECTION`: an extended line that spells this tag is corrupt.
const END_OF_EXTENDED_SECTION: &[u8] = b"BEGIN_REF_SECTION";
/// `SIZEOFNORMALEXTENDEDSECTION`.
const EXTENDED_SECTION_SIZE: usize = 16;
/// `ReadWord` flushes its stack buffer every 8192 bytes.
const WORD_CHUNK: usize = 8192;

/// A C string view of `bytes`: everything before the first NUL (`buffer += Buffer`).
fn c_string(bytes: &[u8]) -> &[u8] {
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..length]
}

impl<D: TextDialect> TextFile<D> {
    /// `FSD_File::ReadLine`: one `getline` of at most 8191 bytes unless at end of file.
    pub(crate) fn fsd_read_line(&mut self) -> AsciiString {
        let mut buffer = AsciiString::default();
        if !self.stream.eof() {
            let mut line = Vec::new();
            self.stream.getline(&mut line, LINE_BUFFER, b'\n');
            buffer.assign_cat(c_string(&line));
        }
        buffer
    }

    /// `FSD_File::ReadString`: as `ReadLine`, dropping leading newlines and spaces.
    pub(crate) fn fsd_read_string(&mut self) -> AsciiString {
        let mut buffer = AsciiString::default();
        if !self.stream.eof() {
            let mut line = Vec::new();
            self.stream.getline(&mut line, LINE_BUFFER, b'\n');
            let text = c_string(&line);
            let start = text
                .iter()
                .position(|byte| *byte != b'\n' && *byte != b' ')
                .unwrap_or(text.len());
            buffer.assign_cat(&text[start..]);
        }
        buffer
    }

    /// Skips spaces/newlines and reads to the next delimiter; `file.read_word()`.
    /// Embedded NUL bytes truncate each native C-string chunk.
    pub fn read_word(&mut self) -> AsciiString {
        let mut byte = 0_u8;
        while !self.stream.eof() {
            self.stream.get(&mut byte);
            if byte != b' ' && byte != b'\n' {
                break;
            }
        }
        let mut word = AsciiString::default();
        let mut chunk = Vec::with_capacity(WORD_CHUNK);
        while !self.stream.eof() {
            if chunk.len() == WORD_CHUNK {
                word.assign_cat(c_string(&chunk));
                chunk.clear();
            }
            chunk.push(byte);
            self.stream.get(&mut byte);
            if byte == b'\n' || byte == b' ' {
                break;
            }
        }
        word.assign_cat(c_string(&chunk));
        word
    }

    /// Reads up to `size` characters from the current position; `file.read_char(4)`.
    /// A failed `get` appends the previous byte, as the native method does.
    pub fn read_char(&mut self, size: usize) -> AsciiString {
        let mut buffer = AsciiString::default();
        let mut byte = 0_u8;
        let mut count = 0;
        while !self.stream.eof() && count < size {
            self.stream.get(&mut byte);
            buffer.assign_cat(byte);
            count += 1;
        }
        buffer
    }
}

impl<D: TextDialect> TextFile<D> {
    /// `FSD_File::WriteExtendedLine`: each unit as its high and low byte, then NUL and a
    /// newline. A unit 0x000A therefore ends the line early for the reader.
    pub(crate) fn fsd_write_extended_line(&mut self, text: &ExtendedString) {
        for unit in text.as_utf16() {
            self.stream.insert_bytes(&unit.to_be_bytes());
        }
        self.stream.insert_bytes(b"\0\n");
    }

    /// `FSD_File::ReadExtendedLine`: byte pairs until a NUL high byte followed by a newline.
    /// A pair that spells the first 16 characters of `BEGIN_REF_SECTION` is an
    /// ext-character parity error. `TCollection_ExtendedString` has no `operator+=` for a
    /// `char16_t`, so each unit is appended as its decimal text (unit 0x41 reads as "65"),
    /// which is what the native reader returns.
    pub(crate) fn fsd_read_extended_line(
        &mut self,
        text: &mut ExtendedString,
    ) -> Result<(), Failure> {
        *text = ExtendedString::default();
        let (mut byte, mut matched, mut finished) = (0_u8, 0_usize, false);
        while !finished && !self.stream.eof() {
            self.stream.get(&mut byte);
            matched = tag_progress(byte, matched)?;
            let high = u16::from(byte);
            finished = byte == 0;
            self.stream.get(&mut byte);
            matched = tag_progress(byte, matched)?;
            if byte != b'\n' {
                finished = false;
                // `buffer += (char16_t)i` binds to `operator+=(const int)`: the unit's
                // decimal digits are appended, not the unit itself.
                text.assign_cat(i32::from((high << 8) | u16::from(byte)));
            }
        }
        Ok(())
    }
}

/// Advances the match against `BEGIN_REF_SECTION`, failing when 16 characters match.
fn tag_progress(byte: u8, matched: usize) -> Result<usize, Failure> {
    let matched = if END_OF_EXTENDED_SECTION.get(matched) == Some(&byte) {
        matched + 1
    } else {
        0
    };
    if matched >= EXTENDED_SECTION_SIZE {
        return Err(StreamException::ExtCharParity
            .raise("FSD_File::ReadExtendedLine: the line runs into BEGIN_REF_SECTION"));
    }
    Ok(matched)
}
