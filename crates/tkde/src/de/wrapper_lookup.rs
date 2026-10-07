// Rust port of DE_Wrapper::FindProvider, FindReadProvider and FindWriteProvider of
// DataExchange/TKDE/DE/DE_Wrapper.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The provider lookup of `DE_Wrapper`: the first enabled node, in map order, whose format
//! accepts the file's extension (or content, for import) and that agrees to load.
use super::{DeConfigurationNode, DeProvider, DeValidationUtils, DeWrapper};
use rustycad_tkernel::{
    ncollection::Buffer,
    osd::{CadPath, IStream},
    standard::{Failure, Handle},
    tcollection::AsciiString,
};

/// What a lookup asks of a node: the direction and, for import, the content to check.
struct LookupRequest<'a> {
    extension: AsciiString,
    to_import: bool,
    content: Option<&'a Handle<Buffer>>,
}

impl DeWrapper {
    /// `FindProvider(thePath, theToImport, theProvider)`: [`DeWrapper::find_read_provider`]
    /// with a content check for import, [`DeWrapper::find_write_provider`] for export.
    pub fn find_provider(
        &self,
        path: &AsciiString,
        to_import: bool,
        provider: &mut Handle<dyn DeProvider>,
    ) -> Result<bool, Failure> {
        if to_import {
            return self.find_read_provider(path, true, provider);
        }
        Ok(self.find_write_provider(path, provider))
    }

    /// `FindReadProvider(thePath, theCheckContent, theProvider)`. With `check_content` the file
    /// must open (its first 2048 bytes are the content), else the lookup fails even when an
    /// extension matches, as natively.
    pub fn find_read_provider(
        &self,
        path: &AsciiString,
        check_content: bool,
        provider: &mut Handle<dyn DeProvider>,
    ) -> Result<bool, Failure> {
        let mut buffer: Handle<Buffer> = Handle::default();
        if check_content && !DeValidationUtils::create_content_buffer_from_path(path, &mut buffer)?
        {
            return Ok(false);
        }
        let request = LookupRequest {
            extension: extension_of(path),
            to_import: true,
            content: check_content.then_some(&buffer),
        };
        Ok(self.find_node_provider(&request, provider))
    }

    /// `FindReadProvider(thePath, theStream, theProvider)`: the extension of `path` or the
    /// content of `stream` (whose position is restored).
    pub fn find_read_provider_from_stream(
        &self,
        path: &AsciiString,
        stream: &mut dyn IStream,
        provider: &mut Handle<dyn DeProvider>,
    ) -> Result<bool, Failure> {
        let mut buffer: Handle<Buffer> = Handle::default();
        if !DeValidationUtils::create_content_buffer(stream, &mut buffer)? {
            return Ok(false);
        }
        let request = LookupRequest {
            extension: extension_of(path),
            to_import: true,
            content: Some(&buffer),
        };
        Ok(self.find_node_provider(&request, provider))
    }

    /// `FindWriteProvider(thePath, theProvider)`: by extension only.
    pub fn find_write_provider(
        &self,
        path: &AsciiString,
        provider: &mut Handle<dyn DeProvider>,
    ) -> bool {
        let request = LookupRequest {
            extension: extension_of(path),
            to_import: false,
            content: None,
        };
        self.find_node_provider(&request, provider)
    }

    /// The node loop shared by the lookups: the first accepting node builds the provider,
    /// then receives the wrapper's units (after `BuildProvider`, as natively).
    fn find_node_provider(
        &self,
        request: &LookupRequest<'_>,
        provider: &mut Handle<dyn DeProvider>,
    ) -> bool {
        let keep_updates: bool = self.keep_updates();
        for node in self.node_snapshot() {
            let Some(object) = node.get() else {
                continue;
            };
            if !accepts(object, request) || !object.update_load(request.to_import, keep_updates) {
                continue;
            }
            *provider = object.build_provider();
            object.set_global_parameters(self.global_parameters());
            return true;
        }
        false
    }
}

/// The node test of a lookup before `UpdateLoad`, in the native evaluation order.
fn accepts(node: &dyn DeConfigurationNode, request: &LookupRequest<'_>) -> bool {
    if !node.is_enabled() {
        return false;
    }
    let supported: bool = if request.to_import {
        node.is_import_supported()
    } else {
        node.is_export_supported()
    };
    if !supported {
        return false;
    }
    node.check_extension(&request.extension)
        || request
            .content
            .is_some_and(|buffer| node.check_content(buffer))
}

/// `OSD_Path(thePath).Extension()`: the extension with its dot.
fn extension_of(path: &AsciiString) -> AsciiString {
    CadPath::new(path.as_bytes()).extension()
}
