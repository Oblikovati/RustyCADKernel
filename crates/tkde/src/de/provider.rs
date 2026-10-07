// Rust port of DataExchange/TKDE/DE/DE_Provider.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DE_Provider`: the one-time transfer object a configuration node builds. Every transfer
//! is unsupported by default; a format overrides the ones it implements.
use super::{DeConfigurationNode, DeProviderReadStreamList, DeProviderWriteStreamList};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient_interface,
    message::{Message, ProgressRange},
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use std::fmt::Write;

/// `occ::handle<XSControl_WorkSession>`: TKDE only forward-declares the work session, so
/// the port passes a handle to any transient; a provider casts it to the work session
/// class (de-design.md section 2).
pub type DeWorkSession = Handle<StandardTransient>;

/// The private `myNode` of `DE_Provider`; `DeProviderBase::default()` is the default
/// constructor (no node), [`DeProviderBase::with_node`] the one from a node.
#[derive(Default)]
pub struct DeProviderBase {
    node: SharedState<Handle<dyn DeConfigurationNode>>,
}
impl DeProviderBase {
    /// `DE_Provider(theNode)`; `DeProviderBase::with_node(&node)`.
    pub fn with_node(node: &Handle<dyn DeConfigurationNode>) -> Self {
        Self {
            node: SharedState::new(node.clone()),
        }
    }
    /// `GetNode()`; `base.node()`.
    pub fn node(&self) -> Handle<dyn DeConfigurationNode> {
        self.node.read().clone()
    }
    /// `SetNode(theNode)`; `base.set_node(&node)`.
    pub fn set_node(&self, node: &Handle<dyn DeConfigurationNode>) {
        *self.node.write() = node.clone();
    }
}

/// `DE_Provider`: reads or writes files and streams of one format and vendor.
///
/// The sixteen `Read`/`Write` overloads are named by what they transfer (`document` or
/// `shape`), from where (`path` or `streams`) and whether they take a work session. Their
/// native default reports "doesn't support" and returns `false`.
pub trait DeProvider: Transient {
    /// The embedded base with the native private field.
    fn provider_base(&self) -> &DeProviderBase;
    /// `GetFormat()` (pure virtual).
    fn get_format(&self) -> AsciiString;
    /// `GetVendor()` (pure virtual).
    fn get_vendor(&self) -> AsciiString;

    /// `GetNode()`: the configuration node of the provider.
    fn get_node(&self) -> Handle<dyn DeConfigurationNode> {
        self.provider_base().node()
    }
    /// `SetNode(theNode)`.
    fn set_node(&self, node: &Handle<dyn DeConfigurationNode>) {
        self.provider_base().set_node(node);
    }

    /// `Read(thePath, theDocument, theWS, theProgress)`.
    fn read_document_with_session(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, document, work_session, progress);
        Ok(report_unsupported(self, "read operation"))
    }
    /// `Write(thePath, theDocument, theWS, theProgress)`.
    fn write_document_with_session(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, document, work_session, progress);
        Ok(report_unsupported(self, "write operation"))
    }
    /// `Read(theStreams, theDocument, theWS, theProgress)`.
    fn read_document_streams_with_session(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, document, work_session, progress);
        Ok(report_unsupported(self, "stream read operation"))
    }
    /// `Write(theStreams, theDocument, theWS, theProgress)`.
    fn write_document_streams_with_session(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, document, work_session, progress);
        Ok(report_unsupported(self, "stream write operation"))
    }
    /// `Read(thePath, theDocument, theProgress)`.
    fn read_document(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, document, progress);
        Ok(report_unsupported(self, "read operation"))
    }
    /// `Write(thePath, theDocument, theProgress)`.
    fn write_document(
        &self,
        path: &AsciiString,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, document, progress);
        Ok(report_unsupported(self, "write operation"))
    }
    /// `Read(theStreams, theDocument, theProgress)`.
    fn read_document_streams(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, document, progress);
        Ok(report_unsupported(self, "stream read operation"))
    }
    /// `Write(theStreams, theDocument, theProgress)`.
    fn write_document_streams(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        document: &Handle<TDocStdDocument>,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, document, progress);
        Ok(report_unsupported(self, "stream write operation"))
    }
    /// `Read(thePath, theShape, theWS, theProgress)`.
    fn read_shape_with_session(
        &self,
        path: &AsciiString,
        shape: &mut Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, shape, work_session, progress);
        Ok(report_unsupported(self, "read operation"))
    }
    /// `Write(thePath, theShape, theWS, theProgress)`.
    fn write_shape_with_session(
        &self,
        path: &AsciiString,
        shape: &Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, shape, work_session, progress);
        Ok(report_unsupported(self, "write operation"))
    }
    /// `Read(theStreams, theShape, theWS, theProgress)`.
    fn read_shape_streams_with_session(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        shape: &mut Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, shape, work_session, progress);
        Ok(report_unsupported(self, "stream read operation"))
    }
    /// `Write(theStreams, theShape, theWS, theProgress)`.
    fn write_shape_streams_with_session(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        shape: &Shape,
        work_session: &mut DeWorkSession,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, shape, work_session, progress);
        Ok(report_unsupported(self, "stream write operation"))
    }
    /// `Read(thePath, theShape, theProgress)`.
    fn read_shape(
        &self,
        path: &AsciiString,
        shape: &mut Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, shape, progress);
        Ok(report_unsupported(self, "read operation"))
    }
    /// `Write(thePath, theShape, theProgress)`.
    fn write_shape(
        &self,
        path: &AsciiString,
        shape: &Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (path, shape, progress);
        Ok(report_unsupported(self, "write operation"))
    }
    /// `Read(theStreams, theShape, theProgress)`.
    fn read_shape_streams(
        &self,
        streams: &mut DeProviderReadStreamList<'_>,
        shape: &mut Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, shape, progress);
        Ok(report_unsupported(self, "stream read operation"))
    }
    /// `Write(theStreams, theShape, theProgress)`.
    fn write_shape_streams(
        &self,
        streams: &mut DeProviderWriteStreamList<'_>,
        shape: &Shape,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        let _ = (streams, shape, progress);
        Ok(report_unsupported(self, "stream write operation"))
    }
}
impl_transient_interface!(dyn DeProvider, "DE_Provider", StandardTransient);

/// The default transfer body: "Error: provider FORMAT VENDOR doesn't support OPERATION",
/// then `false`.
fn report_unsupported<P: DeProvider + ?Sized>(provider: &P, operation: &str) -> bool {
    let mut stream = Message::send_fail();
    let _ = write!(
        stream,
        "Error: provider {} {} doesn't support {operation}",
        String::from_utf8_lossy(provider.get_format().as_bytes()),
        String::from_utf8_lossy(provider.get_vendor().as_bytes())
    );
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::impl_transient;

    /// A provider that implements no transfer.
    #[derive(Default)]
    struct NamedProvider {
        transient: StandardTransient,
        base: DeProviderBase,
    }
    impl_transient!(NamedProvider, "NamedProvider", dyn DeProvider, transient, interfaces: [dyn DeProvider]);
    impl DeProvider for NamedProvider {
        fn provider_base(&self) -> &DeProviderBase {
            &self.base
        }
        fn get_format(&self) -> AsciiString {
            AsciiString::new("FMT")
        }
        fn get_vendor(&self) -> AsciiString {
            AsciiString::new("VND")
        }
    }

    #[test]
    fn every_default_transfer_is_unsupported() {
        let provider = NamedProvider::default();
        let range = ProgressRange::default();
        let path = AsciiString::new("a.fmt");
        let mut shape = Shape::default();
        let mut session = DeWorkSession::default();
        let document: Handle<TDocStdDocument> = Handle::default();
        assert_eq!(provider.read_shape(&path, &mut shape, &range), Ok(false));
        assert_eq!(provider.write_shape(&path, &shape, &range), Ok(false));
        assert_eq!(
            provider.read_document_with_session(&path, &document, &mut session, &range),
            Ok(false)
        );
        let mut streams = DeProviderReadStreamList::default();
        assert_eq!(
            provider.read_shape_streams(&mut streams, &mut shape, &range),
            Ok(false)
        );
        assert!(provider.get_node().is_null());
    }
}
