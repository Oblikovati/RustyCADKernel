//! The libstdc++ iostream model behind every storage driver.
use rustycad_tkernel::storage::{IoStream, format_general, state};
use std::io::{Read, Seek};

#[test]
fn integer_extraction_clamps_out_of_range_values_with_failbit() {
    let mut stream = IoStream::from_bytes(b"2147483648 -99999999999999999999 7".to_vec());
    let mut value = 0;
    assert!(!stream.extract_int(&mut value));
    assert_eq!(value, i32::MAX);
    stream.clear(0);
    assert!(!stream.extract_int(&mut value));
    assert_eq!(value, i32::MIN);
    stream.clear(0);
    assert!(stream.extract_int(&mut value));
    assert_eq!(value, 7);
    assert!(stream.eof(), "the digits ran to the end of the input");
}

#[test]
fn missing_digits_store_zero_and_leave_the_stream_before_the_word() {
    let mut stream = IoStream::from_bytes(b" x".to_vec());
    let mut value = 5;
    assert!(!stream.extract_int(&mut value));
    assert_eq!(value, 0);
    assert_eq!(stream.rdstate(), state::FAIL);
    stream.clear(0);
    let mut byte = 0;
    stream.get(&mut byte);
    assert_eq!(byte, b'x');
}

#[test]
fn unsigned_short_negates_modulo_and_short_clamps() {
    let mut stream = IoStream::from_bytes(b"-80 65536 40000".to_vec());
    let mut unsigned = 0_u16;
    assert!(stream.extract_unsigned_short(&mut unsigned));
    assert_eq!(unsigned, 65456);
    assert!(!stream.extract_unsigned_short(&mut unsigned));
    assert_eq!(unsigned, u16::MAX);
    stream.clear(0);
    let mut short = 0_i16;
    assert!(!stream.extract_short(&mut short));
    assert_eq!(short, i16::MAX);
}

#[test]
fn boolean_extraction_accepts_only_zero_and_one() {
    let mut stream = IoStream::from_bytes(b"1 0 2".to_vec());
    let mut flag = false;
    assert!(stream.extract_bool(&mut flag) && flag);
    assert!(stream.extract_bool(&mut flag) && !flag);
    assert!(!stream.extract_bool(&mut flag));
    assert!(flag, "any other value reads as true with failbit");
}

#[test]
fn getline_fails_on_a_full_buffer_and_consumes_the_delimiter() {
    let mut stream = IoStream::from_bytes(b"abcdef\nxy\n".to_vec());
    let mut line = Vec::new();
    stream.getline(&mut line, 4, b'\n');
    assert_eq!(line, b"abc");
    assert!(stream.fail());
    stream.clear(0);
    stream.getline(&mut line, 100, b'\n');
    assert_eq!((line.as_slice(), stream.gcount()), (b"def".as_slice(), 4));
    stream.getline(&mut line, 100, b'\n');
    assert_eq!(line, b"xy");
    stream.getline(&mut line, 100, b'\n');
    assert_eq!(stream.rdstate(), state::EOF | state::FAIL);
}

#[test]
fn a_word_filling_the_known_buffer_to_the_end_of_input_sets_eof() {
    let mut stream = IoStream::from_bytes(vec![b'9'; 99]);
    assert_eq!(stream.extract_word(100).len(), 99);
    assert!(stream.eof());
    let mut stream = IoStream::from_bytes(vec![b'9'; 120]);
    assert_eq!(stream.extract_word(100).len(), 99);
    assert!(stream.good());
    assert_eq!(stream.extract_word(100).len(), 21);
}

#[test]
fn width_bounds_a_word_and_is_reset_by_the_extraction() {
    let mut stream = IoStream::from_bytes(b"abcdef".to_vec());
    stream.set_width(3);
    assert_eq!(stream.extract_word(100), b"ab");
    assert_eq!(stream.width(), 0);
}

#[test]
fn tellg_fails_at_end_of_file_while_tellp_still_reports() {
    let mut stream = IoStream::from_bytes(b"12".to_vec());
    let mut value = 0;
    stream.extract_int(&mut value);
    assert!(stream.eof());
    assert_eq!(stream.tellp(), 2);
    assert_eq!(stream.tellg(), -1);
    assert!(stream.fail(), "the input sentry adds failbit");
}

#[test]
fn a_closed_stream_reads_end_of_file_and_refuses_writes() {
    let mut stream = IoStream::default();
    stream.insert_integer(5);
    assert!(stream.bad());
    let mut closed = IoStream::default();
    let mut byte = 7;
    closed.get(&mut byte);
    assert_eq!((byte, closed.rdstate()), (7, state::EOF | state::FAIL));
    assert_eq!(IoStream::default().tellp(), -1);
    closed.close();
    assert!(closed.fail(), "closing a closed stream fails");
}

#[test]
fn read_keeps_a_short_prefix_and_counts_it() {
    let mut stream = IoStream::from_bytes(b"abc".to_vec());
    let mut bytes = [0_u8; 5];
    stream.read(&mut bytes);
    assert_eq!((&bytes[..3], stream.gcount()), (b"abc".as_slice(), 3));
    assert_eq!(stream.rdstate(), state::EOF | state::FAIL);
}

#[test]
fn writes_land_after_unread_look_ahead_at_the_logical_position() {
    let mut stream = IoStream::from_bytes(b"12 34".to_vec());
    let mut value = 0;
    stream.extract_int(&mut value);
    stream.write(b"__");
    assert_eq!(stream.tellp(), 4);
}

#[test]
fn general_format_matches_printf_g() {
    assert_eq!(format_general(0.1, 17), "0.10000000000000001");
    assert_eq!(format_general(1e22, 17), "1e+22");
    assert_eq!(format_general(123456.0, 6), "123456");
    assert_eq!(format_general(1234567.0, 6), "1.23457e+06");
    assert_eq!(format_general(1e-5, 6), "1e-05");
    assert_eq!(format_general(0.0001, 6), "0.0001");
    assert_eq!(format_general(-0.0, 17), "-0");
    assert_eq!(format_general(f64::NAN, 17), "nan");
    assert_eq!(format_general(-f64::INFINITY, 17), "-inf");
    assert_eq!(
        format_general(8796093022208.0 + 0.03125, 17),
        "8796093022208.0312"
    );
    assert_eq!(format_general(2.5, 0), "2");
}

#[test]
fn formatted_insertion_pads_to_the_pending_width_once() {
    // `OS << std::setw(4) << 7 << 8 << std::setw(2) << "abc"`: the width right-aligns
    // the next insertion only, and a longer text is never cut.
    let mut stream = IoStream::from_bytes(Vec::new());
    stream.set_width(4);
    stream.insert_integer(7).insert_integer(8);
    assert_eq!(stream.width(), 0);
    stream.set_width(2);
    stream.insert_bytes(b"abc");
    stream.set_width(6);
    stream.insert_real(-0.5);
    let mut device = stream.into_device().unwrap();
    let mut text = Vec::new();
    device.seek(std::io::SeekFrom::Start(0)).unwrap();
    device.read_to_end(&mut text).unwrap();
    assert_eq!(text, b"   78abc  -0.5");
}

#[test]
fn a_failed_insertion_keeps_the_pending_width() {
    let mut stream = IoStream::default();
    stream.set_state(state::BAD);
    stream.set_width(3);
    stream.insert_integer(1);
    assert_eq!(stream.width(), 3, "the sentry failed before width(0)");
}

#[test]
fn seekg_clears_eof_and_moves_the_input_position() {
    let mut stream = IoStream::from_bytes(b"abc".to_vec());
    let mut bytes = [0_u8; 4];
    stream.read(&mut bytes);
    assert_eq!(stream.rdstate(), state::EOF | state::FAIL);
    stream.seekg(1);
    assert_eq!(
        stream.rdstate(),
        state::FAIL,
        "failbit stays: the sentry refuses"
    );
    stream.clear(state::EOF);
    stream.seekg(1);
    assert!(stream.good(), "eofbit alone is cleared before the sentry");
    let mut byte = 0;
    stream.get(&mut byte);
    assert_eq!(byte, b'b');
    assert_eq!(stream.tellg(), 2);
}

#[test]
fn seekg_refuses_a_negative_position_with_failbit() {
    let mut stream = IoStream::from_bytes(b"abc".to_vec());
    stream.seekg(-3);
    assert_eq!(stream.rdstate(), state::FAIL);
    let mut closed = IoStream::default();
    closed.seekg(0);
    assert!(closed.fail(), "a closed buffer refuses every position");
}

#[test]
fn char_extraction_skips_spaces_and_fails_at_the_end() {
    let mut stream = IoStream::from_bytes(b" \n\x01 ".to_vec());
    let mut byte = 9;
    assert!(stream.extract_char(&mut byte));
    assert_eq!(byte, 1);
    assert!(!stream.extract_char(&mut byte));
    assert_eq!(byte, 1, "the value is kept when nothing is extracted");
    assert_eq!(stream.rdstate(), state::EOF | state::FAIL);
}
