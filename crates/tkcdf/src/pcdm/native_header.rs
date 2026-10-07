//! The header queries of the native OCAF containers (`PCDM_ReadWriter_1` over the FSD
//! drivers, and the Xml `<document format=...>` probe through LDOM).
//!
//! Native file formats are not ported (user decision 2026-09, ocaf-design.md section 1).
//! Every query below answers exactly what the native code answers for a file whose type
//! `PCDM::FileDriverType` does not recognise and whose XML probe finds no `document`
//! element: the only kind of file the port can read. These are the single seam where the
//! native readers would plug in.
use super::PcdmReference;
use rustycad_tkernel::{message::Messenger, tcollection::ExtendedString};
use std::sync::Arc;

/// `PCDM_ReadWriter::FileFormat(aFileName)` for a file in no native format: empty.
pub(crate) fn file_format(_file_name: &ExtendedString) -> ExtendedString {
    ExtendedString::default()
}

/// `PCDM_ReadWriter_1::ReadReferenceCounter` for a file in no native format: 0.
pub(crate) fn reference_counter(_file_name: &ExtendedString, _messenger: &Arc<Messenger>) -> i32 {
    0
}

/// `PCDM_ReadWriter_1::ReadDocumentVersion` for a file in no native format: -1.
pub(crate) fn document_version(_file_name: &ExtendedString, _messenger: &Arc<Messenger>) -> i32 {
    -1
}

/// `PCDM_ReadWriter_1::ReadReferences` for a file in no native format: no reference.
pub(crate) fn references(
    _file_name: &ExtendedString,
    _references: &mut Vec<PcdmReference>,
    _messenger: &Arc<Messenger>,
) {
}
