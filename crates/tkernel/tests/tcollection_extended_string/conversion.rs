#[test]
fn TCollection_ExtendedStringTest__UTF8Conversion() {
    let string = ExtendedString::new("Hello World");
    let size = string.utf8_length();
    assert!(size > 0);
    let mut buffer = vec![0; size + 1];
    let actual = string.write_utf8(&mut buffer).unwrap();
    assert_eq!(actual, size);
    assert_eq!(buffer[actual], 0);
    assert_eq!(&buffer[..actual], b"Hello World");
}
#[test]
fn TCollection_ExtendedStringTest__UTF8ConversionUnicode() {
    let string = ExtendedString::from_utf16(&[0x48, 0xE9, 0x6C, 0x6C, 0x6F, 0]);
    let size = string.utf8_length();
    assert!(size > 5);
    let mut buffer = vec![0; size + 1];
    let actual = string.write_utf8(&mut buffer).unwrap();
    assert_eq!(size, actual);
    assert_eq!(buffer[actual], 0);
}
#[test]
fn TCollection_ExtendedStringTest__WideCharConstructor() {
    let wide: Vec<u32> = "Wide string test"
        .chars()
        .map(u32::from)
        .chain([0])
        .collect();
    let string = ExtendedString::from_wide(&wide);
    assert!(!string.is_empty());
    assert!(string.length() > 0);
}
#[test]
fn TCollection_ExtendedStringTest__NumericalConstructors() {
    text(&ExtendedString::from_integer(42), "42");
    assert!(contains(&ExtendedString::from_real(3.14), "3.14"));
}
#[test]
fn TCollection_ExtendedStringTest__FillerConstructor() {
    let string = ExtendedString::filled(5, u16::from(b'X')).unwrap();
    assert_eq!(string.len(), 5);
    text(&string, "XXXXX");
}
#[test]
fn TCollection_ExtendedStringTest__ExtendedCharConstructor() {
    let string = ExtendedString::from_char(0x20AC);
    assert_eq!(string.len(), 1);
    assert!(!string.is_ascii());
    assert_eq!(string.value(1).unwrap(), 0x20AC);
}
#[test]
fn TCollection_ExtendedStringTest__UnicodeCharacters() {
    let string = ExtendedString::from_utf16(&[0x41, 0xE9, 0x20AC, 0x4E2D, 0]);
    assert_eq!(string.len(), 4);
    for (i, value) in [0x41, 0xE9, 0x20AC, 0x4E2D].into_iter().enumerate() {
        assert_eq!(string.value(i as i32 + 1).unwrap(), value);
    }
    assert!(!string.is_ascii());
}
#[test]
fn TCollection_ExtendedStringTest__AsciiDetection() {
    assert!(ExtendedString::new("Simple ASCII").is_ascii());
    assert!(!ExtendedString::from_utf16(&[0x41, 0x20AC, 0]).is_ascii());
}
#[test]
fn TCollection_ExtendedStringTest__EmptyStringHandling() {
    let string = ExtendedString::default();
    assert_eq!(string.len(), 0);
    assert!(string.is_empty());
    assert_eq!(string.utf8_length(), 0);
    let mut buffer = [1];
    let length = string.write_utf8(&mut buffer).unwrap();
    assert_eq!(length, 0);
    assert_eq!(buffer[0], 0);
}
#[test]
fn TCollection_ExtendedStringTest__ConversionRoundTrip() {
    let original = "Test conversion with special chars: !@#$%";
    let ascii = AsciiString::new(original);
    let extended = ExtendedString::from_ascii(&ascii);
    let result = AsciiString::from_extended(&extended, None);
    assert_eq!(result.as_bytes(), original.as_bytes());
    assert_eq!(ascii.len(), extended.len());
    assert_eq!(ascii.len(), result.len());
}
#[test]
fn TCollection_ExtendedStringTest__LargeStrings() {
    let string = ExtendedString::filled(1000, u16::from(b'A')).unwrap();
    assert_eq!(string.len(), 1000);
    assert_eq!(string.value(1).unwrap(), 65);
    assert_eq!(string.value(1000).unwrap(), 65);
    assert!(string.is_ascii());
}
#[test]
fn TCollection_ExtendedStringTest__MemoryAllocation() {
    for length in 1..=16 {
        let string = ExtendedString::filled(length, u16::from(b'X')).unwrap();
        assert_eq!(string.length(), length);
        assert_eq!(string.value(1).unwrap(), 88);
        if length > 1 {
            assert_eq!(string.value(length).unwrap(), 88);
        }
    }
}
#[test]
fn TCollection_ExtendedStringTest__MultiByteCString() {
    let string = ExtendedString::from_utf8("Multi-byte test");
    assert!(string.length() > 0);
    assert!(!string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__BoundaryValues() {
    for (unit, ascii) in [(0x7F, true), (0xFF, true), (0x100, false), (0xFFFF, false)] {
        let string = ExtendedString::from_char(unit);
        assert_eq!(string.len(), 1);
        assert_eq!(string.is_ascii(), ascii);
    }
}
#[test]
fn TCollection_ExtendedStringTest__TestMem_LargeStringAllocation() {
    let string = ExtendedString::filled(1024 * 1024, u16::from(b'A')).unwrap();
    assert_eq!(string.len(), 1024 * 1024);
    assert!(!string.is_empty());
}
#[test]
fn TCollection_ExtendedStringTest__OCC3277_CatOperation() {
    let empty = ExtendedString::default();
    let input = ExtendedString::new("TestString");
    let result = empty.cat(&input);
    assert_eq!(input.len(), result.len());
    assert!(!result.is_empty());
    assert_eq!(input, result);
}
