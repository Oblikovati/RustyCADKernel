//! Case mapping, byte replacement, whitespace trimming and justification.
//!
//! Case and whitespace follow the C locale's `toupper`/`tolower`/`isspace`, which touch
//! ASCII bytes only; high-bit bytes are never changed or trimmed.
use super::AsciiString;
use super::raise::negative_width;
use crate::standard::{Failure, character};

impl AsciiString {
    /// `UpperCase`: maps ASCII lowercase bytes in place; `text.upper_case()`.
    pub fn upper_case(&mut self) {
        self.terminated.make_ascii_uppercase();
    }
    /// `LowerCase`: maps ASCII uppercase bytes in place; `text.lower_case()`.
    pub fn lower_case(&mut self) {
        self.terminated.make_ascii_lowercase();
    }
    /// `Capitalize`: uppercases the first byte and lowercases the rest; `text.capitalize()`.
    pub fn capitalize(&mut self) {
        self.lower_case();
        self.terminated[0].make_ascii_uppercase();
    }
    /// `ChangeAll`: replaces every `from` byte by `to`, optionally ignoring ASCII case;
    /// `text.change_all(b'a', b'o', true)`.
    pub fn change_all(&mut self, from: u8, to: u8, case_sensitive: bool) {
        let length = self.len();
        let wanted = fold(from, case_sensitive);
        for byte in &mut self.terminated[..length] {
            if fold(*byte, case_sensitive) == wanted {
                *byte = to;
            }
        }
    }
    /// `RemoveAll(char)`: erases every occurrence of `byte`; `text.remove_all(b'e')`.
    pub fn remove_all(&mut self, byte: u8) {
        self.remove_all_with_case(byte, true);
    }
    /// `RemoveAll(char, bool)`: erases every occurrence, optionally ignoring ASCII case;
    /// `text.remove_all_with_case(b'E', false)`.
    pub fn remove_all_with_case(&mut self, byte: u8, case_sensitive: bool) {
        let wanted = fold(byte, case_sensitive);
        let length = self.len();
        let mut kept: Vec<u8> = self.terminated[..length]
            .iter()
            .copied()
            .filter(|&value| fold(value, case_sensitive) != wanted)
            .collect();
        kept.push(0);
        self.terminated = kept;
    }
    /// `LeftAdjust`: removes leading C-locale whitespace; `text.left_adjust()`.
    pub fn left_adjust(&mut self) {
        let leading = self
            .as_bytes()
            .iter()
            .take_while(|&&byte| character::is_space(byte))
            .count();
        self.terminated.drain(..leading);
    }
    /// `RightAdjust`: removes trailing C-locale whitespace; `text.right_adjust()`.
    pub fn right_adjust(&mut self) {
        let trailing = self
            .as_bytes()
            .iter()
            .rev()
            .take_while(|&&byte| character::is_space(byte))
            .count();
        self.set_length(self.len() - trailing);
    }
    /// `LeftJustify`: pads on the right with `filler` up to `width`; `text.left_justify(8, b'.')?`.
    pub fn left_justify(&mut self, width: i32, filler: u8) -> Result<(), Failure> {
        if width < 0 {
            return Err(negative_width("LeftJustify", width));
        }
        let padding = (width as usize).saturating_sub(self.len());
        let length = self.len();
        self.terminated
            .splice(length..length, std::iter::repeat_n(filler, padding));
        Ok(())
    }
    /// `RightJustify`: pads on the left with `filler` up to `width`; `text.right_justify(8, b' ')?`.
    pub fn right_justify(&mut self, width: i32, filler: u8) -> Result<(), Failure> {
        if width < 0 {
            return Err(negative_width("RightJustify", width));
        }
        let padding = (width as usize).saturating_sub(self.len());
        self.terminated
            .splice(0..0, std::iter::repeat_n(filler, padding));
        Ok(())
    }
    /// `Center`: pads both sides up to `width`, the odd filler going left;
    /// `AsciiString::new("ab").center(5, b'*')` gives `"**ab*"`.
    pub fn center(&mut self, width: i32, filler: u8) -> Result<(), Failure> {
        if width < 0 {
            return Err(negative_width("Center", width));
        }
        if width > self.length() {
            let left_width = self.length() + (width - self.length()) / 2;
            self.left_justify(left_width, filler)?;
            self.right_justify(width, filler)?;
        }
        Ok(())
    }
}

/// `::UpperCase` when the comparison ignores case, the byte itself otherwise.
fn fold(byte: u8, case_sensitive: bool) -> u8 {
    if case_sensitive {
        byte
    } else {
        character::upper_case(byte)
    }
}
