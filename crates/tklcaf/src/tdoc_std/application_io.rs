// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_Application.cxx (Open, SaveAs and
// Save) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Opening and saving documents with a `TDocStd_Application`.
use super::{TDocStdApplication, TDocStdDocument, TDocStdPathParser};
use crate::live;
use rustycad_tkcdf::{
    cdf::{CdfApplicationServices, CdfStore},
    cdm::{CdmDocument, CdmDocumentLifecycle},
    pcdm::{PcdmReaderFilter, PcdmReaderStatus, PcdmStoreStatus},
};
use rustycad_tkernel::{
    message::{Gravity, ProgressRange},
    standard::{Failure, Handle},
    tcollection::ExtendedString,
};
use std::io;

/// The trek and the `name.extension` of a path, as Open and SaveAs split it.
fn directory_and_file(path: &ExtendedString) -> (ExtendedString, ExtendedString) {
    let parser = TDocStdPathParser::new(path);
    let mut file = parser.name();
    file.assign_cat(ExtendedString::new("."));
    file.assign_cat(parser.extension());
    (parser.trek(), file)
}

/// Whether a filter asks to append into an existing document.
fn is_append_mode(filter: &Handle<PcdmReaderFilter>) -> bool {
    filter.get().is_some_and(PcdmReaderFilter::is_append_mode)
}

/// The members of `TDocStd_Application` that open and save documents.
pub trait TDocStdApplicationIo: TDocStdApplication {
    /// Sends the text of a caught failure to the message driver as a failure.
    fn report_failure(&self, failure: &Failure) {
        let text = ExtendedString::from_c_string(failure.what(), false);
        self.message_driver().send_extended(&text, Gravity::Fail);
    }
    /// `Open(thePath, theDoc, theFilter, theRange)`: retrieves the document of a file and
    /// opens it in the session (unless appending); returns the retrieval status.
    fn open_path(
        &self,
        path: &ExtendedString,
        document: &mut Handle<TDocStdDocument>,
        filter: &Handle<PcdmReaderFilter>,
        range: &ProgressRange<'_>,
    ) -> Result<PcdmReaderStatus, Failure> {
        let (directory, file) = directory_and_file(path);
        let status = self.can_retrieve(&directory, &file, is_append_mode(filter))?;
        if status != PcdmReaderStatus::Ok {
            return Ok(status);
        }
        let retrieved = self
            .retrieve_document(&directory, &file, true, filter, range)
            .and_then(|retrieved| {
                let retrieved: Handle<TDocStdDocument> = retrieved.cast();
                if !is_append_mode(filter) {
                    self.open_document(&retrieved.cast())?;
                }
                Ok(retrieved)
            });
        match retrieved {
            Ok(retrieved) => *document = retrieved,
            Err(failure) => self.report_failure(&failure),
        }
        Ok(self.get_retrieve_status())
    }
    /// `Open(thePath, theDoc, theRange)`: without filter.
    fn open_path_without_filter(
        &self,
        path: &ExtendedString,
        document: &mut Handle<TDocStdDocument>,
        range: &ProgressRange<'_>,
    ) -> Result<PcdmReaderStatus, Failure> {
        self.open_path(path, document, &Handle::default(), range)
    }
    /// `Open(theIStream, theDoc, theFilter, theRange)`: reads a document from a stream.
    fn open_stream(
        &self,
        stream: &mut dyn io::Read,
        document: &mut Handle<TDocStdDocument>,
        filter: &Handle<PcdmReaderFilter>,
        range: &ProgressRange<'_>,
    ) -> PcdmReaderStatus {
        let mut read: Handle<dyn CdmDocument> = document.cast();
        let outcome = self
            .read_stream(stream, &mut read, filter, range)
            .and_then(|()| {
                *document = read.cast();
                if document.is_some() && !is_append_mode(filter) {
                    self.open_document(&document.cast())?;
                }
                Ok(())
            });
        if let Err(failure) = outcome {
            self.report_failure(&failure);
        }
        self.get_retrieve_status()
    }
    /// `Open(theIStream, theDoc, theRange)`: without filter.
    fn open_stream_without_filter(
        &self,
        stream: &mut dyn io::Read,
        document: &mut Handle<TDocStdDocument>,
        range: &ProgressRange<'_>,
    ) -> PcdmReaderStatus {
        self.open_stream(stream, document, &Handle::default(), range)
    }
    /// `SaveAs(theDoc, path, theRange)`: stores the document under a new path.
    fn save_as(
        &self,
        document: &Handle<TDocStdDocument>,
        path: &ExtendedString,
        range: &ProgressRange<'_>,
    ) -> Result<PcdmStoreStatus, Failure> {
        let mut message = ExtendedString::default();
        let (status, folder_found) = self.store_as(document, path, &mut message, range)?;
        let messenger = self.message_driver();
        if !folder_found {
            let (directory, _) = directory_and_file(path);
            let text = ExtendedString::new("TDocStd_Application::SaveAs() - folder ")
                .cat(directory)
                .cat(ExtendedString::new(" does not exist"));
            messenger.send_extended(&text, Gravity::Fail);
        } else if status != PcdmStoreStatus::Ok {
            messenger.send_extended(&message, Gravity::Fail);
        }
        Ok(status)
    }
    /// `SaveAs(theDoc, path, theStatusMessage, theRange)`.
    fn save_as_with_message(
        &self,
        document: &Handle<TDocStdDocument>,
        path: &ExtendedString,
        status_message: &mut ExtendedString,
        range: &ProgressRange<'_>,
    ) -> Result<PcdmStoreStatus, Failure> {
        let (status, folder_found) = self.store_as(document, path, status_message, range)?;
        if folder_found {
            return Ok(status);
        }
        let (directory, _) = directory_and_file(path);
        *status_message =
            ExtendedString::new("TDocStd_Application::SaveAs: No such directory ").cat(directory);
        Ok(PcdmStoreStatus::Failure)
    }
    /// The common part of both `SaveAs(path)`: opens the document here, sets the folder and
    /// name of a store and realizes it; returns the status and whether the folder exists.
    fn store_as(
        &self,
        document: &Handle<TDocStdDocument>,
        path: &ExtendedString,
        message: &mut ExtendedString,
        range: &ProgressRange<'_>,
    ) -> Result<(PcdmStoreStatus, bool), Failure> {
        let (directory, file) = directory_and_file(path);
        let target = live(document, "TDocStd_Application::SaveAs")?;
        target.open(&self.application_base().this()?);
        let mut storer = CdfStore::new(&document.cast())?;
        if !storer.set_folder(&directory)? {
            return Ok((storer.store_status(), false));
        }
        storer.set_name(&file)?;
        self.realize_store(&mut storer, document, message, range)?;
        Ok((storer.store_status(), true))
    }
    /// `Realize` with the native catch, `SetSaved` on success, and the status text.
    fn realize_store(
        &self,
        storer: &mut CdfStore,
        document: &Handle<TDocStdDocument>,
        message: &mut ExtendedString,
        range: &ProgressRange<'_>,
    ) -> Result<(), Failure> {
        if let Err(failure) = storer.realize(range) {
            self.report_failure(&failure);
        }
        if storer.store_status() == PcdmStoreStatus::Ok {
            live(document, "TDocStd_Application::SaveAs")?.set_saved()?;
        }
        *message = storer.associated_status_text();
        Ok(())
    }
    /// `SaveAs(theDoc, theOStream, theRange)`: writes the document with its format's writer.
    fn save_as_stream(
        &self,
        document: &Handle<TDocStdDocument>,
        stream: &mut dyn io::Write,
        range: &ProgressRange<'_>,
    ) -> PcdmStoreStatus {
        let mut message = ExtendedString::default();
        self.save_as_stream_with_message(document, stream, &mut message, range)
    }
    /// `SaveAs(theDoc, theOStream, theStatusMessage, theRange)`.
    fn save_as_stream_with_message(
        &self,
        document: &Handle<TDocStdDocument>,
        stream: &mut dyn io::Write,
        status_message: &mut ExtendedString,
        range: &ProgressRange<'_>,
    ) -> PcdmStoreStatus {
        let outcome = (|| -> Result<PcdmStoreStatus, Failure> {
            let target = live(document, "TDocStd_Application::SaveAs")?;
            let format = target.storage_format();
            let writer = self.writer_from_format(&format)?;
            let Some(writer) = writer.get() else {
                *status_message =
                    ExtendedString::new("TDocStd_Application::SaveAs: no storage driver");
                return Ok(PcdmStoreStatus::DriverFailure);
            };
            writer.set_format(&format);
            writer.write_stream(&document.cast(), stream, range)?;
            if writer.get_store_status() == PcdmStoreStatus::Ok {
                target.set_saved()?;
            }
            Ok(writer.get_store_status())
        })();
        outcome.unwrap_or_else(|failure| {
            self.report_failure(&failure);
            PcdmStoreStatus::Failure
        })
    }
    /// `Save(theDoc, theRange)`: stores a saved document again.
    fn save(
        &self,
        document: &Handle<TDocStdDocument>,
        range: &ProgressRange<'_>,
    ) -> Result<PcdmStoreStatus, Failure> {
        let mut message = ExtendedString::default();
        let saved = live(document, "TDocStd_Application::Save")?.is_saved();
        if !saved {
            self.message_driver().send_extended(
                &ExtendedString::new("Document has not been saved yet"),
                Gravity::Fail,
            );
            return Ok(PcdmStoreStatus::Failure);
        }
        let mut storer = CdfStore::new(&document.cast())?;
        self.realize_store(&mut storer, document, &mut message, range)?;
        Ok(storer.store_status())
    }
    /// `Save(theDoc, theStatusMessage, theRange)`.
    fn save_with_message(
        &self,
        document: &Handle<TDocStdDocument>,
        status_message: &mut ExtendedString,
        range: &ProgressRange<'_>,
    ) -> Result<PcdmStoreStatus, Failure> {
        if !live(document, "TDocStd_Application::Save")?.is_saved() {
            *status_message =
                ExtendedString::new("TDocStd_Application::the document has not been saved yet");
            return Ok(PcdmStoreStatus::Failure);
        }
        let mut storer = CdfStore::new(&document.cast())?;
        self.realize_store(&mut storer, document, status_message, range)?;
        Ok(storer.store_status())
    }
}
impl<T: TDocStdApplication + ?Sized> TDocStdApplicationIo for T {}
