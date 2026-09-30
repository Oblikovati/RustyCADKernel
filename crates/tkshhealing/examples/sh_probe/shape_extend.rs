//! ShapeExtend probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_shapeextend.cxx.
use super::common::{Reader, attempt, end, flag, int, text};
use rustycad_tkernel::message::MsgFile;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkshhealing::shape_extend::{ShapeExtend, ShapeExtendStatus};

/// `flag status`: ShapeExtend::EncodeStatus(status), then DecodeStatus(flag, status).
pub fn status_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let bits = r.integer();
    let status = ShapeExtendStatus::from_index(r.integer()).expect("probe status enumerator");
    int(ShapeExtend::encode_status(status));
    flag(ShapeExtend::decode_status(bits, status));
    end();
}

/// `n (byte)...`: ShapeExtend::Init, then HasMsg and the UTF-8 text of Msg for the key.
pub fn message_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let bytes: Vec<u8> = (0..n).map(|_| r.integer() as u8).collect();
    let key = AsciiString::from_bytes(&bytes);
    attempt(ShapeExtend::init);
    let catalog = MsgFile::global();
    flag(catalog.has_msg(&key));
    text(catalog.msg(&key).to_utf8().as_bytes());
    end();
}
