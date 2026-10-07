// Rust port of the Read and Write members of DataExchange/TKDE/DE/DE_Wrapper.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The transfers of `DE_Wrapper`: find the provider of a path or of the first stream, then
//! hand the transfer to it. The overloads are named as those of `DeProvider`.
use super::{
    DeProvider, DeProviderReadStreamList, DeProviderWriteStreamList, DeValidationUtils,
    DeWorkSession, DeWrapper, live,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    message::{Message, ProgressRange},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use std::fmt::Write;

/// The context of the stream checks of the wrapper.
const READ_CONTEXT: &str = "DE_Wrapper Read";
/// The context of the stream checks of the wrapper.
const WRITE_CONTEXT: &str = "DE_Wrapper Write";

impl DeWrapper {
    /// `Read(thePath, theDocument, theWS, theProgress)`: `false` for a null document; a null
    /// work session reads without one ([`DeWrapper::read_document`]).
    pub fn read_document_with_session(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if document.is_null() {
            return Ok(false);
        }
        if work_session.is_null() {
            return self.read_document(path, document, progress);
        }
        let Some(provider) = self.path_provider(path, true)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_document_with_session(
            path,
            document,
            work_session,
            progress,
        )
    }

    /// `Write(thePath, theDocument, theWS, theProgress)`: `false` for a null document; a null
    /// work session writes without one ([`DeWrapper::write_document`]).
    pub fn write_document_with_session(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if document.is_null() {
            return Ok(false);
        }
        if work_session.is_null() {
            return self.write_document(path, document, progress);
        }
        let Some(provider) = self.path_provider(path, false)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_document_with_session(
            path,
            document,
            work_session,
            progress,
        )
    }

    /// `Read(thePath, theDocument, theProgress)`: `false` for a null document.
    pub fn read_document(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if document.is_null() {
            return Ok(false);
        }
        let Some(provider) = self.path_provider(path, true)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_document(path, document, progress)
    }

    /// `Write(thePath, theDocument, theProgress)`: `false` for a null document.
    pub fn write_document(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if document.is_null() {
            return Ok(false);
        }
        let Some(provider) = self.path_provider(path, false)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_document(path, document, progress)
    }

    /// `Read(thePath, theShape, theWS, theProgress)`: a null work session reads without one.
    pub fn read_shape_with_session(
        &self,
        path: &AsciiString,
        shape: &mut Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if work_session.is_null() {
            return self.read_shape(path, shape, progress);
        }
        let Some(provider) = self.path_provider(path, true)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_shape_with_session(
            path,
            shape,
            work_session,
            progress,
        )
    }

    /// `Write(thePath, theShape, theWS, theProgress)`: a null work session writes without one.
    pub fn write_shape_with_session(
        &self,
        path: &AsciiString,
        shape: &Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        if work_session.is_null() {
            return self.write_shape(path, shape, progress);
        }
        let Some(provider) = self.path_provider(path, false)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_shape_with_session(
            path,
            shape,
            work_session,
            progress,
        )
    }

    /// `Read(thePath, theShape, theProgress)`.
    pub fn read_shape(
        &self,
        path: &AsciiString,
        shape: &mut Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.path_provider(path, true)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_shape(path, shape, progress)
    }

    /// `Write(thePath, theShape, theProgress)`.
    pub fn write_shape(
        &self,
        path: &AsciiString,
        shape: &Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.path_provider(path, false)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_shape(path, shape, progress)
    }

    /// `Read(theStreams, theDocument, theWS, theProgress)`.
    pub fn read_document_streams_with_session(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_read_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_document_streams_with_session(
            streams,
            document,
            work_session,
            progress,
        )
    }

    /// `Write(theStreams, theDocument, theWS, theProgress)`.
    pub fn write_document_streams_with_session(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_write_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_document_streams_with_session(
            streams,
            document,
            work_session,
            progress,
        )
    }

    /// `Read(theStreams, theDocument, theProgress)`.
    pub fn read_document_streams(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_read_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_document_streams(streams, document, progress)
    }

    /// `Write(theStreams, theDocument, theProgress)`.
    pub fn write_document_streams(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_write_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_document_streams(streams, document, progress)
    }

    /// `Read(theStreams, theShape, theWS, theProgress)`.
    pub fn read_shape_streams_with_session(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        shape: &mut Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_read_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_shape_streams_with_session(
            streams,
            shape,
            work_session,
            progress,
        )
    }

    /// `Write(theStreams, theShape, theWS, theProgress)`.
    pub fn write_shape_streams_with_session(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        shape: &Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_write_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_shape_streams_with_session(
            streams,
            shape,
            work_session,
            progress,
        )
    }

    /// `Read(theStreams, theShape, theProgress)`.
    pub fn read_shape_streams(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        shape: &mut Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_read_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Read")?.read_shape_streams(streams, shape, progress)
    }

    /// `Write(theStreams, theShape, theProgress)`.
    pub fn write_shape_streams(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        shape: &Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let Some(provider) = self.stream_write_provider(streams)? else {
            return Ok(false);
        };
        live(&provider, "DE_Wrapper::Write")?.write_shape_streams(streams, shape, progress)
    }

    /// The `FindProvider` step of the path transfers.
    fn path_provider(
        &self,
        path: &AsciiString,
        to_import: bool,
    ) -> Result<Option<Handle<dyn DeProvider>>, Failure> {
        let mut provider: Handle<dyn DeProvider> = Handle::default();
        Ok(self
            .find_provider(path, to_import, &mut provider)?
            .then_some(provider))
    }

    /// The prelude of the stream reads: a valid list, a provider found by the first path and
    /// stream, and a node that supports streams.
    fn stream_read_provider(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
    ) -> Result<Option<Handle<dyn DeProvider>>, Failure> {
        if !DeValidationUtils::validate_read_stream_list(
            streams,
            &AsciiString::new(READ_CONTEXT),
            true,
        ) {
            return Ok(None);
        }
        let Some(first) = streams.first_mut() else {
            return Ok(None);
        };
        let mut provider: Handle<dyn DeProvider> = Handle::default();
        if !self.find_read_provider_from_stream(&first.path, &mut *first.stream, &mut provider)? {
            report_missing_provider(&first.path);
            return Ok(None);
        }
        streaming_provider(provider)
    }

    /// The prelude of the stream writes: a valid list, a provider found by the first path,
    /// and a node that supports streams.
    fn stream_write_provider(
        &self,
        streams: &DeProviderWriteStreamList<'_>,
    ) -> Result<Option<Handle<dyn DeProvider>>, Failure> {
        if !DeValidationUtils::validate_write_stream_list(
            streams,
            &AsciiString::new(WRITE_CONTEXT),
            true,
        ) {
            return Ok(None);
        }
        let Some(first) = streams.first() else {
            return Ok(None);
        };
        let mut provider: Handle<dyn DeProvider> = Handle::default();
        if !self.find_write_provider(&first.path, &mut provider) {
            report_missing_provider(&first.path);
            return Ok(None);
        }
        streaming_provider(provider)
    }
}

/// "Error: DE_Wrapper cannot find provider for stream PATH".
fn report_missing_provider(path: &AsciiString) {
    let mut stream = Message::send_fail();
    let _ = write!(
        stream,
        "Error: DE_Wrapper cannot find provider for stream {}",
        String::from_utf8_lossy(path.as_bytes())
    );
}

/// The provider when its node supports streams; otherwise a failure message and `None`.
/// A null provider or node raises `Standard_NullObject` (a native null dereference).
fn streaming_provider(
    provider: Handle<dyn DeProvider>,
) -> Result<Option<Handle<dyn DeProvider>>, Failure> {
    let object: &dyn DeProvider = live(&provider, "DE_Wrapper stream provider")?;
    let node = object.get_node();
    if live(&node, "DE_Provider::GetNode")?.is_stream_supported() {
        return Ok(Some(provider));
    }
    let mut stream = Message::send_fail();
    let _ = write!(
        stream,
        "Error: Provider {} {} doesn't support stream operations",
        String::from_utf8_lossy(object.get_format().as_bytes()),
        String::from_utf8_lossy(object.get_vendor().as_bytes())
    );
    Ok(None)
}
