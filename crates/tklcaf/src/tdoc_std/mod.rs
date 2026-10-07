//! The TDocStd package: OCAF documents and applications, external links and multi-document
//! transactions (ocaf-design.md section 6).
mod application;
mod application_io;
mod document;
mod document_commands;
mod document_compaction;
mod multi_transaction;
mod multi_transaction_dump;
mod owner;
mod small;
mod xlink;
mod xlink_tool;
pub use application::{TDocStdApplication, TDocStdApplicationBase, TDocStdStandardApplication};
pub use application_io::TDocStdApplicationIo;
pub use document::TDocStdDocument;
pub use multi_transaction::{
    ApplicationDeltaDocuments, TDocStdApplicationDelta, TDocStdMultiTransactionManager,
};
pub use owner::{TDocStdModified, TDocStdOwner};
pub use small::{
    TDOCSTD_FORMAT_VERSION_LOWER, TDOCSTD_FORMAT_VERSION_UPPER, TDocStdCompoundDelta,
    TDocStdContext, TDocStdFormatVersion, TDocStdPathParser,
};
pub use xlink::{TDocStdXLink, TDocStdXLinkIterator, TDocStdXLinkRoot};
pub use xlink_tool::TDocStdXLinkTool;

use rustycad_tkernel::standard::Guid;

/// `TDocStd`: the package class.
pub struct TDocStd;
impl TDocStd {
    /// `IDList`: appends the ID of `TDocStd_XLink`.
    pub fn id_list(ids: &mut Vec<Guid>) {
        ids.push(TDocStdXLink::get_id());
    }
}
