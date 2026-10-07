// Rust port of CDF_Application::Retrieve(aMetaData, UseStorageConfiguration, IsComponent, ...)
// (ApplicationFramework/TKCDF/CDF/CDF_Application.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The retrieval of a stored document by its meta-data.
use super::{CdfApplication, CdfApplicationServices};
use crate::cdm::{
    CdmDocument, CdmDocumentLifecycle, CdmDocumentReferences, CdmDocumentStorage, CdmMetaData,
};
use crate::live;
use crate::pcdm::{PcdmReaderFilter, PcdmReaderStatus, PcdmRetrievalDriverBase};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Failure, Handle, exceptions},
    tcollection::ExtendedString,
};

/// The native body: checks a component, then reads the file into a new document (or into
/// the retrieved one when it was modified or in append mode).
pub(crate) fn retrieve<A: CdfApplication + ?Sized>(
    application: &A,
    meta_data: &Handle<CdmMetaData>,
    use_storage_configuration: bool,
    is_component: bool,
    filter: &Handle<PcdmReaderFilter>,
    range: &ProgressRange<'_>,
) -> Result<Handle<dyn CdmDocument>, Failure> {
    let base = application.cdf_application_base();
    base.set_retrievable_status(PcdmReaderStatus::DriverFailure);
    let append_mode = filter.get().is_some_and(PcdmReaderFilter::is_append_mode);
    if is_component {
        check_component(application, meta_data, append_mode)?;
    }
    let meta = live(meta_data, "CDF_Application::Retrieve")?;
    let already_retrieved = meta.is_retrieved();
    if already_retrieved {
        base.set_retrievable_status(PcdmReaderStatus::AlreadyRetrieved);
    }
    let modified =
        already_retrieved && live(&meta.document(), "CDF_Application::Retrieve")?.is_modified();
    if modified {
        base.set_retrievable_status(PcdmReaderStatus::AlreadyRetrievedAndModified);
    }
    if already_retrieved && !modified && !append_mode {
        return Ok(meta.document());
    }
    let reading = Reading {
        meta_data,
        use_storage_configuration,
        modified,
        append_mode,
        filter,
        range,
    };
    read_document(application, &reading)
}

/// The arguments of one retrieval read.
struct Reading<'a, 'r> {
    meta_data: &'a Handle<CdmMetaData>,
    use_storage_configuration: bool,
    modified: bool,
    append_mode: bool,
    filter: &'a Handle<PcdmReaderFilter>,
    range: &'a ProgressRange<'r>,
}

/// The component check: `Standard_Failure` with the native message unless the meta-data
/// can be retrieved.
fn check_component<A: CdfApplication + ?Sized>(
    application: &A,
    meta_data: &Handle<CdmMetaData>,
    append_mode: bool,
) -> Result<(), Failure> {
    let base = application.cdf_application_base();
    let status = application.can_retrieve_meta_data(meta_data, append_mode)?;
    base.set_retrievable_status(status);
    let path = || {
        meta_data
            .get()
            .map(|meta| meta.path().to_utf8())
            .unwrap_or_default()
    };
    let message = match status {
        PcdmReaderStatus::UnknownDocument => format!(
            "could not find the referenced document: {}; not found.",
            path()
        ),
        PcdmReaderStatus::PermissionDenied => format!(
            "Could not find the referenced document: {}; permission denied. ",
            path()
        ),
        PcdmReaderStatus::NoDocument => "Document for appending is not defined.".to_owned(),
        _ => {
            base.set_retrievable_status(PcdmReaderStatus::DriverFailure);
            return Ok(());
        }
    };
    Err(Failure::new(&message))
}

/// The reading part of the retrieval.
fn read_document<A: CdfApplication + ?Sized>(
    application: &A,
    reading: &Reading<'_, '_>,
) -> Result<Handle<dyn CdmDocument>, Failure> {
    let base = application.cdf_application_base();
    let meta = live(reading.meta_data, "CDF_Application::Retrieve")?;
    let file_name = meta.file_name();
    let mut format = ExtendedString::default();
    if !application.format(&file_name, &mut format)? {
        return Err(exceptions::no_such_object(&format!(
            "Could not determine format for the file {}",
            file_name.to_utf8()
        )));
    }
    let reader = application.reader_from_format(&format)?;
    let document = target_document(application, reading, &format)?;
    let this = application.application_base().this()?;
    let reader_ref = live(&reader, "CDF_Application::Retrieve")?;
    if let Err(failure) =
        reader_ref.read_file(&file_name, &document, &this, reading.filter, reading.range)
    {
        base.set_retrievable_status(reader_ref.get_status());
        if reader_ref.get_status() > PcdmReaderStatus::AlreadyRetrieved {
            let mut text = Vec::new();
            let _ = failure.print_to(&mut text);
            text.push(b'\n');
            return Err(Failure::new(&String::from_utf8_lossy(&text)));
        }
    }
    base.set_retrievable_status(reader_ref.get_status());
    if !reading.append_mode {
        let target = live(&document, "CDF_Application::Retrieve")?;
        target.open(&this); // must be done before SetMetaData
        target.set_meta_data(reading.meta_data)?;
    }
    Ok(document)
}

/// The document the reader fills: the retrieved one (its references removed unless in
/// append mode), or a new one with the stored reference counter, version and references.
fn target_document<A: CdfApplication + ?Sized>(
    application: &A,
    reading: &Reading<'_, '_>,
    format: &ExtendedString,
) -> Result<Handle<dyn CdmDocument>, Failure> {
    let meta = live(reading.meta_data, "CDF_Application::Retrieve")?;
    if reading.modified || reading.append_mode {
        let document = meta.document();
        if !reading.append_mode {
            live(&document, "CDF_Application::Retrieve")?.remove_all_references()?;
        }
        return Ok(document);
    }
    let mut document: Handle<dyn CdmDocument> = Handle::default();
    application.new_document(format, &mut document)?;
    let counter = PcdmRetrievalDriverBase::reference_counter(
        &meta.file_name(),
        &application.message_driver(),
    );
    application.set_reference_counter(&document, counter)?;
    application.set_document_version(&document, reading.meta_data)?;
    let driver = application.meta_data_driver()?;
    let iterator = live(&driver, "CDF_Application::Retrieve")?
        .reference_iterator(&application.message_driver());
    let this = application.application_base().this()?;
    live(&iterator, "CDF_Application::Retrieve")?.load_references(
        &document,
        reading.meta_data,
        &this,
        reading.use_storage_configuration,
    )?;
    Ok(document)
}
