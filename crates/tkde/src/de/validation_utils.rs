// Rust port of DataExchange/TKDE/DE/DE_ValidationUtils.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_ValidationUtils`: the checks providers share (node type, files, streams, document)
//! and the content buffer that format detection reads.
use super::{DeConfigurationNode, DeProviderReadStreamList, DeProviderWriteStreamList};
use rustycad_tkernel::{
    message::Message,
    ncollection::{BaseAllocator, Buffer},
    osd::{File, FileStream, IStream, IosOpenMode, SeekDir, StreamAccess, default_file_system},
    standard::{Failure, Handle, RuntimeType},
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use std::fmt::Write;
use std::path::PathBuf;

/// The bytes a content buffer holds (`aBufferLength`).
const CONTENT_BUFFER_LENGTH: usize = 2048;

/// `DE_ValidationUtils`: static checks with optional failure messages.
///
/// ```
/// use rustycad_tkde::de::DeValidationUtils;
/// assert!(!DeValidationUtils::validate_file_for_reading(&"".into(), &"Read".into(), false));
/// ```
pub struct DeValidationUtils;

impl DeValidationUtils {
    /// `ValidateConfigurationNode`: the node is not null and is a kind of `expected_type`.
    pub fn validate_configuration_node(
        node: &Handle<dyn DeConfigurationNode>,
        expected_type: &RuntimeType,
        context: &AsciiString,
        is_verbose: bool,
    ) -> bool {
        let Some(dynamic_type) = node.dynamic_type() else {
            fail(is_verbose, context, "Configuration Node is null");
            return false;
        };
        if node.is_kind(expected_type) {
            return true;
        }
        let detail: String = format!(
            "Configuration Node is not of expected type. Expected: {}, got: {}",
            expected_type.name(),
            dynamic_type.name()
        );
        fail(is_verbose, context, &detail);
        false
    }

    /// `ValidateFileForReading`: the path is not empty, the file exists and opens for reading.
    pub fn validate_file_for_reading(
        path: &AsciiString,
        context: &AsciiString,
        is_verbose: bool,
    ) -> bool {
        if path.is_empty() {
            fail(is_verbose, context, "File path is empty");
            return false;
        }
        let text: String = lossy(path);
        if !File::new(path.as_bytes()).exists() {
            fail(
                is_verbose,
                context,
                &format!("File '{text}' does not exist"),
            );
            return false;
        }
        if !opens(path, IosOpenMode::IN, FileStream::input()) {
            fail(
                is_verbose,
                context,
                &format!("Cannot open file '{text}' for reading"),
            );
            return false;
        }
        true
    }

    /// `ValidateFileForWriting`: the path is not empty and opens for appending (which creates
    /// a missing file, as the native `std::ofstream` does).
    pub fn validate_file_for_writing(
        path: &AsciiString,
        context: &AsciiString,
        is_verbose: bool,
    ) -> bool {
        if path.is_empty() {
            fail(is_verbose, context, "File path is empty");
            return false;
        }
        if !opens(
            path,
            IosOpenMode::OUT | IosOpenMode::APP,
            FileStream::output(),
        ) {
            let detail: String = format!("Cannot open file '{}' for writing", lossy(path));
            fail(is_verbose, context, &detail);
            return false;
        }
        true
    }

    /// `ValidateReadStreamList`: at least one stream (more is a warning) and a first stream
    /// that has not failed.
    pub fn validate_read_stream_list(
        streams: &DeProviderReadStreamList<'_>,
        context: &AsciiString,
        is_verbose: bool,
    ) -> bool {
        let Some(first) = streams.first() else {
            fail(is_verbose, context, "Stream list is empty");
            return false;
        };
        warn_extra_streams(streams.len(), context, is_verbose);
        valid_stream(
            first.stream.fail(),
            &first.path,
            "Input",
            context,
            is_verbose,
        )
    }

    /// `ValidateWriteStreamList`: at least one stream (more is a warning) and a first stream
    /// that has not failed.
    pub fn validate_write_stream_list(
        streams: &DeProviderWriteStreamList<'_>,
        context: &AsciiString,
        is_verbose: bool,
    ) -> bool {
        let Some(first) = streams.first() else {
            fail(is_verbose, context, "Stream list is empty");
            return false;
        };
        warn_extra_streams(streams.len(), context, is_verbose);
        valid_stream(
            first.stream.fail(),
            &first.path,
            "Output",
            context,
            is_verbose,
        )
    }

    /// `ValidateDocument`: the document handle is not null.
    pub fn validate_document(
        document: &Handle<TDocStdDocument>,
        context: &AsciiString,
        is_verbose: bool,
    ) -> bool {
        if document.is_null() {
            fail(is_verbose, context, "Document handle is null");
            return false;
        }
        true
    }

    /// `WarnLengthUnitNotSupported`: warns when the unit is not 1; always `true`.
    pub fn warn_length_unit_not_supported(
        length_unit: f64,
        context: &AsciiString,
        is_verbose: bool,
    ) -> bool {
        if is_verbose && length_unit != 1.0 {
            let mut stream = Message::send_warning();
            let _ = write!(
                stream,
                "Warning during {}: Format doesn't support custom length unit scaling (unit: {})",
                lossy(context),
                lossy(&AsciiString::from_real(length_unit))
            );
        }
        true
    }

    /// `CreateContentBuffer(thePath, theBuffer)`: the first bytes of a file through the
    /// default file system; a file that does not open nullifies `buffer` and gives `false`.
    pub fn create_content_buffer_from_path(
        path: &AsciiString,
        buffer: &mut Handle<Buffer>,
    ) -> Result<bool, Failure> {
        let mode: IosOpenMode = IosOpenMode::IN | IosOpenMode::BINARY;
        let Some(stream) = default_file_system().open_istream(path, mode, 0, None)? else {
            buffer.nullify();
            return Ok(false);
        };
        let mut guard = stream
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::create_content_buffer(&mut *guard, buffer)
    }

    /// `CreateContentBuffer(theStream, theBuffer)`: reads up to 2048 bytes into a new buffer,
    /// writes a NUL after them (over the last byte of a full buffer), then clears the stream
    /// state and seeks back to where it was.
    pub fn create_content_buffer(
        stream: &mut dyn IStream,
        buffer: &mut Handle<Buffer>,
    ) -> Result<bool, Failure> {
        let mut content: Buffer =
            Buffer::new(Some(BaseAllocator::common()), CONTENT_BUFFER_LENGTH)?;
        let original_position: i64 = stream.tellg();
        let bytes: &mut [u8] = content.as_mut_slice();
        let read: usize = stream.read(bytes);
        bytes[read.min(CONTENT_BUFFER_LENGTH - 1)] = 0;
        stream.clear();
        stream.seekg(original_position, SeekDir::Beg);
        *buffer = Handle::new(content);
        Ok(true)
    }
}

/// `Message::SendFail() << "Error during " << context << ": " << detail` when verbose.
fn fail(is_verbose: bool, context: &AsciiString, detail: &str) {
    if !is_verbose {
        return;
    }
    let mut stream = Message::send_fail();
    let _ = write!(stream, "Error during {}: {detail}", lossy(context));
}

/// The warning of a list with more than one stream.
fn warn_extra_streams(count: usize, context: &AsciiString, is_verbose: bool) {
    if !is_verbose || count <= 1 {
        return;
    }
    let mut stream = Message::send_warning();
    let _ = write!(
        stream,
        "Warning during {}: Received {count} streams, using only the first one",
        lossy(context)
    );
}

/// The state check of the first stream of a list.
fn valid_stream(
    failed: bool,
    path: &AsciiString,
    side: &str,
    context: &AsciiString,
    is_verbose: bool,
) -> bool {
    if !failed {
        return true;
    }
    let key: String = if path.is_empty() {
        "<empty path>".to_owned()
    } else {
        lossy(path)
    };
    fail(
        is_verbose,
        context,
        &format!("{side} stream '{key}' is in invalid state"),
    );
    false
}

/// Opens `path` with a file stream as `std::ifstream`/`std::ofstream` do, and tells whether
/// it is open and good.
fn opens(path: &AsciiString, mode: IosOpenMode, mut stream: FileStream) -> bool {
    stream.open(&PathBuf::from(lossy(path)), mode);
    stream.is_open() && stream.good()
}

/// The text of a native string for a message or a path.
fn lossy(text: &AsciiString) -> String {
    String::from_utf8_lossy(text.as_bytes()).into_owned()
}
