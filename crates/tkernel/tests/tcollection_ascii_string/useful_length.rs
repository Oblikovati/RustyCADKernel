#[test]
fn TCollection_AsciiStringTest__UsefullLength_EmptyString() {
    let string = AsciiString::new("");
    assert_eq!(string.useful_length(), 0);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_AsciiNoTrailing() {
    let string = AsciiString::new("Hello");
    assert_eq!(string.useful_length(), 5);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_TrailingSpaces() {
    let string = AsciiString::new("Hello   ");
    assert_eq!(string.useful_length(), 5);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_AllSpaces() {
    let string = AsciiString::new("     ");
    assert_eq!(string.useful_length(), 0);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_AllControlChars() {
    let string = AsciiString::new("\t\n\r");
    assert_eq!(string.useful_length(), 0);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_SingleChar() {
    let string = AsciiString::new("A");
    assert_eq!(string.useful_length(), 1);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_SingleSpace() {
    let string = AsciiString::new(" ");
    assert_eq!(string.useful_length(), 0);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_MixedTrailing() {
    let string = AsciiString::new("Part Name  \t\n");
    assert_eq!(string.useful_length(), 9);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_Utf8AtEnd() {
    let string = AsciiString::new("Hello世界");
    assert_eq!(string.useful_length(), 11);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_Utf8ThenSpaces() {
    let string = AsciiString::new("Привет   ");
    assert_eq!(string.useful_length(), 12);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_Utf8Only() {
    let string = AsciiString::new("日本");
    assert_eq!(string.useful_length(), 6);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_Utf8FourByteAtEnd() {
    let string = AsciiString::new("Test😀");
    assert_eq!(string.useful_length(), 8);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_TrailingControlChars() {
    let mut string = AsciiString::new("Hello");
    string += b'\t';
    string += b'\n';
    string += b'\r';
    assert_eq!(string.useful_length(), 5);
}
#[test]
fn TCollection_AsciiStringTest__UsefullLength_Utf8ThenControlChars() {
    let mut string = AsciiString::new("Ä");
    string += b'\t';
    string += b'\n';
    assert_eq!(string.useful_length(), 2);
}
