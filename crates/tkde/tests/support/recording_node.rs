//! `RecordingNode` and `RecordingProvider`: a format node whose provider records which
//! transfer the wrapper handed to it, to check the dispatch of `DE_Wrapper::Read`/`Write`.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkde::de::{
    DeConfigurationContext, DeConfigurationNode, DeConfigurationNodeBase, DeProvider,
    DeProviderBase, DeProviderReadStreamList, DeProviderWriteStreamList, DeWorkSession,
    new_configuration_node,
};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    ncollection::List,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use std::sync::Arc;

/// The transfers a provider received, in call order.
pub type TransferLog = Arc<SharedState<Vec<&'static str>>>;

/// A provider that accepts every shape and stream transfer and records it.
pub struct RecordingProvider {
    transient: StandardTransient,
    base: DeProviderBase,
    log: TransferLog,
}
impl_transient!(RecordingProvider, "RecordingProvider", dyn DeProvider, transient, interfaces: [dyn DeProvider]);
impl RecordingProvider {
    fn record(&self, transfer: &'static str) -> Result<bool, Failure> {
        self.log.write().push(transfer);
        Ok(true)
    }
}
impl DeProvider for RecordingProvider {
    fn provider_base(&self) -> &DeProviderBase {
        &self.base
    }
    fn get_format(&self) -> AsciiString {
        AsciiString::new("REC")
    }
    fn get_vendor(&self) -> AsciiString {
        AsciiString::new("TEST")
    }
    fn read_shape(
        &self,
        _: &AsciiString,
        _: &mut Shape,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.record("read_shape")
    }
    fn write_shape(
        &self,
        _: &AsciiString,
        _: &Shape,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.record("write_shape")
    }
    fn write_shape_with_session(
        &self,
        _: &AsciiString,
        _: &Shape,
        _: &mut DeWorkSession,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.record("write_shape_with_session")
    }
    fn read_shape_streams(
        &self,
        _: &mut DeProviderReadStreamList<'_>,
        _: &mut Shape,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.record("read_shape_streams")
    }
    fn write_shape_streams(
        &self,
        _: &mut DeProviderWriteStreamList<'_>,
        _: &Shape,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.record("write_shape_streams")
    }
    fn write_document(
        &self,
        _: &AsciiString,
        _: &Handle<TDocStdDocument>,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.record("write_document")
    }
}

/// A node of format `REC` (extension `rec`) whose stream support is configurable.
pub struct RecordingNode {
    transient: StandardTransient,
    base: DeConfigurationNodeBase,
    streams: bool,
    log: TransferLog,
}
impl_transient!(RecordingNode, "RecordingNode", dyn DeConfigurationNode, transient, interfaces: [dyn DeConfigurationNode]);

impl RecordingNode {
    /// A bound node and the log its providers write to.
    pub fn create(streams: bool) -> (Handle<RecordingNode>, TransferLog) {
        let log: TransferLog = Arc::new(SharedState::new(Vec::new()));
        let node = RecordingNode {
            transient: StandardTransient::default(),
            base: DeConfigurationNodeBase::default(),
            streams,
            log: log.clone(),
        };
        (new_configuration_node(node), log)
    }
}

impl DeConfigurationNode for RecordingNode {
    fn configuration_node_base(&self) -> &DeConfigurationNodeBase {
        &self.base
    }
    fn load(&self, _: &Handle<DeConfigurationContext>) -> Result<bool, Failure> {
        Ok(true)
    }
    fn save(&self) -> AsciiString {
        AsciiString::default()
    }
    fn build_provider(&self) -> Handle<dyn DeProvider> {
        let provider = RecordingProvider {
            transient: StandardTransient::default(),
            base: DeProviderBase::with_node(&self.base.this().unwrap_or_default()),
            log: self.log.clone(),
        };
        Handle::new(provider).cast()
    }
    fn copy(&self) -> Handle<dyn DeConfigurationNode> {
        RecordingNode::create(self.streams).0.cast()
    }
    fn get_format(&self) -> AsciiString {
        AsciiString::new("REC")
    }
    fn get_vendor(&self) -> AsciiString {
        AsciiString::new("TEST")
    }
    fn get_extensions(&self) -> List<AsciiString> {
        let mut list: List<AsciiString> = List::default();
        list.append(AsciiString::new("rec"));
        list
    }
    fn is_import_supported(&self) -> bool {
        true
    }
    fn is_export_supported(&self) -> bool {
        true
    }
    fn is_stream_supported(&self) -> bool {
        self.streams
    }
}
