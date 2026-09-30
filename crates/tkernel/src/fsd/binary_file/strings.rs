//! Length-prefixed ASCII and UTF-16 strings of the binary format.
use super::{BinaryFile, write_error};
use crate::standard::Failure;
use crate::storage::DriverPrimitives;
use crate::tcollection::{AsciiString, ExtendedString};

/// Bytes read per step of a large string, so a corrupt length cannot request gigabytes
/// before the short read is detected (natively it allocates the full length first).
const READ_STEP: usize = 1 << 16;

/// UTF-16 code units from native-order bytes.
pub(crate) fn units_from_bytes(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_ne_bytes(*pair))
        .collect()
}

/// Native-order bytes of UTF-16 code units.
pub(crate) fn bytes_from_units(units: &[u16]) -> Vec<u8> {
    units.iter().flat_map(|unit| unit.to_ne_bytes()).collect()
}

impl BinaryFile {
    /// `fread(block, size, 1)`: the bytes, or `None` when the file ends first.
    fn read_block(&mut self, size: usize) -> Result<Option<Vec<u8>>, Failure> {
        let mut block = Vec::new();
        while block.len() < size {
            let start = block.len();
            let step = (size - start).min(READ_STEP);
            block.resize(start + step, 0);
            if self.file()?.read(&mut block[start..], step) == 0 {
                return Ok(None);
            }
        }
        Ok(Some(block))
    }

    /// Protected `WriteString`: the length, then the bytes when not empty.
    pub(crate) fn write_string(&mut self, text: &AsciiString) -> Result<(), Failure> {
        self.put_integer(text.length())?;
        if text.is_empty() {
            return Ok(());
        }
        self.write_item(text.as_bytes(), "FSD_BinaryFile::WriteString")
    }

    /// Protected `ReadString`: a short read raises `Storage_StreamWriteError`, as natively;
    /// the text stops at its first NUL (`aString = c`).
    pub(crate) fn read_string(&mut self, text: &mut AsciiString) -> Result<(), Failure> {
        let mut size = 0;
        self.get_integer(&mut size)?;
        if size <= 0 {
            *text = AsciiString::default();
            return Ok(());
        }
        let block = self
            .read_block(size as usize)?
            .ok_or_else(|| write_error("FSD_BinaryFile::ReadString"))?;
        *text = AsciiString::from_bytes(&block);
        Ok(())
    }

    /// Protected `WriteExtendedString`: the unit count, then native-order units.
    pub(crate) fn write_extended_string(&mut self, text: &ExtendedString) -> Result<(), Failure> {
        self.put_integer(text.length())?;
        if text.is_empty() {
            return Ok(());
        }
        let bytes = bytes_from_units(text.as_utf16());
        self.write_item(&bytes, "FSD_BinaryFile::WriteExtendedString")
    }

    /// Protected `ReadExtendedString`: as `ReadString`, the text stops at its first zero unit.
    pub(crate) fn read_extended_string(
        &mut self,
        text: &mut ExtendedString,
    ) -> Result<(), Failure> {
        let mut size = 0;
        self.get_integer(&mut size)?;
        if size <= 0 {
            *text = ExtendedString::default();
            return Ok(());
        }
        let block = self
            .read_block(size as usize * 2)?
            .ok_or_else(|| write_error("FSD_BinaryFile::ReadExtendedString"))?;
        *text = ExtendedString::from_utf16(&units_from_bytes(&block));
        Ok(())
    }
}
