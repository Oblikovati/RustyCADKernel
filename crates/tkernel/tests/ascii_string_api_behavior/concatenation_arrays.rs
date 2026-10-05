#[test]
fn array_concatenation_stops_at_nul_but_spans_keep_their_length() {
    let literal = *b"x\0y\0";
    assert_eq!(bytes(&AsciiString::from(&literal)), b"x");
    let mut assigned = AsciiString::new("head");
    assigned.assign_cat(&literal);
    assert_eq!(bytes(&assigned), b"headx");

    let mut added = AsciiString::new("head");
    added += &literal;
    assert_eq!(bytes(&added), b"headx");

    let base = AsciiString::new("head");
    assert_eq!(bytes(&base.cat(&literal)), b"headx");
    assert_eq!(bytes(&(&base + &literal)), b"headx");

    let span: &[u8] = b"x\0y";
    let mut explicit = base.clone();
    explicit.assign_cat(span);
    assert_eq!(bytes(&explicit), b"headx\0y");
    assert_eq!(bytes(&base.cat(span)), b"headx\0y");
}
