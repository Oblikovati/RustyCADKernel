//! The `TCollection_AsciiString` members `OSD_Path.cxx` calls, over byte vectors, with the
//! native 1-based positions and `Standard_OutOfRange` results.
//!
//! `OSD_Path` reaches these out-of-range branches with ordinary inputs (a VMS username
//! without its closing quote, an empty relative path), so the path code needs them as
//! `Result`s; the crate's `AsciiString` asserts instead. Only the members OSD_Path uses
//! are here, each mirroring its TCollection_AsciiString.cxx body.
use crate::standard::Failure;

pub(super) type Text = Vec<u8>;

pub(super) fn out_of_range(message: &str) -> Failure {
    Failure::with_type("Standard_OutOfRange", message, "")
}

/// `Value(where)`.
pub(super) fn value(text: &[u8], at: i32) -> Result<u8, Failure> {
    usize::try_from(at)
        .ok()
        .filter(|&at| at >= 1 && at <= text.len())
        .map(|at| text[at - 1])
        .ok_or_else(|| out_of_range("TCollection_AsciiString::Value : parameter theWhere"))
}

/// `Value(Length())`: the last byte, raising for an empty string.
pub(super) fn last(text: &[u8]) -> Result<u8, Failure> {
    value(text, len(text))
}

pub(super) fn len(text: &[u8]) -> i32 {
    text.len() as i32
}

/// `Insert(where, string)`.
pub(super) fn insert(text: &mut Text, at: i32, what: &[u8]) -> Result<(), Failure> {
    if at < 1 || at > len(text) + 1 {
        return Err(out_of_range(
            "TCollection_AsciiString::Insert : parameter theWhere",
        ));
    }
    let offset: usize = at as usize - 1;
    text.splice(offset..offset, what.iter().copied());
    Ok(())
}

/// `Remove(where, howmany)`.
///
/// Native only checks `where + howmany <= Length() + 1`: a zero count at a non-positive
/// position (OSD_Path's `Remove(Search(""), 0)`) copies every byte onto itself, a no-op.
/// A nonzero count before position 1 writes before the buffer natively; OSD_Path never
/// reaches it, and it is refused here.
pub(super) fn remove(text: &mut Text, at: i32, count: i32) -> Result<(), Failure> {
    if at + count > len(text) + 1 || count < 0 || (at < 1 && count > 0) {
        return Err(out_of_range(
            "TCollection_AsciiString::Remove: Too many characters to erase or invalid starting value.",
        ));
    }
    if count > 0 {
        let offset: usize = at as usize - 1;
        text.drain(offset..offset + count as usize);
    }
    Ok(())
}

/// `Trunc(howmany)`.
pub(super) fn trunc(text: &mut Text, length: i32) -> Result<(), Failure> {
    if length < 0 || length > len(text) {
        return Err(out_of_range(
            "TCollection_AsciiString::Trunc : parameter 'theHowMany'",
        ));
    }
    text.truncate(length as usize);
    Ok(())
}

/// `SubString(from, to)`.
pub(super) fn sub_string(text: &[u8], from: i32, to: i32) -> Result<Text, Failure> {
    if from <= 0 || to <= 0 || to > len(text) || from > to {
        return Err(out_of_range(
            "TCollection_AsciiString::SubString() out of range",
        ));
    }
    Ok(text[from as usize - 1..to as usize].to_vec())
}

/// `Search(what)`: the 1-based first position, -1 when absent or `what` is empty.
pub(super) fn search(text: &[u8], what: &[u8]) -> i32 {
    if what.is_empty() {
        return -1;
    }
    text.windows(what.len())
        .position(|part| part == what)
        .map_or(-1, |index| index as i32 + 1)
}

/// `SearchFromEnd(what)`: the 1-based last position, -1 when absent or `what` is empty.
pub(super) fn search_from_end(text: &[u8], what: &[u8]) -> i32 {
    if what.is_empty() {
        return -1;
    }
    text.windows(what.len())
        .rposition(|part| part == what)
        .map_or(-1, |index| index as i32 + 1)
}

/// `Token(separators, which)`: the which-th run of non-separator bytes, empty when absent
/// or when `which < 1`.
pub(super) fn token(text: &[u8], separators: &[u8], which: i32) -> Text {
    let Ok(skip) = usize::try_from(which - 1) else {
        return Text::new();
    };
    text.split(|byte| separators.contains(byte))
        .filter(|part| !part.is_empty())
        .nth(skip)
        .map_or_else(Text::new, <[u8]>::to_vec)
}

/// `Location(n, char, from, to)`: the position of the n-th `byte` in `from..=to`, 0 when
/// absent.
pub(super) fn location(text: &[u8], n: i32, byte: u8, from: i32, to: i32) -> Result<i32, Failure> {
    if from <= 0 || to > len(text) || from > to {
        return Err(out_of_range(""));
    }
    let mut count: i32 = 0;
    for position in from..=to {
        if text[position as usize - 1] == byte {
            count += 1;
            if count == n {
                return Ok(position);
            }
        }
    }
    Ok(0)
}

/// `ChangeAll(from, to)`.
pub(super) fn change_all(text: &mut [u8], from: u8, to: u8) {
    text.iter_mut()
        .filter(|byte| **byte == from)
        .for_each(|byte| *byte = to);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remove_keeps_the_native_zero_count_no_op_and_range_check() {
        let mut text: Text = b"abc".to_vec();
        remove(&mut text, -1, 0).unwrap();
        assert_eq!(text, b"abc");
        assert!(remove(&mut text, 2, 3).is_err());
        remove(&mut text, 2, 2).unwrap();
        assert_eq!(text, b"a");
    }
    #[test]
    fn token_counts_runs_between_separators_from_one() {
        assert_eq!(token(b"||a|bc|", b"|", 2), b"bc");
        assert!(token(b"a", b"|", 0).is_empty());
        assert!(token(b"a", b"|", 2).is_empty());
    }
    #[test]
    fn location_raises_for_an_empty_range() {
        assert_eq!(location(b"|a|b", 2, b'|', 1, 4).unwrap(), 3);
        assert_eq!(location(b"|a|b", 3, b'|', 1, 4).unwrap(), 0);
        assert!(location(b"", 1, b'|', 1, 0).is_err());
    }
}
