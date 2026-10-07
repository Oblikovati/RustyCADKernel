//! The transfer dispatch of `DE_Wrapper`: which provider member a `Read`/`Write` reaches,
//! the null-document and null-session shortcuts, and the stream-support check.
mod support;

use rustycad_tkbrep::topods::Shape;
use rustycad_tkde::de::{
    DeProviderReadStreamList, DeProviderReadStreamNode, DeProviderWriteStreamList,
    DeProviderWriteStreamNode, DeWorkSession, DeWrapper,
};
use rustycad_tkernel::{
    message::ProgressRange,
    standard::{Handle, StandardTransient},
    tcollection::AsciiString,
};
use rustycad_tklcaf::tdoc_std::TDocStdDocument;
use support::memory_stream::{memory_istream, memory_ostream};
use support::recording_node::{RecordingNode, TransferLog};

fn wrapper(streams: bool) -> (DeWrapper, TransferLog) {
    let (node, log) = RecordingNode::create(streams);
    let wrapper = DeWrapper::new();
    assert!(wrapper.bind(&node.cast()));
    (wrapper, log)
}

fn path(text: &str) -> AsciiString {
    AsciiString::new(text)
}

#[test]
fn path_transfers_reach_the_provider_of_the_extension() {
    let (wrapper, log) = wrapper(false);
    let range = ProgressRange::default();
    let shape = Shape::default();
    assert_eq!(
        wrapper.write_shape(&path("out.rec"), &shape, &range),
        Ok(true)
    );
    assert_eq!(
        wrapper.write_shape(&path("out.other"), &shape, &range),
        Ok(false)
    );
    let mut session: DeWorkSession = Handle::new(StandardTransient::default());
    assert_eq!(
        wrapper.write_shape_with_session(&path("a.rec"), &shape, &mut session, &range),
        Ok(true)
    );
    let mut null_session = DeWorkSession::default();
    assert_eq!(
        wrapper.write_shape_with_session(&path("a.rec"), &shape, &mut null_session, &range),
        Ok(true)
    );
    assert_eq!(
        *log.read(),
        ["write_shape", "write_shape_with_session", "write_shape"]
    );
}

#[test]
fn a_read_by_path_needs_a_file_to_check_the_content() {
    let (wrapper, log) = wrapper(false);
    let mut shape = Shape::default();
    let range = ProgressRange::default();
    assert_eq!(
        wrapper.read_shape(&path("/no/such/file.rec"), &mut shape, &range),
        Ok(false)
    );
    assert!(log.read().is_empty());
}

#[test]
fn a_null_document_is_refused_before_any_lookup() {
    let (wrapper, log) = wrapper(false);
    let range = ProgressRange::default();
    let document: Handle<TDocStdDocument> = Handle::default();
    assert_eq!(
        wrapper.write_document(&path("a.rec"), &document, &range),
        Ok(false)
    );
    let mut session = DeWorkSession::default();
    let written =
        wrapper.write_document_with_session(&path("a.rec"), &document, &mut session, &range);
    assert_eq!(written, Ok(false));
    assert!(log.read().is_empty());
}

#[test]
fn stream_transfers_need_a_node_that_supports_streams() {
    let range = ProgressRange::default();
    let mut shape = Shape::default();
    for (streams, expected) in [(true, true), (false, false)] {
        let (wrapper, log) = wrapper(streams);
        let mut input = memory_istream(b"data");
        let mut inputs = DeProviderReadStreamList::default();
        inputs.append(DeProviderReadStreamNode::new(&path("in.rec"), &mut input));
        assert_eq!(
            wrapper.read_shape_streams(&mut inputs, &mut shape, &range),
            Ok(expected)
        );
        let mut output = memory_ostream();
        let mut outputs = DeProviderWriteStreamList::default();
        outputs.append(DeProviderWriteStreamNode::new(
            &path("out.rec"),
            &mut output,
        ));
        assert_eq!(
            wrapper.write_shape_streams(&mut outputs, &shape, &range),
            Ok(expected)
        );
        let calls: Vec<&str> = if expected {
            vec!["read_shape_streams", "write_shape_streams"]
        } else {
            vec![]
        };
        assert_eq!(*log.read(), calls);
    }
}

#[test]
fn stream_transfers_without_a_provider_or_streams_fail() {
    let (wrapper, log) = wrapper(true);
    let range = ProgressRange::default();
    let mut shape = Shape::default();
    let mut empty = DeProviderReadStreamList::default();
    assert_eq!(
        wrapper.read_shape_streams(&mut empty, &mut shape, &range),
        Ok(false)
    );
    let mut input = memory_istream(b"data");
    let mut unknown = DeProviderReadStreamList::default();
    unknown.append(DeProviderReadStreamNode::new(&path("in.zzz"), &mut input));
    assert_eq!(
        wrapper.read_shape_streams(&mut unknown, &mut shape, &range),
        Ok(false)
    );
    assert!(log.read().is_empty());
}
