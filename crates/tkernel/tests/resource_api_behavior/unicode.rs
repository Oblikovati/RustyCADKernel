//! Resource_Unicode conversions, the current format and Resource_FormatType.
use super::support::FakeResourceSystem;
use rustycad_tkernel::{
    resource::{FormatType, ResourceManager, ResourceUnicode},
    tcollection::ExtendedString,
};
use std::{path::PathBuf, sync::Arc};

/// 日 (U+65E5) and 本 (U+672C) in each legacy encoding.
const NIHON: [u16; 2] = [0x65E5, 0x672C];

fn decoded(format: FormatType, bytes: &[u8]) -> Vec<u16> {
    let mut text = ExtendedString::default();
    ResourceUnicode::convert_format_to_unicode(format, bytes, &mut text).unwrap();
    text.as_utf16().to_vec()
}
fn encoded(format: FormatType, units: &[u16], size: usize) -> (bool, Vec<u8>) {
    let mut buffer = vec![0xA5; size];
    let done = ResourceUnicode::convert_unicode_to_format(
        format,
        &ExtendedString::from_raw(units),
        &mut buffer,
    )
    .unwrap();
    (done, buffer)
}

#[test]
fn table_encodings_round_trip_through_the_exact_tables() {
    for (format, bytes) in [
        (FormatType::Sjis, &b"\x93\xfa\x96{"[..]),
        (FormatType::Euc, b"\xc6\xfc\xcb\xdc"),
        (FormatType::Gb, b"\xc8\xd5\xb1\xbe"),
    ] {
        assert_eq!(decoded(format, bytes), NIHON, "{format:?}");
        let (done, buffer) = encoded(format, &NIHON, 8);
        assert!(done);
        assert_eq!(&buffer[..5], [bytes, b"\0"].concat().as_slice());
    }
    let mut text = ExtendedString::default();
    ResourceUnicode::convert_sjis_to_unicode(b"A\x93\xfa", &mut text);
    assert_eq!(text.as_utf16(), &[0x41, 0x65E5]);
    ResourceUnicode::convert_euc_to_unicode(b"\xc6\xfc", &mut text);
    ResourceUnicode::convert_gb_to_unicode(b"\xc8\xd5", &mut text);
    assert_eq!(text.as_utf16(), &[0x65E5]);
}
#[test]
fn gbk_and_big5_decoders_report_invalid_input() {
    let mut text = ExtendedString::default();
    assert!(ResourceUnicode::convert_gbk_to_unicode(b"\xc8\xd5\x80", &mut text).unwrap());
    assert_eq!(text.as_utf16(), &[0x65E5, 0x20AC]);
    assert!(!ResourceUnicode::convert_gbk_to_unicode(b"A\xff", &mut text).unwrap());
    let raised = ResourceUnicode::convert_gbk_to_unicode(b"x\x81 ", &mut text).unwrap_err();
    assert_eq!(
        (raised.exception_type(), text.as_utf16()),
        ("Standard_OutOfRange", &[0x78][..])
    );
    assert!(ResourceUnicode::convert_big5_to_unicode(b"\xa4\xe9", &mut text).unwrap());
    assert_eq!(text.as_utf16(), &[0x65E5]);
    assert!(!ResourceUnicode::convert_big5_to_unicode(b"\xa4\xff", &mut text).unwrap());
    assert!(ResourceUnicode::convert_big5_to_unicode(b"\xa4 ", &mut text).is_err());
}
#[test]
fn encoders_truncate_like_native_buffers() {
    assert_eq!(
        encoded(FormatType::Sjis, &NIHON, 4),
        (false, vec![0x93, 0xfa, 0, 0xA5])
    );
    assert_eq!(
        encoded(FormatType::Euc, &[0x41, 0x65E5], 4),
        (false, vec![0, 0xA5, 0xA5, 0xA5])
    );
    // ANSI reports false once the text fills maxsize - 1 bytes, even when it is complete.
    assert_eq!(
        encoded(FormatType::ANSI, &[0x41, 0x100], 3),
        (false, vec![0x41, b' ', 0])
    );
    assert_eq!(
        encoded(FormatType::ANSI, &[0x41], 3),
        (true, vec![0x41, 0, 0xA5])
    );
    let mut empty: [u8; 0] = [];
    let error = ResourceUnicode::convert_unicode_to_ansi(&ExtendedString::default(), &mut empty);
    assert_eq!(error.unwrap_err().exception_type(), "Standard_OutOfRange");
    let mut buffer = [0u8; 4];
    let text = ExtendedString::from_raw(&NIHON);
    assert!(!ResourceUnicode::convert_unicode_to_sjis(&text, &mut buffer[..3]).unwrap());
    assert!(ResourceUnicode::convert_unicode_to_euc(&text, &mut [0u8; 8]).unwrap());
    assert!(ResourceUnicode::convert_unicode_to_gb(&text, &mut [0u8; 8]).unwrap());
}
#[test]
fn code_pages_decode_every_byte_and_encode_only_upper_slots() {
    assert_eq!(
        decoded(FormatType::Cp1252, b"a\x80\x81"),
        [0x61, 0x20AC, 0x81]
    );
    assert_eq!(decoded(FormatType::Iso8859_5, b"\xb0"), [0x410]);
    // ASCII is absent from the 128 code-page slots, so it encodes as '?'.
    assert_eq!(
        encoded(FormatType::Cp1252, &[0x41, 0x20AC], 3),
        (true, vec![b'?', 0x80, 0])
    );
    assert_eq!(
        encoded(FormatType::Cp1252, &[0x41, 0x20AC], 1),
        (false, vec![0xA5])
    );
    let mut buffer = [0u8; 4];
    let text = ExtendedString::from_raw(&[0x20AC]);
    let error = ResourceUnicode::convert_unicode_to_format(FormatType::Cp1251, &text, &mut buffer);
    assert_eq!(error.unwrap_err().exception_type(), "Standard_OutOfRange");
}
#[test]
fn utf8_locale_and_unconverted_formats() {
    assert_eq!(
        decoded(FormatType::Utf8, "é😀".as_bytes()),
        [0xE9, 0xD83D, 0xDE00]
    );
    assert_eq!(
        decoded(FormatType::Utf8, b"a\xf8\x88\x80\x80\x80"),
        [0x61, 0xF8, 0x88, 0x80, 0x80, 0x80]
    );
    assert_eq!(decoded(FormatType::NoConversion, b"\xe9"), [0xE9]);
    assert_eq!(decoded(FormatType::SystemLocale, b"ab"), [0x61]);
    assert_eq!(
        encoded(FormatType::Utf8, &[0xE9], 3),
        (true, vec![0xC3, 0xA9, 0])
    );
    assert_eq!(encoded(FormatType::Utf8, &[0xE9], 1), (false, vec![0xA5]));
    assert_eq!(
        encoded(FormatType::SystemLocale, &[0x41], 2),
        (true, vec![0x41, 0])
    );
    let mut buffer = [0u8; 4];
    let text = ExtendedString::from_raw(&[0x41]);
    for format in [FormatType::Gbk, FormatType::Big5] {
        let error = ResourceUnicode::convert_unicode_to_format(format, &text, &mut buffer);
        assert_eq!(
            error.unwrap_err().exception_type(),
            "Standard_NotImplemented"
        );
    }
}
#[test]
fn current_format_is_process_wide_and_read_from_charset() {
    let _serial = super::serial();
    let system: Arc<FakeResourceSystem> = Arc::default();
    system.write_resource(&PathBuf::from("charset"), "CharSet", "FormatType", "EUC");
    system.set_environment("CSF_CharSetDefaults", PathBuf::from("charset"));
    ResourceUnicode::read_format_with(&system.host()).unwrap();
    assert_eq!(
        ResourceUnicode::get_format_with(&system.host()).unwrap(),
        FormatType::Euc
    );
    ResourceUnicode::set_format(FormatType::Utf8);
    assert_eq!(ResourceUnicode::get_format().unwrap(), FormatType::Utf8);
    let mut manager = ResourceManager::empty(system.host());
    manager.set_resource("title", "é".as_bytes());
    assert_eq!(manager.ext_value("title").unwrap().as_utf16(), &[0xE9]);
    // The cache is not refreshed by a later byte assignment.
    manager.set_resource("title", b"x");
    assert_eq!(manager.ext_value("title").unwrap().as_utf16(), &[0xE9]);
    manager.set_resource_ext("ext", &[0x20AC, 0, 0x41]).unwrap();
    assert_eq!(manager.value("ext").unwrap().as_bytes(), "€".as_bytes());
    let mut text = ExtendedString::default();
    ResourceUnicode::convert_current_to_unicode("€".as_bytes(), &mut text).unwrap();
    assert_eq!(text.as_utf16(), &[0x20AC]);
    let mut buffer = [0u8; 4];
    assert!(ResourceUnicode::convert_unicode_to_current(&text, &mut buffer).unwrap());
    assert_eq!(buffer, [0xE2, 0x82, 0xAC, 0]);
    ResourceUnicode::set_format(FormatType::Cp1252);
    assert!(manager.set_resource_ext("cp", &[0x20AC]).is_err());
    assert_eq!(manager.ext_value("cp").unwrap().as_utf16(), &[0x20AC]);
    ResourceUnicode::set_format(FormatType::ANSI);
}
#[test]
fn format_type_keeps_native_discriminants_and_aliases() {
    assert_eq!(FormatType::ANSI, FormatType::NoConversion);
    assert_eq!(FormatType::from_raw(26), Some(FormatType::Big5));
    assert_eq!(FormatType::from_raw(27), None);
    assert_eq!(FormatType::from_raw(-1), None);
    for (index, format) in FormatType::ALL.iter().enumerate() {
        assert_eq!(*format as usize, index);
    }
}
