//! Additional stream and parser edge cases beyond the legacy suite.
use rustycad_tkernel::standard::dump::{self, DumpOutput, DumpStream};
use std::io;
struct OpaqueDumpOutput {
    position: i64,
    written: Vec<u8>,
}
impl DumpOutput for OpaqueDumpOutput {
    fn position(&self) -> i64 {
        self.position
    }
    fn content(&self) -> Option<&[u8]> {
        None
    }
    fn write_fragment(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.written.extend_from_slice(bytes);
        Ok(())
    }
}
#[test]
fn stream_positions_and_opaque_outputs() {
    let mut stream = DumpStream::default();
    stream.append("content");
    stream.seek(0);
    dump::add_values_separator(&mut stream).unwrap();
    assert_eq!(stream.text(), "content");
    stream.seek(2);
    dump::add_values_separator(&mut stream).unwrap();
    assert_eq!(stream.text(), "co, ent");
    let mut opaque = OpaqueDumpOutput {
        position: 3,
        written: Vec::new(),
    };
    dump::add_values_separator(&mut opaque).unwrap();
    assert_eq!(opaque.written, b", ");
    opaque.position = -1;
    dump::add_values_separator(&mut opaque).unwrap();
    assert_eq!(opaque.written, b", ");
    let mut nul = DumpStream::default();
    nul.append("{\0ignored");
    dump::add_values_separator(&mut nul).unwrap();
    assert_eq!(nul.bytes(), b"{\0ignored");
}
#[test]
fn malformed_vectors_retain_partial_writes_and_cursor() {
    let mut values = [91.0, 92.0, 93.0];
    let mut position = 1;
    assert!(!dump::init_real_values("[1, bad, 3]", &mut position, &mut values).unwrap());
    assert_eq!(values, [1.0, 92.0, 93.0]);
    assert_eq!(position, 1);
    let error = dump::init_real_values("[1, 2, 3", &mut position, &mut values).unwrap_err();
    assert_eq!(values, [1.0, 2.0, 93.0]);
    assert_eq!(position, 1);
    assert_eq!(
        error.message(),
        "TCollection_AsciiString::SubString() out of range"
    );
    assert!(dump::init_real_values("[, 2, 3]", &mut position, &mut values).is_err());
}
#[test]
fn field_mismatches_and_scalar_validation() {
    let mut position = 1;
    assert!(!dump::process_stream_name(", wrong", "gp_Pnt", &mut position).unwrap());
    assert_eq!(position, 3);
    let mut integer = 17;
    position = 1;
    assert!(
        !dump::init_integer_field("\"Flags\": bad", "Flags", &mut position, &mut integer).unwrap()
    );
    assert_eq!((position, integer), (1, 17));
    let mut real = 12.0;
    assert!(!dump::init_real_field("\"Gap\": bad", "Gap", &mut position, &mut real).unwrap());
    assert_eq!(real, 12.0);
    assert!(dump::init_value("}tail", &mut position).is_err());
}
