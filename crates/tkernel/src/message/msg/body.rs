//! The native `Message_Msg` text algorithms over UTF-16 code units.
//!
//! Every character test goes through `ToCharacter`, the low byte of the unit, so U+0125
//! reads as `%` exactly as in OCCT. Index checks mirror `TCollection_ExtendedString`'s
//! `Remove`/`Insert`/`Split`/`Value` and raise the same `Standard_OutOfRange`.
use super::Msg;
use crate::standard::Failure;
use crate::tcollection::ExtendedString;

/// The `FormatType` of a specification, stored in the triplet sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FormatKind {
    Integer = 0,
    Real = 1,
    String = 2,
}

/// A parsed message: units after `%%` collapsing plus the format triplets.
pub(super) struct BodyText {
    pub(super) units: Vec<u16>,
    pub(super) formats: Vec<i32>,
}

/// `ToCharacter`: the low byte of a UTF-16 unit; the terminator reads as 0.
fn character(units: &[u16], index: usize) -> u8 {
    units.get(index).map_or(0, |&unit| (unit & 0x00ff) as u8)
}

fn is_flag(character: u8) -> bool {
    matches!(character, b'-' | b'+' | b' ' | b'#' | b'.') || character.is_ascii_digit()
}

fn kind_of(character: u8) -> Option<FormatKind> {
    match character {
        b'd' | b'i' | b'o' | b'u' | b'x' | b'X' => Some(FormatKind::Integer),
        b'f' | b'e' | b'E' | b'g' | b'G' => Some(FormatKind::Real),
        b's' => Some(FormatKind::String),
        _ => None,
    }
}

/// Outcome of scanning one `%` directive.
enum Scan {
    /// Stop scanning the whole text (the native `break`).
    Stop,
    /// No specification; continue after this index.
    Skip(usize),
    /// A specification ending at this index.
    Found(FormatKind, usize),
}

impl BodyText {
    /// `Message_Msg::Set`: collapses `%%` and records `(kind, start, length)` triplets.
    ///
    /// Native quirk kept: after `%%` collapses, scanning continues from the character that
    /// followed, so `%%d` both prints `%` and is recorded as an integer specification.
    pub(super) fn parse(text: &[u16]) -> Self {
        let mut parsed = Self {
            units: text.to_vec(),
            formats: Vec::new(),
        };
        let mut index = 0;
        while index < parsed.units.len() {
            if character(&parsed.units, index) == b'%' {
                match parsed.scan_directive(index) {
                    Scan::Stop => break,
                    Scan::Skip(end) => index = end,
                    Scan::Found(kind, end) => {
                        parsed.formats.extend([
                            kind as i32,
                            index as i32,
                            (end + 1 - index) as i32,
                        ]);
                        index = end;
                    }
                }
            }
            index += 1;
        }
        parsed
    }

    fn scan_directive(&mut self, start: usize) -> Scan {
        let mut index = start + 1;
        if character(&self.units, index) == b'%' {
            self.units.remove(index);
            if index >= self.units.len() {
                return Scan::Stop;
            }
        }
        while index < self.units.len() && is_flag(character(&self.units, index)) {
            index += 1;
        }
        if index >= self.units.len() {
            return Scan::Stop;
        }
        if matches!(character(&self.units, index), b'h' | b'l') {
            index += 1;
        }
        match kind_of(character(&self.units, index)) {
            Some(kind) => Scan::Found(kind, index),
            None => Scan::Skip(index),
        }
    }
}

fn out_of_range(operation: &str, index: i32, length: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!("TCollection_ExtendedString::{operation}: index {index}; text length {length}"),
        "",
    )
}

impl Msg {
    /// `getFormat`: pops the first triplet of `kind`; returns its 1-based start and C format.
    pub(super) fn take_format(
        &mut self,
        kind: FormatKind,
    ) -> Result<Option<(i32, Vec<u8>)>, Failure> {
        let Some(slot) = (0..self.formats.len())
            .step_by(3)
            .find(|&slot| self.formats[slot] == kind as i32)
        else {
            return Ok(None);
        };
        let (first, length) = (self.formats[slot + 1], self.formats[slot + 2]);
        let units = self.body.as_utf16();
        let mut format = vec![b' '; length.max(0) as usize];
        for (offset, byte) in format.iter_mut().enumerate() {
            let position = first + offset as i32 + 1;
            let unit = checked_value(units, position)?;
            if unit & 0xff00 == 0 {
                *byte = unit as u8;
            }
        }
        self.formats.drain(slot..slot + 3);
        Ok(Some((first + 1, format)))
    }

    /// `replaceText`: swaps a specification for its text and shifts later triplets.
    pub(super) fn replace_text(
        &mut self,
        first: i32,
        count: usize,
        text: &ExtendedString,
    ) -> Result<(), Failure> {
        let mut units = self.body.as_utf16().to_vec();
        remove_units(&mut units, first, count as i32)?;
        let at = first as usize - 1;
        units.splice(at..at, text.as_utf16().iter().copied());
        self.body = ExtendedString::from_raw(&units);
        let increment = text.len() as i32 - count as i32;
        if increment == 0 {
            return Ok(());
        }
        for slot in (0..self.formats.len()).step_by(3) {
            if self.formats[slot + 1] > first {
                self.formats[slot + 1] += increment;
            }
        }
        Ok(())
    }
}

/// `Value(position)` with the native 1-based bounds check.
fn checked_value(units: &[u16], position: i32) -> Result<u16, Failure> {
    if position < 1 || position as usize > units.len() {
        return Err(out_of_range("Value", position, units.len()));
    }
    Ok(units[position as usize - 1])
}

/// `Remove(where, howMany)` with the native bounds check.
fn remove_units(units: &mut Vec<u16>, first: i32, count: i32) -> Result<(), Failure> {
    let length = units.len() as i32;
    if first < 1 || first > length || count < 0 || first + count > length + 1 {
        return Err(out_of_range("Remove", first, units.len()));
    }
    units.drain(first as usize - 1..(first + count) as usize - 1);
    Ok(())
}

impl Msg {
    /// Fills every pending specification with `UNKNOWN` and returns the text (`Get()`,
    /// also the `const TCollection_ExtendedString&` conversion); `msg.get()?`.
    ///
    /// Native quirk kept: the pending triplets are not consumed, so a second call inserts
    /// `UNKNOWN` again at the stale positions. Positions past the end raise
    /// `Standard_OutOfRange` from `Split`/`Remove`, returned here as an error.
    pub fn get(&mut self) -> Result<&ExtendedString, Failure> {
        let mut units = self.body.as_utf16().to_vec();
        let outcome = self.fill_unknown(&mut units);
        // The native loop edits the body in place, so a failure keeps the partial edit.
        self.body = ExtendedString::from_raw(&units);
        outcome.map(|()| &self.body)
    }
    fn fill_unknown(&self, units: &mut Vec<u16>) -> Result<(), Failure> {
        const UNKNOWN: [u16; 7] = [85, 78, 75, 78, 79, 87, 78];
        let mut increment = 0;
        for slot in (0..self.formats.len().saturating_sub(2)).step_by(3) {
            let split = self.formats[slot + 1] + increment;
            if split < 0 || split >= units.len() as i32 {
                return Err(out_of_range("Split", split, units.len()));
            }
            let mut right = units.split_off(split as usize);
            remove_units(&mut right, 1, self.formats[slot + 2])?;
            units.extend_from_slice(&UNKNOWN);
            units.extend_from_slice(&right);
            increment += UNKNOWN.len() as i32 - self.formats[slot + 2];
        }
        Ok(())
    }
}
