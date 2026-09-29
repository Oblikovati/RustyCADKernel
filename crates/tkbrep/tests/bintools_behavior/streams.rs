//! The static BinTools scalar encodings and the BinTools_OStream / BinTools_IStream
//! primitives.
use super::fakes::written;
use rustycad_tkbrep::bintools::{self, BinIStream, BinOStream, ObjectType};
use rustycad_tkernel::storage::{IoStream, state};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Transform3;

#[test]
fn scalars_are_written_in_the_platform_byte_order_and_read_back() {
    let bytes = written(|stream| {
        bintools::put_real(stream, 1.5);
        bintools::put_short_real(stream, -2.0);
        bintools::put_integer(stream, -7);
        bintools::put_bool(stream, true);
        bintools::put_ext_char(stream, 0x1234);
    });
    let mut expected = 1.5_f64.to_ne_bytes().to_vec();
    expected.extend((-2.0_f32).to_ne_bytes());
    expected.extend((-7_i32).to_ne_bytes());
    expected.push(1);
    expected.extend(0x1234_u16.to_ne_bytes());
    assert_eq!(bytes, expected);
    let mut stream = IoStream::from_bytes(bytes);
    assert_eq!(bintools::get_real(&mut stream).unwrap(), 1.5);
    assert_eq!(bintools::get_short_real(&mut stream).unwrap(), -2.0);
    assert_eq!(bintools::get_integer(&mut stream).unwrap(), -7);
    assert!(bintools::get_bool(&mut stream));
    assert_eq!(bintools::get_ext_char(&mut stream).unwrap(), 0x1234);
}

#[test]
fn a_short_read_raises_the_stream_type_mismatch_but_a_bool_past_the_end_is_true() {
    let mut stream = IoStream::from_bytes(vec![1, 2, 3]);
    let error = bintools::get_integer(&mut stream).unwrap_err();
    assert_eq!(error.exception_type(), "Storage_StreamTypeMismatchError");
    // `IS.get() != 0` with get() returning EOF (-1).
    assert!(bintools::get_bool(&mut stream));
    assert_eq!(stream.rdstate(), state::EOF | state::FAIL);
}

#[test]
fn references_take_the_narrowest_width_for_their_delta() {
    let cases: [(u64, u64, &[u8]); 4] = [
        (10, 7, &[1, 3]),
        (300, 0, &[2, 44, 1]),
        (70000, 0, &[3, 0x70, 0x11, 1, 0]),
        (5, 6, &[4, 255, 255, 255, 255, 255, 255, 255, 255]),
    ];
    for (prefix, target, expected) in cases {
        let bytes = written(|stream| {
            stream.write(&vec![0; prefix as usize]);
            let mut out = BinOStream::new(stream);
            out.write_reference(target);
            assert_eq!(out.position(), prefix + expected.len() as u64);
        });
        assert_eq!(&bytes[prefix as usize..], expected);
    }
}

#[test]
fn a_shape_record_starts_with_its_type_and_orientation_code() {
    let bytes = written(|stream| {
        let mut out = BinOStream::new(stream);
        out.write_shape(ShapeEnum::Edge, Orientation::Reversed);
        out.write_object_type(ObjectType::END_SHAPE);
        out.put_bools3(true, false, true);
        out.put_bools7([false, true, false, false, false, false, true]);
    });
    assert_eq!(bytes, [225, 199, 5, 66]);
    let mut stream = IoStream::from_bytes(bytes);
    let mut input = BinIStream::new(&mut stream);
    input.read_type();
    assert_eq!(input.shape_type(), ShapeEnum::Edge as i32);
    assert_eq!(input.shape_orientation(), Orientation::Reversed);
    assert_eq!(input.read_type(), ObjectType::END_SHAPE);
    assert_eq!(input.read_bools3(), [true, false, true]);
    assert!(input.read_bools7()[6]);
}

#[test]
fn a_type_below_the_shape_codes_decodes_to_a_negative_shape_type() {
    let mut stream = IoStream::from_bytes(vec![5]);
    let mut input = BinIStream::new(&mut stream);
    input.read_type();
    assert_eq!(input.shape_type(), -49);
    // The end of the stream reads as the type 255.
    assert_eq!(input.read_type(), ObjectType(255));
    assert!(!input.is_ok());
}

#[test]
fn a_reference_points_back_from_its_type_byte() {
    let mut stream = IoStream::from_bytes(vec![9, 9, 9, 1, 3]);
    let mut input = BinIStream::new(&mut stream);
    input.go_to(3);
    assert!(input.read_type().is_reference());
    assert_eq!(input.read_reference().unwrap(), 0);
    assert_eq!(input.position(), 5);
}

#[test]
fn a_zero_delta_is_an_invalid_reference() {
    let mut stream = IoStream::from_bytes(vec![1, 0]);
    let mut input = BinIStream::new(&mut stream);
    input.read_type();
    let error = input.read_reference().unwrap_err();
    assert_eq!(
        error.message(),
        "BinTools_IStream::ReadReference: invalid reference \u{1}\n"
    );
}

#[test]
fn transformations_are_the_rows_of_the_vectorial_part_with_the_translation() {
    let transform = Transform3::translation([1.0, 2.0, 3.0]);
    let bytes = written(|stream| BinOStream::new(stream).write_transform(&transform));
    let grouped = written(|stream| {
        bintools::put_transform(stream, &transform);
    });
    assert_eq!(bytes, grouped);
    assert_eq!(&bytes[24..32], &1.0_f64.to_ne_bytes());
    let mut stream = IoStream::from_bytes(bytes);
    let read = BinIStream::new(&mut stream).read_transform().unwrap();
    assert_eq!(read.translation_part().y(), 2.0);
    let singular = vec![0_u8; 96];
    let mut stream = IoStream::from_bytes(singular);
    let error = BinIStream::new(&mut stream).read_transform().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn unchecked_reads_past_the_end_read_zero_and_still_advance() {
    let mut stream = IoStream::from_bytes(vec![7]);
    let mut input = BinIStream::new(&mut stream);
    assert_eq!(input.read_byte(), 7);
    assert_eq!(input.read_byte(), 0);
    assert_eq!(input.read_short_real(), 0.0);
    assert_eq!(input.position(), 6);
    assert_eq!(
        input.read_pnt().unwrap_err().exception_type(),
        "Storage_StreamTypeMismatchError"
    );
}
