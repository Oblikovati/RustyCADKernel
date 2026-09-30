//! Search, SearchFromEnd, Token, SubString and the set locators.
use super::{
    ExtendedString,
    errors::{int_length, offset, out_of_range},
};
use crate::standard::Failure;
impl ExtendedString {
    /// One-based start of the first occurrence, or -1 for an empty or absent needle;
    /// `ExtendedString::new("hello").search(&[108]) == 3`.
    pub fn search(&self, what: impl AsRef<[u16]>) -> i32 {
        let what = what.as_ref();
        if what.is_empty() || what.len() > self.len() {
            return -1;
        }
        self.as_utf16()
            .windows(what.len())
            .position(|window| window == what)
            .map_or(-1, |start| int_length(start + 1))
    }
    /// One-based start of the last occurrence, or -1; `ExtendedString::new("hello").search_from_end(&[108]) == 4`.
    pub fn search_from_end(&self, what: impl AsRef<[u16]>) -> i32 {
        let what = what.as_ref();
        if what.is_empty() || what.len() > self.len() {
            return -1;
        }
        self.as_utf16()
            .windows(what.len())
            .rposition(|window| window == what)
            .map_or(-1, |start| int_length(start + 1))
    }
    /// Copies the inclusive one-based range `from..=to`, zero units included;
    /// `ExtendedString::new("abcdefg").sub_string(3, 6)?`.
    pub fn sub_string(&self, from: i32, to: i32) -> Result<Self, Failure> {
        const CONTEXT: &str = "TCollection_ExtendedString::SubString : ";
        if from < 1 || from > self.length() {
            return Err(self.index_error(CONTEXT, "theFromIndex", from));
        }
        if to < 1 || to > self.length() {
            return Err(self.index_error(CONTEXT, "theToIndex", to));
        }
        if from > to {
            return Err(out_of_range(
                "TCollection_ExtendedString::SubString : theFromIndex > theToIndex",
                format_args!(
                    "theFromIndex {from}, theToIndex {to}; expected theFromIndex <= theToIndex"
                ),
            ));
        }
        Ok(Self::from_raw(
            &self.as_utf16()[offset(from)..offset(to) + 1],
        ))
    }
    /// Extracts the `which_one`-th token; separators are read up to their first zero unit.
    /// An empty separator set, a `which_one` < 1 or a missing token give an empty string;
    /// a token ends at a zero unit. `ExtendedString::new("a b").token(&[32], 2)`.
    pub fn token(&self, separators: &[u16], which_one: i32) -> Self {
        let units = self.as_utf16();
        let end = separators.iter().position(|&unit| unit == 0);
        let separators = &separators[..end.unwrap_or(separators.len())];
        let mut buffer = vec![0_u16; units.len() + 1];
        let (mut found, mut next) = (0, 0);
        while found < which_one && next < units.len() {
            buffer[0] = 0;
            next = skip_separators(units, separators, next);
            next = copy_token(units, separators, next, &mut buffer);
            found += 1;
        }
        if found < which_one {
            return Self::default();
        }
        Self::from_utf16(&buffer)
    }
    /// First one-based index in `from..=to` whose unit is in `set`, or 0; both bounds must
    /// lie in 1..=len (a reversed range finds nothing);
    /// `text.first_location_in_set(&set, 1, text.length())?`.
    pub fn first_location_in_set(&self, set: &Self, from: i32, to: i32) -> Result<i32, Failure> {
        self.check_location_bounds("FirstLocationInSet", from, to)?;
        if set.is_empty() {
            return Ok(0);
        }
        Ok(self.locate(from, to, |unit| set.as_utf16().contains(&unit)))
    }
    /// First one-based index in `from..=to` whose unit is not in `set`, or 0; with an empty
    /// set that is `from` when `from <= to`; `text.first_location_not_in_set(&set, 1, text.length())?`.
    pub fn first_location_not_in_set(
        &self,
        set: &Self,
        from: i32,
        to: i32,
    ) -> Result<i32, Failure> {
        self.check_location_bounds("FirstLocationNotInSet", from, to)?;
        if set.is_empty() {
            return Ok(if from <= to { from } else { 0 });
        }
        Ok(self.locate(from, to, |unit| !set.as_utf16().contains(&unit)))
    }
    fn locate(&self, from: i32, to: i32, matches: impl Fn(u16) -> bool) -> i32 {
        (from..=to)
            .find(|&index| matches(self.as_utf16()[offset(index)]))
            .unwrap_or(0)
    }
    fn check_location_bounds(&self, member: &str, from: i32, to: i32) -> Result<(), Failure> {
        let context = format!("TCollection_ExtendedString::{member} : ");
        if from < 1 || from > self.length() {
            return Err(self.index_error(&context, "theFromIndex", from));
        }
        if to < 1 || to > self.length() {
            return Err(self.index_error(&context, "theToIndex", to));
        }
        Ok(())
    }
    fn index_error(&self, context: &str, name: &str, index: i32) -> Failure {
        out_of_range(
            &format!("{context}{name}"),
            format_args!("{name} {index}; expected 1..={}", self.len()),
        )
    }
}
/// Token's first phase: skips the separators in front of the token.
fn skip_separators(units: &[u16], separators: &[u16], mut next: usize) -> usize {
    while next < units.len() && separators.contains(&units[next]) {
        next += 1;
    }
    next
}
/// Token's second phase, literally: with no separators nothing is written, so the buffer
/// keeps the zero OCCT stored in its first unit and the token comes out empty.
fn copy_token(units: &[u16], separators: &[u16], mut next: usize, buffer: &mut [u16]) -> usize {
    let (mut written, mut at_separator) = (0, false);
    while !at_separator && next < units.len() {
        if !separators.is_empty() {
            at_separator = separators.contains(&units[next]);
            buffer[written] = if at_separator { 0 } else { units[next] };
        }
        next += 1;
        written += 1;
        if next == units.len() {
            buffer[written] = 0;
        }
    }
    next
}
