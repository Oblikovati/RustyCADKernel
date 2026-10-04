//! Rust-callable counterparts of `FSD_File`'s protected virtual text primitives.
use super::{TextDialect, TextFile};
use crate::standard::Failure;
use crate::storage::StorageError;
use crate::tcollection::{AsciiString, ExtendedString};

impl<D: TextDialect> TextFile<D> {
    /// Calls qualified FSD `FlushEndOfLine`, retaining virtual `ReadLine` dispatch.
    /// Example: `file.flush_fsd_end_of_line_base()`.
    pub fn flush_fsd_end_of_line_base(&mut self) {
        D::read_line(self);
    }

    /// Calls qualified FSD `FindTag`, retaining virtual `ReadString` dispatch.
    /// Example: `file.find_fsd_tag_base("BEGIN_INFO_SECTION")`.
    pub fn find_fsd_tag_base(&mut self, tag: &str) -> StorageError {
        let c_tag = tag.split('\0').next().unwrap_or_default();
        self.find_tag_c_string(c_tag.as_bytes())
    }

    /// Reads through the selected dialect's virtual `ReadLine`; `file.read_line()`.
    pub fn read_line(&mut self) -> AsciiString {
        D::read_line(self)
    }

    /// Reads through the selected dialect's virtual `ReadString`; `file.read_string()`.
    pub fn read_string(&mut self) -> AsciiString {
        D::read_string(self)
    }

    /// Reads an extended line through the dialect override; `file.read_extended_line(&mut text)`.
    pub fn read_extended_line(&mut self, text: &mut ExtendedString) -> Result<(), Failure> {
        D::read_extended_line(self, text)
    }

    /// Writes an extended line through the dialect override; `file.write_extended_line(&text)`.
    pub fn write_extended_line(&mut self, text: &ExtendedString) -> Result<(), Failure> {
        D::write_extended_line(self, text)
    }
}

impl<D: TextDialect> TextFile<D> {
    /// Finds a line equal to `tag` using dialect-specific `ReadString`; `file.find_tag("BEGIN")`.
    pub fn find_tag(&mut self, tag: &str) -> StorageError {
        let c_tag = tag.split('\0').next().unwrap_or_default();
        D::find_tag(self, c_tag)
    }

    /// Reads through the dialect-specific line virtual until a newline; `file.flush_end_of_line()`.
    pub fn flush_end_of_line(&mut self) {
        D::flush_end_of_line(self);
    }

    pub(crate) fn find_tag_c_string(&mut self, tag: &[u8]) -> StorageError {
        let mut line = D::read_string(self);
        while line.as_bytes() != tag && !self.stream.eof() {
            line = D::read_string(self);
        }
        if self.stream.eof() {
            StorageError::SectionNotFound
        } else {
            StorageError::Ok
        }
    }
}
