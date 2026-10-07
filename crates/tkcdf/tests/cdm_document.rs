//! Port-only behaviour tests of the CDM package against the native C++ bodies.
mod support;
use rustycad_tkcdf::cdm::{
    CdmCanCloseStatus, CdmDocument, CdmDocumentLifecycle, CdmDocumentReferences,
    CdmDocumentStorage, CdmMetaData, CdmReferenceIterator, MetaDataLookUpTable,
};
use rustycad_tkernel::standard::{Handle, dump::DumpStream};
use support::{new_application, new_document, text};

#[test]
fn references_get_increasing_identifiers_and_are_reused() {
    let (from, _) = new_document();
    let (to, _) = new_document();
    let (other, _) = new_document();
    assert_eq!(from.create_reference(&to).unwrap(), 1);
    assert_eq!(from.create_reference(&to).unwrap(), 1);
    assert_eq!(from.create_reference(&other).unwrap(), 2);
    assert_eq!(from.to_references_number(), 2);
    assert_eq!(to.from_references_number(), 1);
    assert!(from.document(1).unwrap() == to);
    assert!(from.document(0).unwrap() == from);
    assert!(from.document(9).unwrap().is_null());
    assert!(from.is_in_session(1).unwrap());
    assert!(!from.is_stored_reference(1).unwrap());
    let error = from.is_in_session(7).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
    from.remove_reference(1).unwrap();
    assert_eq!(from.to_references_number(), 1);
    assert_eq!(to.from_references_number(), 0);
    from.remove_all_references().unwrap();
    assert_eq!(other.from_references_number(), 0);
    assert_eq!(from.reference_counter(), 2);
}

#[test]
fn up_to_date_follows_the_referenced_counter() {
    let (from, _) = new_document();
    let (to, _) = new_document();
    let id = from.create_reference(&to).unwrap();
    assert!(from.is_up_to_date(id).unwrap());
    to.modify();
    assert!(!from.is_up_to_date(id).unwrap());
    from.set_is_up_to_date(id).unwrap();
    assert!(from.is_up_to_date(id).unwrap());
    assert!(from.is_up_to_date(5).is_err());
}

#[test]
fn shallow_and_deep_references_keep_the_native_target() {
    let (from, _) = new_document();
    let (to, _) = new_document();
    from.create_reference(&to).unwrap();
    // The references *to* `to` point at `to` itself (native quirk).
    assert!(to.shallow_references(&to));
    assert!(!to.shallow_references(&from));
    assert!(to.deep_references(&to));
    assert!(!to.deep_references(&from));
}

#[test]
fn update_from_documents_calls_every_referencing_document() {
    let (application, _) = new_application(0);
    let (from, concrete) = new_document();
    let (to, _) = new_document();
    from.open(&application);
    let id = from.create_reference(&to).unwrap();
    to.update_from_documents(None).unwrap();
    assert_eq!(*concrete.updates.lock().unwrap(), vec![id]);
    let (unopened, _) = new_document();
    unopened.create_reference(&to).unwrap();
    assert!(to.update_from_documents(None).is_err());
}

#[test]
fn modification_counter_comments_and_requests() {
    let (document, _) = new_document();
    assert_eq!(document.modifications(), 1);
    assert!(document.is_modified());
    document.modify();
    document.un_modify();
    assert_eq!(document.modifications(), 0);
    assert!(document.comment().is_none());
    document.set_comment(&text("a"));
    document.add_comment(&text("b"));
    let mut comments = Vec::new();
    document.comments(&mut comments);
    assert_eq!(comments, vec![text("a"), text("b")]);
    assert_eq!(document.requested_name(), text("Document_"));
    assert!(document.requested_folder().is_err());
    document.set_requested_folder(&text(""));
    assert!(!document.has_requested_folder());
    document.set_requested_folder(&text("dir"));
    assert_eq!(document.requested_folder().unwrap(), text("dir"));
    assert!(document.requested_previous_version().is_err());
    document.set_requested_comment(&text("c"));
    assert_eq!(document.requested_comment(), text("c"));
}

#[test]
fn meta_data_storage_and_resources() {
    let (application, concrete) = new_application(4);
    let (document, _) = new_document();
    assert!(document.meta_data().is_err());
    assert!(document.load_resources().is_err());
    document.open(&application);
    concrete
        .resources
        .write()
        .set_resource("TestFormat.FileExtension", b"tst");
    assert!(document.find_file_extension().unwrap());
    assert_eq!(document.file_extension().unwrap(), text("tst"));
    assert!(!document.find_description().unwrap());
    let meta = {
        let mut table = application.meta_data_look_up_table();
        CdmMetaData::look_up_with_version(
            &mut table,
            &text("f"),
            &text("n"),
            &text("p"),
            &text("v"),
            &text("p"),
            false,
        )
    };
    document.modify();
    document.set_meta_data(&meta).unwrap();
    assert!(document.is_stored() && !document.is_modified());
    assert_eq!(document.storage_version(), 2);
    assert!(meta.is_retrieved() && meta.document() == document);
    assert_eq!(document.folder().unwrap(), text("f"));
    assert_eq!(document.requested_previous_version().unwrap(), text("v"));
    assert_eq!(document.name(0).unwrap(), text("n"));
    document.set_is_read_only();
    assert!(document.is_read_only() && meta.is_read_only());
    document.unset_is_read_only();
    document.unset_is_stored();
    assert!(!meta.is_retrieved());
}

#[test]
fn stored_references_retrieve_and_report_versions() {
    let (application, concrete) = new_application(7);
    let (from, _) = new_document();
    let (target, _) = new_document();
    *concrete.retrieved.lock().unwrap() = target.clone();
    let mut table = MetaDataLookUpTable::default();
    let meta = CdmMetaData::look_up(
        &mut table,
        &text("f"),
        &text("n"),
        &text("p"),
        &text("p"),
        false,
    );
    let id = from
        .create_reference_to_meta_data(&meta, &application, 7, false)
        .unwrap();
    assert_eq!(
        from.create_reference_to_meta_data(&meta, &application, 7, false)
            .unwrap(),
        id
    );
    assert!(from.is_stored_reference(id).unwrap() && !from.is_in_session(id).unwrap());
    assert!(from.is_up_to_date(id).unwrap());
    assert!(!from.is_opened_reference(id));
    let mut iterator = CdmReferenceIterator::new(&from).unwrap();
    assert_eq!(iterator.reference_identifier().unwrap(), id);
    assert_eq!(iterator.document_version().unwrap(), 7);
    assert!(iterator.document().unwrap() == target);
    iterator.next();
    assert!(!iterator.more() && iterator.document().is_err());
    // The reference now holds the retrieved document, so the copy finds it again.
    assert_eq!(from.copy_reference(&from, id).unwrap(), id);
    from.create_reference_with_identifier(&meta, 10, &application, 1, true)
        .unwrap();
    assert_eq!(from.reference_counter(), 10);
}

#[test]
fn close_follows_the_native_statuses() {
    let (application, _) = new_application(0);
    let (from, concrete) = new_document();
    let (to, _) = new_document();
    assert_eq!(to.can_close().unwrap(), CdmCanCloseStatus::NotOpen);
    assert_eq!(
        to.close().unwrap_err().message(),
        "cannot close a document that has not been opened"
    );
    to.open(&application);
    from.create_reference(&to).unwrap();
    assert_eq!(
        to.can_close().unwrap(),
        CdmCanCloseStatus::UnstoredReferenced
    );
    let meta = {
        let mut table = application.meta_data_look_up_table();
        CdmMetaData::look_up(
            &mut table,
            &text("f"),
            &text("n"),
            &text("q"),
            &text("q"),
            false,
        )
    };
    to.set_meta_data(&meta).unwrap();
    *concrete.refuse_close.lock().unwrap() = true;
    assert_eq!(
        to.can_close().unwrap(),
        CdmCanCloseStatus::ReferenceRejection
    );
    *concrete.refuse_close.lock().unwrap() = false;
    to.modify();
    assert_eq!(
        to.can_close().unwrap(),
        CdmCanCloseStatus::ModifiedReferenced
    );
    to.un_modify();
    to.close().unwrap();
    assert!(!to.is_opened() && !from.is_in_session(1).unwrap());
}

#[test]
fn dump_json_lists_the_native_fields() {
    let (document, _) = new_document();
    document.set_comment(&text("hello"));
    let mut stream = DumpStream::default();
    document.dump_json(&mut stream, -1).unwrap();
    let text = String::from_utf8_lossy(stream.bytes()).into_owned();
    assert!(
        text.starts_with("\"className\": \"CDM_Document\", \"Comment\": \"hello\""),
        "{text}"
    );
    assert!(text.contains("\"Version\": 1") && text.contains("\"RequestedComment\": \"\""));
    let mut alternative: Handle<dyn CdmDocument> = Handle::default();
    assert!(
        document
            .get_alternative_document(&super_text(), &mut alternative)
            .unwrap()
    );
    assert!(alternative == document);
}

fn super_text() -> rustycad_tkernel::tcollection::ExtendedString {
    text("TestFormat")
}
