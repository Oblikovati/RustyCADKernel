use rustycad_tkernel::ncollection::{UtfBufferError, UtfIterator};

#[test]
fn utf8_iterator_decodes_symbols_and_positions() {
    let text = "a\u{e9}\u{20ac}\u{1f600}".as_bytes();
    let mut iter = UtfIterator::new(text);
    let mut seen = Vec::new();
    while iter.current() != 0 {
        seen.push((
            iter.current(),
            iter.index(),
            iter.buffer_here(),
            iter.buffer_next(),
        ));
        iter.advance();
    }
    assert_eq!(
        seen,
        [
            (0x61, 0, 0, 1),
            (0xE9, 1, 1, 3),
            (0x20AC, 2, 3, 6),
            (0x1F600, 3, 6, 10)
        ]
    );
    assert_eq!((iter.index(), iter.buffer_here()), (4, 10));
}

#[test]
fn truncated_utf8_reads_zeros_past_the_end() {
    // 0xC3 announces one trailing byte; the missing byte reads as the terminator's zero.
    let iter = UtfIterator::new(&[0xC3u8][..]);
    assert_eq!((iter.current(), iter.buffer_next()), (0x40, 2));
    // Overlong NUL decodes to zero and therefore ends the text.
    assert_eq!(UtfIterator::new(&[0xC0u8, 0x80][..]).current(), 0);
}

#[test]
fn utf16_joins_only_ordered_surrogate_pairs() {
    let units = [0xD83Du16, 0xDE00, 0xDC00, 0xD800, 0x41];
    let mut iter = UtfIterator::new(&units[..]);
    assert_eq!((iter.current(), iter.buffer_next()), (0x1F600, 2));
    assert_eq!(iter.advance().current(), 0xDC00);
    assert_eq!(iter.advance().current(), 0xD800);
    assert_eq!(iter.advance().current(), 0x41);
}

#[test]
fn advances_follow_the_target_encoding() {
    let symbols = [
        0x7Fu32, 0x80, 0x7FF, 0x800, 0xD800, 0xFFFF, 0x10000, 0x10FFFF, 0x110000,
    ];
    let expected8 = [1, 2, 2, 3, 0, 3, 4, 4, 0];
    let expected16 = [1, 1, 1, 1, 0, 1, 2, 2, 0];
    for (index, &symbol) in symbols.iter().enumerate() {
        let iter = UtfIterator::new(&symbols[index..=index]);
        assert_eq!(iter.current(), symbol);
        assert_eq!(iter.advance_bytes_utf8(), expected8[index]);
        assert_eq!(iter.advance_code_units_utf16(), expected16[index]);
        assert_eq!(iter.advance_bytes_utf16(), 2 * expected16[index]);
        assert_eq!(iter.advance_bytes_utf32(), 4);
        assert_eq!(iter.advance_bytes_utf::<u8>(), expected8[index]);
        assert_eq!(iter.is_valid(), symbol <= 0x10FFFF);
    }
}

#[test]
fn getters_encode_and_report_short_buffers() {
    let units = [0x1F600u32];
    let iter = UtfIterator::new(&units[..]);
    let mut utf8 = [0u8; 4];
    assert_eq!(iter.get_utf8(&mut utf8), Ok(4));
    assert_eq!(utf8, [0xF0, 0x9F, 0x98, 0x80]);
    let mut utf32 = [0u32; 1];
    assert_eq!(iter.get_utf32(&mut utf32), Ok(1));
    let mut short = [0u16; 1];
    let error = iter.get_utf16(&mut short).unwrap_err();
    assert_eq!(
        error,
        UtfBufferError {
            needed: 2,
            available: 1
        }
    );
    assert!(error.to_string().contains("expected at least 2"));
}

#[test]
fn illegal_symbols_write_nothing() {
    let units = [0xDC00u32];
    let iter = UtfIterator::new(&units[..]);
    assert_eq!(iter.get_utf8(&mut []), Ok(0));
    assert_eq!(iter.get_utf::<u16>(&mut []), Ok(0));
}

#[test]
fn init_post_increment_and_equality() {
    let text = b"xy";
    let mut iter = UtfIterator::new(&text[..]);
    let before = iter.post_increment();
    assert_eq!(
        (before.current(), iter.current(), iter.index()),
        (0x78, 0x79, 1)
    );
    assert!(before != iter);
    let mut again = UtfIterator::new(&text[..]);
    again.advance();
    assert!(again == iter);
    iter.init(b"z");
    assert_eq!(
        (iter.current(), iter.index(), iter.buffer_here()),
        (0x7A, 0, 0)
    );
}

#[test]
fn empty_text_starts_on_the_terminator() {
    let iter = UtfIterator::<u16>::new(&[]);
    assert_eq!(
        (iter.current(), iter.index(), iter.buffer_next()),
        (0, 0, 1)
    );
}
