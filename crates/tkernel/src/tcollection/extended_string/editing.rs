//! In-place edits: Insert, SetValue, Remove, Trunc, Split, Copy and ownership moves.
use super::{
    ExtendedString,
    errors::{int_length, leading_units, offset, out_of_range},
    utf,
};
use crate::standard::Failure;
impl ExtendedString {
    /// Inserts one code unit, zero included, at a one-based position in 1..=len+1;
    /// `text.insert_char(1, 0x57)?`.
    pub fn insert_char(&mut self, index: i32, unit: u16) -> Result<(), Failure> {
        self.check_insert_position(index)?;
        self.terminated.insert(offset(index), unit);
        Ok(())
    }
    /// Inserts a converted string or code-unit slice at a one-based position in 1..=len+1;
    /// `text.insert(1, &ExtendedString::new("ab"))?`.
    pub fn insert(&mut self, index: i32, input: impl Into<Self>) -> Result<(), Failure> {
        self.check_insert_position(index)?;
        let input = input.into();
        let at = offset(index);
        self.terminated
            .splice(at..at, input.as_utf16().iter().copied());
        Ok(())
    }
    /// The (pointer, length) Insert: the position is checked first, then a null pointer or a
    /// length <= 0 inserts nothing; `text.insert_span(1, Some(&[65]), 1)?`.
    pub fn insert_span(
        &mut self,
        index: i32,
        units: Option<&[u16]>,
        length: i32,
    ) -> Result<(), Failure> {
        self.check_insert_position(index)?;
        let (Some(units), Ok(count)) = (units, usize::try_from(length)) else {
            return Ok(());
        };
        self.insert(index, leading_units(units, count))
    }
    /// Replaces one code unit at a one-based position in 1..=len; `text.set_value(6, 0x67)?`.
    pub fn set_value(&mut self, index: i32, unit: u16) -> Result<(), Failure> {
        if index < 1 || index > self.length() {
            return Err(out_of_range(
                "TCollection_ExtendedString::SetValue : parameter theWhere",
                format_args!("theWhere {index}; expected 1..={}", self.len()),
            ));
        }
        self.terminated[offset(index)] = unit;
        Ok(())
    }
    /// Overwrites from a one-based position in 1..=len+1, growing the string when the
    /// replacement runs past its end; `text.set_value_units(2, &[65, 66])?`.
    pub fn set_value_units(&mut self, index: i32, units: impl AsRef<[u16]>) -> Result<(), Failure> {
        let units = units.as_ref();
        if index < 1 || index > self.length() + 1 {
            return Err(out_of_range(
                "TCollection_ExtendedString::SetValue : parameter theWhere",
                format_args!("theWhere {index}; expected 1..={}", self.len() + 1),
            ));
        }
        let at = offset(index);
        let end = (at + units.len()).max(self.len());
        self.terminated.resize(end + 1, 0);
        self.terminated[at..at + units.len()].copy_from_slice(units);
        Ok(())
    }
    /// The (pointer, length) SetValue: the position is checked first, then a null pointer
    /// or a length <= 0 changes nothing; `text.set_value_span(1, Some(&[65]), 1)?`.
    pub fn set_value_span(
        &mut self,
        index: i32,
        units: Option<&[u16]>,
        length: i32,
    ) -> Result<(), Failure> {
        let (Some(units), Ok(count)) = (units, usize::try_from(length)) else {
            return self.set_value_units(index, []);
        };
        self.set_value_units(index, leading_units(units, count))
    }
    /// Erases `count` units from a one-based position; the position must be in 1..=len and
    /// the range must end inside the string; `text.remove(2, 2)?`.
    pub fn remove(&mut self, index: i32, count: i32) -> Result<(), Failure> {
        // i64 keeps theWhere + theHowMany exact where OCCT's int sum would overflow.
        let end = i64::from(index) + i64::from(count);
        if index < 1 || index > self.length() || count < 0 || end > i64::from(self.length()) + 1 {
            return Err(self.remove_error(index, count));
        }
        let at = offset(index);
        self.terminated.drain(at..at + count as usize);
        Ok(())
    }
    /// Removes every occurrence of a code unit, zero included; `text.remove_all(0x20)`.
    pub fn remove_all(&mut self, unit: u16) {
        let length = self.len();
        let mut kept: Vec<u16> = self.terminated[..length]
            .iter()
            .copied()
            .filter(|&current| current != unit)
            .collect();
        kept.push(0);
        self.terminated = kept;
    }
    /// Keeps the first `length` units, `length` in 0..=len; `text.trunc(3)?`.
    pub fn trunc(&mut self, length: i32) -> Result<(), Failure> {
        let Some(kept) = usize::try_from(length)
            .ok()
            .filter(|&kept| kept <= self.len())
        else {
            return Err(out_of_range(
                "TCollection_ExtendedString::Trunc : parameter 'theHowMany'",
                format_args!("theHowMany {length}; expected 0..={}", self.len()),
            ));
        };
        self.terminated.truncate(kept);
        self.terminated.push(0);
        Ok(())
    }
    /// Keeps the first `index` units (0 <= index < len) and returns the rest. OCCT builds
    /// the returned part with the `const char16_t*` constructor, so it ends at the first
    /// zero unit of the tail; `text.split(3)?`.
    pub fn split(&mut self, index: i32) -> Result<Self, Failure> {
        let Some(at) = usize::try_from(index).ok().filter(|&at| at < self.len()) else {
            return Err(out_of_range(
                "TCollection_ExtendedString::Split index",
                format_args!("theWhere {index}; expected 0..{}", self.len()),
            ));
        };
        let tail = Self::from_utf16(&self.as_utf16()[at..]);
        self.trunc(index)?;
        Ok(tail)
    }
    /// Replaces every occurrence of a code unit; `text.change_all(0x48, 0x4D)`.
    pub fn change_all(&mut self, from: u16, to: u16) {
        let length = self.len();
        for unit in &mut self.terminated[..length] {
            if *unit == from {
                *unit = to;
            }
        }
    }
    /// Replaces the content with a string or code-unit slice (`Copy`, `operator=`);
    /// `text.copy(&[65, 66])`.
    pub fn copy(&mut self, units: impl AsRef<[u16]>) {
        self.terminated.clear();
        self.terminated.extend_from_slice(units.as_ref());
        self.terminated.push(0);
    }
    /// Copies a bounded UTF-16 C string through its first NUL; `text.copy_c_units(Some(&[65, 0, 66]))`.
    /// `None` and an empty prefix clear the string, as the native pointer overload does.
    pub fn copy_c_units(&mut self, units: Option<&[u16]>) -> &mut Self {
        self.copy(units.map_or(&[][..], utf::c_units_prefix));
        self
    }
    /// The (pointer, length) Copy: a null pointer or a length <= 0 empties the string;
    /// `text.copy_span(None, 3)`.
    pub fn copy_span(&mut self, units: Option<&[u16]>, length: i32) {
        match (units, usize::try_from(length)) {
            (Some(units), Ok(count)) => self.copy(leading_units(units, count)),
            _ => self.clear(),
        }
    }
    /// Empties the string; `text.clear()`.
    pub fn clear(&mut self) {
        self.terminated.clear();
        self.terminated.push(0);
    }
    /// Exchanges the content of two strings; `first.swap(&mut second)`.
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other);
    }
    /// Takes the source's content and leaves the source empty (`Move`);
    /// `target.move_assign(&mut source)`.
    pub fn move_assign(&mut self, source: &mut Self) {
        *self = std::mem::take(source);
    }
    fn remove_error(&self, index: i32, count: i32) -> Failure {
        out_of_range(
            "TCollection_ExtendedString::Remove: Invalid parameters",
            format_args!(
                "theWhere {index}, theHowMany {count}; expected 1 <= theWhere <= {}, theHowMany >= 0 and theWhere + theHowMany <= {}",
                self.len(),
                self.len() + 1
            ),
        )
    }
    /// Checks Insert's position in the order OCCT raises: too big, then below one.
    fn check_insert_position(&self, index: i32) -> Result<(), Failure> {
        if i64::from(index) > i64::from(int_length(self.len())) + 1 {
            return Err(out_of_range(
                "TCollection_ExtendedString::Insert : Parameter theWhere is too big",
                format_args!("theWhere {index}; expected 1..={}", self.len() + 1),
            ));
        }
        if index < 1 {
            return Err(out_of_range(
                "TCollection_ExtendedString::Insert : Parameter theWhere is out of range",
                format_args!("theWhere {index}; expected 1..={}", self.len() + 1),
            ));
        }
        Ok(())
    }
}
