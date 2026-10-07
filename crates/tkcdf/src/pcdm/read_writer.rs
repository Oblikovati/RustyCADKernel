// Rust port of ApplicationFramework/TKCDF/PCDM/PCDM_ReadWriter.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `PCDM_ReadWriter`: reads and writes the information section (references, extensions,
//! version) of a stored document.
use super::{PcdmReference, native_header};
use crate::cdm::CdmDocument;
use crate::live;
use rustycad_tkernel::{
    impl_transient_interface,
    message::Messenger,
    standard::{Failure, Handle, StandardTransient, Transient},
    storage::Data,
    tcollection::{AsciiString, ExtendedString},
};
use std::{io, sync::Arc};

/// The `FILE_FORMAT: ` user-info key.
const FILE_FORMAT: &str = "FILE_FORMAT: ";

/// `PCDM_ReadWriter`'s pure virtual members: one header layout version.
pub trait PcdmReadWriter: Transient {
    /// `Version`: the layout version this object reads and writes.
    fn version(&self) -> AsciiString;
    /// `WriteReferenceCounter`.
    fn write_reference_counter(
        &self,
        data: &Handle<Data>,
        document: &Handle<dyn CdmDocument>,
    ) -> Result<(), Failure>;
    /// `WriteReferences`.
    fn write_references(
        &self,
        data: &Handle<Data>,
        document: &Handle<dyn CdmDocument>,
        referencer_file_name: &ExtendedString,
    ) -> Result<(), Failure>;
    /// `WriteExtensions`.
    fn write_extensions(
        &self,
        data: &Handle<Data>,
        document: &Handle<dyn CdmDocument>,
    ) -> Result<(), Failure>;
    /// `WriteVersion`.
    fn write_version(
        &self,
        data: &Handle<Data>,
        document: &Handle<dyn CdmDocument>,
    ) -> Result<(), Failure>;
    /// `ReadReferenceCounter`.
    fn read_reference_counter(
        &self,
        file_name: &ExtendedString,
        messenger: &Arc<Messenger>,
    ) -> Result<i32, Failure>;
    /// `ReadReferences`.
    fn read_references(
        &self,
        file_name: &ExtendedString,
        references: &mut Vec<PcdmReference>,
        messenger: &Arc<Messenger>,
    ) -> Result<(), Failure>;
    /// `ReadExtensions`.
    fn read_extensions(
        &self,
        file_name: &ExtendedString,
        extensions: &mut Vec<ExtendedString>,
        messenger: &Arc<Messenger>,
    ) -> Result<(), Failure>;
    /// `ReadDocumentVersion`.
    fn read_document_version(
        &self,
        file_name: &ExtendedString,
        messenger: &Arc<Messenger>,
    ) -> Result<i32, Failure>;
}
impl_transient_interface!(dyn PcdmReadWriter, "PCDM_ReadWriter", StandardTransient);

/// The static members of `PCDM_ReadWriter`.
pub struct PcdmReadWriterBase;
impl PcdmReadWriterBase {
    /// `WriteFileFormat`: adds `FILE_FORMAT: <storage format>` (non-Latin-1 units as `?`)
    /// to the user information of `data`.
    pub fn write_file_format(
        data: &Handle<Data>,
        document: &Handle<dyn CdmDocument>,
    ) -> Result<(), Failure> {
        let format = live(document, "PCDM_ReadWriter::WriteFileFormat")?.storage_format();
        let mut line = AsciiString::new(FILE_FORMAT);
        line.append(&AsciiString::from_extended(&format, Some(b'?')));
        live(data, "PCDM_ReadWriter::WriteFileFormat")?.add_to_user_info(&line);
        Ok(())
    }
    /// `ParseReference`: splits `"<identifier> <version> <file name>"`; false, outputs
    /// unchanged, unless both numbers are integers and the file name is not empty.
    ///
    /// ```
    /// use rustycad_tkcdf::pcdm::PcdmReadWriterBase;
    /// use rustycad_tkernel::tcollection::ExtendedString;
    /// let (mut id, mut version, mut file) = (0, 0, ExtendedString::default());
    /// let text = ExtendedString::new("12 3 a b.ext");
    /// assert!(PcdmReadWriterBase::parse_reference(&text, &mut id, &mut version, &mut file));
    /// assert_eq!((id, version, file), (12, 3, ExtendedString::new("a b.ext")));
    /// ```
    pub fn parse_reference(
        reference: &ExtendedString,
        identifier: &mut i32,
        document_version: &mut i32,
        file_name: &mut ExtendedString,
    ) -> bool {
        let Some((parsed_identifier, parsed_version, parsed_file)) = split_reference(reference)
        else {
            return false;
        };
        *identifier = parsed_identifier;
        *document_version = parsed_version;
        *file_name = parsed_file;
        true
    }
    /// `FileFormat(aFileName)`: the format stored in a native header; empty for a file in no
    /// native format (see `pcdm::native_header`).
    pub fn file_format(file_name: &ExtendedString) -> ExtendedString {
        native_header::file_format(file_name)
    }
    /// `FileFormat(theIStream, theData)`: as [`PcdmReadWriterBase::file_format`] for a stream
    /// in no native format: empty, and the stream and data are not touched.
    pub fn file_format_stream(
        _stream: &mut dyn io::Read,
        _data: &mut Handle<Data>,
    ) -> ExtendedString {
        ExtendedString::default()
    }
}

/// The three fields of a reference line, or `None` for a malformed one.
fn split_reference(reference: &ExtendedString) -> Option<(i32, i32, ExtendedString)> {
    let space = [u16::from(b' ')];
    let first = reference.search(space);
    if first <= 0 || first >= reference.length() {
        return None;
    }
    let mut identifier = reference.clone();
    let mut rest = identifier.split(first).ok()?;
    let second = rest.search(space);
    if second <= 0 || second >= rest.length() {
        return None;
    }
    let file = rest.split(second).ok()?;
    if file.is_empty() || !identifier.is_integer_value() || !rest.is_integer_value() {
        return None;
    }
    Some((
        identifier.integer_value().ok()?,
        rest.integer_value().ok()?,
        file,
    ))
}
