// Rust port of ApplicationFramework/TKCDF/CDM/CDM_Document.cxx (reference members)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The reference members of `CDM_Document`.
use super::document::identifier_of;
use super::{
    CdmApplication, CdmDocument, CdmDocumentLifecycle, CdmMetaData, CdmReference, ModifContext,
};
use crate::live;
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::ExtendedString,
};

/// The reference members of `CDM_Document`, provided for every [`CdmDocument`].
pub trait CdmDocumentReferences: CdmDocument {
    /// `CreateReference(anOtherDocument)`: the identifier of the reference to the document,
    /// created with the next identifier when there is none.
    fn create_reference(&self, other_document: &Handle<dyn CdmDocument>) -> Result<i32, Failure> {
        let base = self.document_base();
        for reference in base.references_from_this() {
            if *other_document == reference_of(&reference)?.document() {
                return Ok(reference_of(&reference)?.reference_identifier());
            }
        }
        let other = live(other_document, "CDM_Document::CreateReference")?;
        let identifier = next_identifier(self);
        let reference = CdmReference::to_document_in_session(
            &base.this()?,
            other_document,
            identifier,
            other.document_base().modifications(),
        );
        base.add_to_reference(&reference);
        other.document_base().add_from_reference(&reference);
        Ok(identifier)
    }
    /// `RemoveReference`: removes the first reference with that identifier from both
    /// documents (the referenced document is retrieved if needed, as natively).
    fn remove_reference(&self, reference_identifier: i32) -> Result<(), Failure> {
        let base = self.document_base();
        let references = base.references_from_this();
        let Some(index) = references
            .iter()
            .position(|reference| identifier_of(reference) == Some(reference_identifier))
        else {
            return Ok(());
        };
        let to_document = reference_of(&references[index])?.to_document()?;
        live(&to_document, "CDM_Document::RemoveReference")?
            .document_base()
            .remove_from_reference(reference_identifier);
        base.write()
            .to_references
            .retain(|kept| *kept != references[index]);
        Ok(())
    }
    /// `RemoveAllReferences`: removes every reference from this document.
    fn remove_all_references(&self) -> Result<(), Failure> {
        let base = self.document_base();
        for reference in base.references_from_this() {
            let reference = reference_of(&reference)?;
            let to_document = reference.to_document()?;
            live(&to_document, "CDM_Document::RemoveAllReferences")?
                .document_base()
                .remove_from_reference(reference.reference_identifier());
        }
        base.write().to_references.clear();
        Ok(())
    }
    /// `Document(aReferenceIdentifier)`: this document for 0, else the referenced one
    /// (retrieved if needed), null for an unknown identifier.
    fn document(&self, reference_identifier: i32) -> Result<Handle<dyn CdmDocument>, Failure> {
        if reference_identifier == 0 {
            return self.document_base().this();
        }
        let reference = self.reference(reference_identifier);
        match reference.get() {
            Some(reference) => reference.to_document(),
            None => Ok(Handle::default()),
        }
    }
    /// `Reference`: the first reference from this document with that identifier, or null.
    fn reference(&self, reference_identifier: i32) -> Handle<CdmReference> {
        let references = self.document_base().references_from_this();
        references
            .into_iter()
            .find(|reference| identifier_of(reference) == Some(reference_identifier))
            .unwrap_or_default()
    }
    /// `IsInSession(id)`: true for 0; `Standard_NoSuchObject` for an unknown identifier.
    fn is_in_session(&self, reference_identifier: i32) -> Result<bool, Failure> {
        if reference_identifier == 0 {
            return Ok(true);
        }
        Ok(known_reference(self, reference_identifier)?.is_in_session())
    }
    /// `IsStored(id)`: `IsStored()` for 0; `Standard_NoSuchObject` for an unknown
    /// identifier (the native message names `IsInSession`).
    fn is_stored_reference(&self, reference_identifier: i32) -> Result<bool, Failure> {
        if reference_identifier == 0 {
            return Ok(self.document_base().is_stored());
        }
        Ok(known_reference(self, reference_identifier)?.is_stored())
    }
    /// `Name(id)`: the meta-data name of this (0) or the referenced stored document;
    /// `Standard_DomainError` when it is not stored.
    fn name(&self, reference_identifier: i32) -> Result<ExtendedString, Failure> {
        if !self.is_stored_reference(reference_identifier)? {
            return Err(exceptions::domain_error(
                "CDM_Document::Name: document is not stored",
            ));
        }
        let meta_data = if reference_identifier == 0 {
            self.document_base().read().meta_data.clone()
        } else {
            reference_of(&self.reference(reference_identifier))?.meta_data()
        };
        Ok(live(&meta_data, "CDM_Document::Name")?.name())
    }
    /// `UpdateFromDocuments`: calls `Update(this, id, context)` on every referencing document,
    /// then lets each one recompute between `BeginOfUpdate` and `EndOfUpdate`.
    ///
    /// Native quirk kept: the list of documents to update is a fresh local, so every call
    /// starts an update cycle and the ordering loop never runs; every referencing document
    /// is appended once per reference.
    fn update_from_documents(&self, modif_context: ModifContext) -> Result<(), Failure> {
        let this = self.document_base().this()?;
        let mut to_update: Vec<Handle<dyn CdmDocument>> = Vec::new();
        for reference in self.document_base().references_to_this() {
            let reference = reference_of(&reference)?;
            let from_document = reference.from_document();
            let from = live(&from_document, "CDM_Document::UpdateFromDocuments")?;
            to_update.push(from_document.clone());
            from.update_reference(&this, reference.reference_identifier(), modif_context);
        }
        to_update.iter().try_for_each(update_one_document)
    }
    /// `ToReferencesNumber`: references from this document.
    fn to_references_number(&self) -> i32 {
        self.document_base().read().to_references.len() as i32
    }
    /// `FromReferencesNumber`: references to this document.
    #[allow(clippy::wrong_self_convention)] // the native name FromReferencesNumber
    fn from_references_number(&self) -> i32 {
        self.document_base().read().from_references.len() as i32
    }
    /// `ShallowReferences`. Native quirk kept: it compares `aDocument` with the target of
    /// the references *to* this document, which is this document itself, so it is true
    /// only for this document while another document references it.
    fn shallow_references(&self, document: &Handle<dyn CdmDocument>) -> bool {
        let references = self.document_base().references_to_this();
        references
            .iter()
            .any(|reference| reference.get().is_some_and(|r| r.document() == *document))
    }
    /// `DeepReferences`: [`CdmDocumentReferences::shallow_references`] applied recursively.
    ///
    /// Native quirk: the recursion is on the target of the references to this document,
    /// which is this document again, so native code overflows the stack whenever this
    /// document is referenced and `aDocument` is another one. The port visits each
    /// document once and answers false there.
    fn deep_references(&self, document: &Handle<dyn CdmDocument>) -> bool {
        let mut visited: Vec<*const ()> = Vec::new();
        deep_references_from(
            self.document_base().references_to_this(),
            document,
            &mut visited,
        )
    }
    /// `CopyReference`: re-creates in this document the reference `id` of this document
    /// (in session or stored); 0 for an unknown identifier.
    fn copy_reference(
        &self,
        _from_document: &Handle<dyn CdmDocument>,
        reference_identifier: i32,
    ) -> Result<i32, Failure> {
        let reference = self.reference(reference_identifier);
        let Some(reference) = reference.get() else {
            return Ok(0);
        };
        let document = reference.document();
        if document.is_some() {
            return self.create_reference(&document);
        }
        self.create_reference_to_meta_data(
            &reference.meta_data(),
            &reference.application(),
            reference.document_version(),
            reference.use_storage_configuration(),
        )
    }
    /// `IsReadOnly(id)`: of the referenced document (`Standard_NullObject` for an unknown
    /// identifier, a crash natively).
    fn is_read_only_reference(&self, reference_identifier: i32) -> Result<bool, Failure> {
        reference_of(&self.reference(reference_identifier))?.is_read_only()
    }
    /// `IsUpToDate(id)`: the reference version equals the referenced document's version.
    fn is_up_to_date(&self, reference_identifier: i32) -> Result<bool, Failure> {
        reference_of(&self.reference(reference_identifier))?.is_up_to_date()
    }
    /// `SetIsUpToDate(id)`: takes the referenced document's actual version.
    fn set_is_up_to_date(&self, reference_identifier: i32) -> Result<(), Failure> {
        reference_of(&self.reference(reference_identifier))?.set_is_up_to_date()
    }
    /// `IsOpened(id)`: the referenced document is retrieved and opened.
    fn is_opened_reference(&self, reference_identifier: i32) -> bool {
        let reference = self.reference(reference_identifier);
        reference.get().is_some_and(CdmReference::is_opened)
    }
    /// `CreateReference(aMetaData, id, anApplication, aToDocumentVersion, UseStorageConfiguration)`:
    /// re-creates a stored reference with a given identifier (used by readers).
    fn create_reference_with_identifier(
        &self,
        meta_data: &Handle<CdmMetaData>,
        reference_identifier: i32,
        application: &Handle<dyn CdmApplication>,
        to_document_version: i32,
        use_storage_configuration: bool,
    ) -> Result<(), Failure> {
        let base = self.document_base();
        {
            let mut state = base.write();
            state.actual_reference_identifier =
                state.actual_reference_identifier.max(reference_identifier);
        }
        let meta = live(meta_data, "CDM_Document::CreateReference")?;
        let this = base.this()?;
        if !meta.is_retrieved() {
            let reference = CdmReference::to_stored_document(
                &this,
                meta_data,
                reference_identifier,
                application,
                to_document_version,
                use_storage_configuration,
            );
            base.add_to_reference(&reference);
            return Ok(());
        }
        let document = meta.document();
        let reference = CdmReference::to_document_in_session(
            &this,
            &document,
            reference_identifier,
            to_document_version,
        );
        base.add_to_reference(&reference);
        live(&document, "CDM_Document::CreateReference")?
            .document_base()
            .add_from_reference(&reference);
        Ok(())
    }
    /// `CreateReference(aMetaData, anApplication, aDocumentVersion, UseStorageConfiguration)`:
    /// the identifier of the reference to that meta-data, created when there is none.
    fn create_reference_to_meta_data(
        &self,
        meta_data: &Handle<CdmMetaData>,
        application: &Handle<dyn CdmApplication>,
        document_version: i32,
        use_storage_configuration: bool,
    ) -> Result<i32, Failure> {
        let base = self.document_base();
        for reference in base.references_from_this() {
            if *meta_data == reference_of(&reference)?.meta_data() {
                return Ok(reference_of(&reference)?.reference_identifier());
            }
        }
        let identifier = next_identifier(self);
        let reference = CdmReference::to_stored_document(
            &base.this()?,
            meta_data,
            identifier,
            application,
            document_version,
            use_storage_configuration,
        );
        base.add_to_reference(&reference);
        Ok(identifier)
    }
    /// `ReferenceCounter`: the last reference identifier given.
    fn reference_counter(&self) -> i32 {
        self.document_base().read().actual_reference_identifier
    }
    /// `SetReferenceCounter`; `document.set_reference_counter(5)`.
    fn set_reference_counter(&self, reference_counter: i32) {
        self.document_base().write().actual_reference_identifier = reference_counter;
    }
}
impl<T: CdmDocument + ?Sized> CdmDocumentReferences for T {}

/// `++myActualReferenceIdentifier`.
fn next_identifier<D: CdmDocument + ?Sized>(document: &D) -> i32 {
    let mut state = document.document_base().write();
    state.actual_reference_identifier += 1;
    state.actual_reference_identifier
}

/// The reference behind a handle; `Standard_NullObject` for a null one.
fn reference_of(reference: &Handle<CdmReference>) -> Result<&CdmReference, Failure> {
    live(reference, "CDM_Document reference")
}

/// The reference with that identifier, or the native `Standard_NoSuchObject`.
fn known_reference<D: CdmDocument + ?Sized>(
    document: &D,
    reference_identifier: i32,
) -> Result<Handle<CdmReference>, Failure> {
    let reference = document.reference(reference_identifier);
    if reference.is_null() {
        return Err(exceptions::no_such_object(&format!(
            "CDM_Document::IsInSession: invalid reference identifier {reference_identifier}"
        )));
    }
    Ok(reference)
}

/// One step of the update cycle of `UpdateFromDocuments`.
fn update_one_document(document: &Handle<dyn CdmDocument>) -> Result<(), Failure> {
    let target = live(document, "CDM_Document::UpdateFromDocuments")?;
    let application = target.application()?;
    let application = live(&application, "CDM_Document::UpdateFromDocuments")?;
    let mut error_string = ExtendedString::default();
    application.begin_of_update(document);
    let status = target.update_with_error(&mut error_string);
    application.end_of_update(document, status, &error_string);
    Ok(())
}

/// The recursion of `DeepReferences` over the references to one document.
fn deep_references_from(
    references: Vec<Handle<CdmReference>>,
    document: &Handle<dyn CdmDocument>,
    visited: &mut Vec<*const ()>,
) -> bool {
    for reference in references {
        let Some(target) = reference.get().map(CdmReference::document) else {
            continue;
        };
        let Some(target_document) = target.get() else {
            continue;
        };
        if target == *document {
            return true;
        }
        if visited.contains(&target.object_address()) {
            continue;
        }
        visited.push(target.object_address());
        let next = target_document.document_base().references_to_this();
        if deep_references_from(next, document, visited) {
            return true;
        }
    }
    false
}
