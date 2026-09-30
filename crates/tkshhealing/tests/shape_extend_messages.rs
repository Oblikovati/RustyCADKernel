//! Behavior tests of ShapeExtend::Init, the SHMessage resource and the message registrators.
mod support;
use rustycad_tkernel::message::{Gravity, Msg, MsgFile, MsgFileSource};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkshhealing::sh_message::SHAPE_US;
use rustycad_tkshhealing::shape_extend::{
    BasicMsgRegistrator, ShapeExtend, ShapeExtendBasicMsgRegistrator, ShapeExtendMsgRegistrator,
    TransientKey,
};
use std::sync::Arc;

/// A message source with at most one file, `<dir>/SHAPE.us`, found through `CSF_SHMessage`.
struct FakeMessageSource {
    shape_file: Option<Vec<u8>>,
}

impl MsgFileSource for FakeMessageSource {
    fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        (path == "/res/SHAPE.us")
            .then(|| self.shape_file.clone())
            .flatten()
    }
    fn variable(&self, name: &str) -> Option<String> {
        (name == "CSF_SHMessage" && self.shape_file.is_some()).then(|| "/res".to_owned())
    }
}

fn catalog(shape_file: Option<&[u8]>) -> MsgFile {
    MsgFile::new(Arc::new(FakeMessageSource {
        shape_file: shape_file.map(<[u8]>::to_vec),
    }))
}

#[test]
fn the_compiled_resource_is_loaded_without_a_message_directory() {
    let messages = catalog(None);
    ShapeExtend::load_messages(&messages).unwrap();
    let key = AsciiString::new("ShapeFix.FixSmallSolid.MSG0");
    assert!(messages.has_msg(&key));
    assert!(!SHAPE_US.is_empty());
}

#[test]
fn a_message_file_replaces_the_compiled_resource_and_must_hold_the_check_key() {
    let messages = catalog(Some(b".ShapeFix.FixSmallSolid.MSG0\nfrom the file\n"));
    ShapeExtend::load_messages(&messages).unwrap();
    let text = messages.msg(&AsciiString::new("ShapeFix.FixSmallSolid.MSG0"));
    assert_eq!(text.to_utf8(), "from the file");
    let invalid = catalog(Some(b".Other.MSG0\ntext\n"));
    let failure = ShapeExtend::load_messages(&invalid).unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
}

#[test]
fn init_loads_the_global_catalog_once() {
    ShapeExtend::init().unwrap();
    ShapeExtend::init().unwrap();
    assert!(MsgFile::global().has_msg(&AsciiString::new("ShapeFix.FixSmallSolid.MSG0")));
}

fn text_message(text: &str) -> Msg {
    let mut msg = Msg::new();
    msg.set_text(text.as_bytes());
    msg
}

#[test]
fn the_registrator_keeps_messages_per_object_and_per_shape() {
    let registrator = ShapeExtendMsgRegistrator::new();
    let object: Handle<StandardTransient> = Handle::new(StandardTransient::default());
    let edge = support::square_edges().remove(0);
    registrator.send_transient(&object, &text_message("a"), Gravity::Info);
    registrator.send_transient(&object, &text_message("b"), Gravity::Fail);
    registrator.send_shape(&edge.reversed(), &text_message("c"), Gravity::Warning);
    registrator.send(&text_message("dropped"), Gravity::Alarm);
    let texts = registrator.map_transient(|map| {
        let list = map.seek(&TransientKey(object.clone())).unwrap();
        list.iter().map(|m| m.value().to_utf8()).collect::<Vec<_>>()
    });
    assert_eq!(texts, ["a", "b"]);
    assert!(registrator.map_shape(|map| map.seek(&edge).is_some()));
}

#[test]
fn the_basic_registrator_drops_everything() {
    let registrator: Handle<dyn BasicMsgRegistrator> =
        Handle::new(ShapeExtendBasicMsgRegistrator::new()).cast();
    registrator.send(&text_message("x"), Gravity::Trace);
    assert!(registrator.cast::<ShapeExtendMsgRegistrator>().is_null());
    assert!(registrator.is_kind_name("ShapeExtend_BasicMsgRegistrator"));
}
