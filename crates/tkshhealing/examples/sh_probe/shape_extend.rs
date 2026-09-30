//! ShapeExtend probe operations, the Rust twin of
//! tools/oracle/tkshhealing_probe_shapeextend.cxx.
use super::common::{Reader, end, flag, int};
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
