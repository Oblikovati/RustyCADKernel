// Rust port of ApplicationFramework/TKCDF/CDM/CDM_Document.cxx (modification, comment,
// open/close and dump members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The life-cycle members of `CDM_Document`.
use super::{CdmApplication, CdmCanCloseStatus, CdmDocument, CdmDocumentReferences};
use crate::json_dump::{DumpChild, class_begin, field_dumped, field_integer, field_string};
use crate::live;
use rustycad_tkernel::{
    standard::{Failure, Handle, dump::DumpStream},
    tcollection::ExtendedString,
};
use std::io;

/// The life-cycle members of `CDM_Document`, provided for every [`CdmDocument`].
pub trait CdmDocumentLifecycle: CdmDocument {
    /// `IsReadOnly()`: the read-only flag of a stored document's meta-data.
    fn is_read_only(&self) -> bool {
        self.document_base().is_read_only()
    }
    /// `SetIsReadOnly`: sets the flag on the meta-data of a stored document.
    fn set_is_read_only(&self) {
        let meta_data = self.document_base().read().meta_data.clone();
        if let Some(meta_data) = meta_data.get() {
            meta_data.set_is_read_only();
        }
    }
    /// `UnsetIsReadOnly`: clears the flag on the meta-data of a stored document.
    fn unset_is_read_only(&self) {
        let meta_data = self.document_base().read().meta_data.clone();
        if let Some(meta_data) = meta_data.get() {
            meta_data.unset_is_read_only();
        }
    }
    /// `Modify`: increments the modification counter.
    fn modify(&self) {
        self.document_base().write().version += 1;
    }
    /// `Modifications`: the modification counter (1 for a new document).
    fn modifications(&self) -> i32 {
        self.document_base().modifications()
    }
    /// `UnModify`: resets the counter to the storage version.
    fn un_modify(&self) {
        let mut state = self.document_base().write();
        state.version = state.storage_version;
    }
    /// `SetModifications`; `document.set_modifications(3)`.
    fn set_modifications(&self, modifications: i32) {
        self.document_base().write().version = modifications;
    }
    /// `SetComment`: replaces the comments by one.
    fn set_comment(&self, comment: &ExtendedString) {
        let mut state = self.document_base().write();
        state.comments.clear();
        state.comments.push(comment.clone());
    }
    /// `AddComment`: appends a comment.
    fn add_comment(&self, comment: &ExtendedString) {
        self.document_base().write().comments.push(comment.clone());
    }
    /// `SetComments`: replaces every comment.
    fn set_comments(&self, comments: &[ExtendedString]) {
        self.document_base().write().comments = comments.to_vec();
    }
    /// `Comments`: copies every comment into `comments`.
    fn comments(&self, comments: &mut Vec<ExtendedString>) {
        comments.clone_from(&self.document_base().read().comments);
    }
    /// `Comment`: the first comment; `None` is the native null pointer.
    fn comment(&self) -> Option<ExtendedString> {
        self.document_base().read().comments.first().cloned()
    }
    /// `IsStored()`: the document has meta-data.
    fn is_stored(&self) -> bool {
        self.document_base().is_stored()
    }
    /// `IsModified`: the counter exceeds the storage version.
    fn is_modified(&self) -> bool {
        let state = self.document_base().read();
        state.version > state.storage_version
    }
    /// `Print` (and `operator<<`): writes nothing.
    fn print(&self, _stream: &mut dyn io::Write) -> io::Result<()> {
        Ok(())
    }
    /// `IsOpened()`: an application opened the document.
    fn is_opened(&self) -> bool {
        self.document_base().is_opened()
    }
    /// `Open`: records the opening application.
    fn open(&self, application: &Handle<dyn CdmApplication>) {
        self.document_base().write().application = application.clone();
    }
    /// `CanClose`: the first reason (in native order) that forbids closing.
    fn can_close(&self) -> Result<CdmCanCloseStatus, Failure> {
        if !self.is_opened() {
            return Ok(CdmCanCloseStatus::NotOpen);
        }
        let references = self.document_base().references_to_this();
        if references.is_empty() {
            return Ok(CdmCanCloseStatus::Ok);
        }
        if !self.is_stored() {
            return Ok(CdmCanCloseStatus::UnstoredReferenced);
        }
        if self.is_modified() {
            return Ok(CdmCanCloseStatus::ModifiedReferenced);
        }
        let this = self.document_base().this()?;
        for reference in &references {
            let reference = live(reference, "CDM_Document::CanClose")?;
            let from_document = reference.from_document();
            let from = live(&from_document, "CDM_Document::CanClose")?;
            if !from.can_close_reference(&this, reference.reference_identifier()) {
                return Ok(CdmCanCloseStatus::ReferenceRejection);
            }
        }
        Ok(CdmCanCloseStatus::Ok)
    }
    /// `Close`: `Standard_Failure` with the native message unless [`Self::can_close`] is
    /// `Ok`; then the referencing documents keep only the meta-data of this one, its own
    /// references are removed, and it is no longer stored nor opened.
    fn close(&self) -> Result<(), Failure> {
        if let Some(message) = close_refusal(self.can_close()?) {
            return Err(Failure::new(message));
        }
        let references = self.document_base().references_to_this();
        if !references.is_empty() {
            let meta_data = super::CdmDocumentStorage::meta_data(self)?;
            let application = self.document_base().read().application.clone();
            for reference in &references {
                live(reference, "CDM_Document::Close")?.unset_to_document(&meta_data, &application);
            }
        }
        self.remove_all_references()?;
        super::CdmDocumentStorage::unset_is_stored(self);
        self.document_base().write().application = Handle::default();
        Ok(())
    }
    /// `Application`: the opening application; `Standard_Failure` when not opened.
    fn application(&self) -> Result<Handle<dyn CdmApplication>, Failure> {
        let application = self.document_base().read().application.clone();
        if application.is_null() {
            return Err(Failure::new(
                "this document has not yet been opened by any application",
            ));
        }
        Ok(application)
    }
    /// `DumpJson`: the native fields in native order.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "CDM_Document")?;
        let (comments, from_references, to_references) = {
            let state = self.document_base().read();
            let lists = (state.comments.clone(), state.from_references.clone());
            (lists.0, lists.1, state.to_references.clone())
        };
        for comment in &comments {
            field_string(stream, "aComment", &comment.to_utf8())?;
        }
        for reference in from_references.iter().filter_map(Handle::get) {
            let child = |s: &mut DumpStream, d: i32| reference.dump_json(s, d);
            field_dumped(stream, depth, "aFromReference.get()", Some(&child))?;
        }
        for reference in to_references.iter().filter_map(Handle::get) {
            let child = |s: &mut DumpStream, d: i32| reference.dump_json(s, d);
            field_dumped(stream, depth, "aToReference.get()", Some(&child))?;
        }
        dump_scalar_fields(self, stream, depth)
    }
}
impl<T: CdmDocument + ?Sized> CdmDocumentLifecycle for T {}

/// The native exception message of `Close` for each refusing status.
fn close_refusal(status: CdmCanCloseStatus) -> Option<&'static str> {
    match status {
        CdmCanCloseStatus::Ok => None,
        CdmCanCloseStatus::NotOpen => Some("cannot close a document that has not been opened"),
        CdmCanCloseStatus::UnstoredReferenced => {
            Some("cannot close an unstored document which is referenced")
        }
        CdmCanCloseStatus::ModifiedReferenced => Some(
            "cannot close a document which is referenced when the document has been modified since it was stored.",
        ),
        CdmCanCloseStatus::ReferenceRejection => {
            Some("cannot close this document because a document referencing it refuses")
        }
    }
}

/// The fields of `DumpJson` after the comment and reference lists.
fn dump_scalar_fields<D: CdmDocument + ?Sized>(
    document: &D,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    let base = document.document_base();
    let (version, identifier, storage_version, meta_data) = {
        let s = base.read();
        (
            s.version,
            s.actual_reference_identifier,
            s.storage_version,
            s.meta_data.clone(),
        )
    };
    field_integer(stream, "myVersion", version)?;
    field_integer(stream, "myActualReferenceIdentifier", identifier)?;
    field_integer(stream, "myStorageVersion", storage_version)?;
    if let Some(meta_data) = meta_data.get() {
        let child = |s: &mut DumpStream, d: i32| meta_data.dump_json(s, d);
        field_dumped(stream, depth, "myMetaData.get()", Some(&child))?;
    }
    dump_request_fields(document, stream)?;
    let application = base.read().application.clone();
    let child = |s: &mut DumpStream, d: i32| {
        application
            .get()
            .map_or(Ok(()), |application| application.dump_json(s, d))
    };
    let child: Option<DumpChild<'_>> = application.get().map(|_| &child as DumpChild<'_>);
    field_dumped(stream, depth, "myApplication.get()", child)?;
    Ok(())
}

/// The requested-storage and resource fields of `DumpJson`.
fn dump_request_fields<D: CdmDocument + ?Sized>(
    document: &D,
    stream: &mut DumpStream,
) -> io::Result<()> {
    let state = document.document_base().read();
    field_string(
        stream,
        "myRequestedComment",
        &state.requested_comment.to_utf8(),
    )?;
    field_string(
        stream,
        "myRequestedFolder",
        &state.requested_folder.to_utf8(),
    )?;
    field_integer(
        stream,
        "myRequestedFolderIsDefined",
        i32::from(state.requested_folder_is_defined),
    )?;
    field_string(stream, "myRequestedName", &state.requested_name.to_utf8())?;
    field_integer(
        stream,
        "myRequestedNameIsDefined",
        i32::from(state.requested_name_is_defined),
    )?;
    let previous_defined = i32::from(state.requested_previous_version_is_defined);
    field_integer(
        stream,
        "myRequestedPreviousVersionIsDefined",
        previous_defined,
    )?;
    field_string(
        stream,
        "myRequestedPreviousVersion",
        &state.requested_previous_version.to_utf8(),
    )?;
    field_string(stream, "myFileExtension", &state.file_extension.to_utf8())?;
    field_string(stream, "myDescription", &state.description.to_utf8())?;
    field_integer(
        stream,
        "myFileExtensionWasFound",
        i32::from(state.file_extension_was_found),
    )?;
    field_integer(
        stream,
        "myDescriptionWasFound",
        i32::from(state.description_was_found),
    )
}
