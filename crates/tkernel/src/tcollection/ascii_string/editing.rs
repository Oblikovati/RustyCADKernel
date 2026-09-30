//! One-based element access, insertion, overwriting, removal, truncation and copying.
use super::AsciiString;
use super::raise::out_of_range;
use crate::standard::Failure;

impl AsciiString {
    /// `Value`: the byte at a one-based position; `AsciiString::new("ab").value(2)? == b'b'`.
    pub fn value(&self, position: i32) -> Result<u8, Failure> {
        if position > 0 && position <= self.length() {
            return Ok(self.terminated[position as usize - 1]);
        }
        Err(out_of_range(
            "Value",
            format!("position {position}; expected 1..={}", self.length()),
        ))
    }
    /// `SubString`: bytes `from..=to`, copied through the first NUL as the `(const char*, int)`
    /// constructor does; `AsciiString::new("abcdef").sub_string(2, 4)?.as_bytes() == b"bcd"`.
    pub fn sub_string(&self, from: i32, to: i32) -> Result<Self, Failure> {
        if from <= 0 || to <= 0 || to > self.length() || from > to {
            return Err(out_of_range(
                "SubString",
                format!(
                    "range {from}..={to}; expected 1 <= from <= to <= {}",
                    self.length()
                ),
            ));
        }
        Ok(Self::from_bytes(
            &self.as_bytes()[from as usize - 1..to as usize],
        ))
    }
    /// `Insert(int, char)`: inserts one byte, NUL included, before `position` in `1..=len+1`;
    /// `text.insert_char(1, b'>')?`.
    pub fn insert_char(&mut self, position: i32, byte: u8) -> Result<(), Failure> {
        if position > self.length() + 1 {
            return Err(out_of_range(
                "Insert",
                format!(
                    "position {position} is too big; expected <= {}",
                    self.length() + 1
                ),
            ));
        }
        if position < 1 {
            return Err(out_of_range(
                "Insert",
                format!("position {position} is too small; expected >= 1"),
            ));
        }
        self.splice_bytes(position as usize - 1, &[byte]);
        Ok(())
    }
    /// `Insert(int, const char*, int)` and its string overloads: inserts a byte span before
    /// `position` in `1..=len+1`; `text.insert(1, "prefix")?`.
    pub fn insert(&mut self, position: i32, what: impl AsRef<[u8]>) -> Result<(), Failure> {
        let offset = self.insertion_offset(position)?;
        self.splice_bytes(offset, what.as_ref());
        Ok(())
    }
    /// `Insert(int, const char*, int)` with a pointer into this string: inserts a copy of the
    /// zero-based `source` span; `text.insert_self(2, 0..3)?`.
    pub fn insert_self(
        &mut self,
        position: i32,
        source: std::ops::Range<usize>,
    ) -> Result<(), Failure> {
        let offset = self.insertion_offset(position)?;
        let copy = self.as_bytes()[source].to_vec();
        self.splice_bytes(offset, &copy);
        Ok(())
    }
    /// `InsertAfter`: inserts after `index` in `0..=len`; `text.insert_after(0, "head")?`.
    pub fn insert_after(&mut self, index: i32, what: impl AsRef<[u8]>) -> Result<(), Failure> {
        if index < 0 || index > self.length() {
            return Err(out_of_range(
                "InsertAfter",
                format!("index {index}; expected 0..={}", self.length()),
            ));
        }
        self.insert(index + 1, what)
    }
    /// `InsertBefore`: inserts before `index` in `1..=len`; `text.insert_before(1, "head")?`.
    pub fn insert_before(&mut self, index: i32, what: impl AsRef<[u8]>) -> Result<(), Failure> {
        if index < 1 || index > self.length() {
            return Err(out_of_range(
                "InsertBefore",
                format!("index {index}; expected 1..={}", self.length()),
            ));
        }
        self.insert(index, what)
    }
    fn insertion_offset(&self, position: i32) -> Result<usize, Failure> {
        if position < 1 || position > self.length() + 1 {
            return Err(out_of_range(
                "Insert",
                format!("position {position}; expected 1..={}", self.length() + 1),
            ));
        }
        Ok(position as usize - 1)
    }
}

impl AsciiString {
    /// `SetValue(int, char)`: replaces one existing byte; a NUL byte is refused like an
    /// out-of-range position; `text.set_value_char(1, b'h')?`.
    pub fn set_value_char(&mut self, position: i32, byte: u8) -> Result<(), Failure> {
        if position <= 0 || position > self.length() {
            return Err(out_of_range(
                "SetValue",
                format!("position {position}; expected 1..={}", self.length()),
            ));
        }
        if byte == 0 {
            return Err(out_of_range(
                "SetValue",
                "byte 0 (NULL terminator); expected a nonzero byte".into(),
            ));
        }
        self.terminated[position as usize - 1] = byte;
        Ok(())
    }
    /// `SetValue(int, const char*, int)` and its string overloads: overwrites from `position`,
    /// growing the string when the span runs past its end; `text.set_value(3, "XYZ")?`.
    ///
    /// OCCT leaves the bytes between the old end and a `position` beyond `len + 1`
    /// uninitialized; that position is refused here instead.
    pub fn set_value(&mut self, position: i32, what: impl AsRef<[u8]>) -> Result<(), Failure> {
        if position <= 0 || position > self.length() + 1 {
            return Err(out_of_range(
                "SetValue",
                format!("position {position}; expected 1..={}", self.length() + 1),
            ));
        }
        let what = what.as_ref();
        let offset = position as usize - 1;
        let end = offset + what.len();
        if end > self.len() {
            self.terminated.resize(end + 1, 0);
        }
        self.terminated[offset..end].copy_from_slice(what);
        Ok(())
    }
    /// `Remove`: erases `count` bytes from `position` when `position + count <= len + 1`;
    /// `AsciiString::new("abcdef").remove(2, 3)?` leaves `"aef"`.
    ///
    /// A position below 1 or a negative count is undefined behavior in OCCT and refused here.
    pub fn remove(&mut self, position: i32, count: i32) -> Result<(), Failure> {
        let end = i64::from(position) + i64::from(count);
        if position < 1 || count < 0 || end > i64::from(self.length()) + 1 {
            return Err(out_of_range(
                "Remove",
                format!(
                    "{count} bytes from {position}; expected 1 <= position, 0 <= count, position + count <= {}",
                    self.length() + 1
                ),
            ));
        }
        let offset = position as usize - 1;
        self.terminated.drain(offset..offset + count as usize);
        Ok(())
    }
    /// `Trunc`: keeps the first `length` bytes, `length` in `0..=len`; `text.truncate(5)?`.
    pub fn truncate(&mut self, length: i32) -> Result<(), Failure> {
        if length < 0 || length > self.length() {
            return Err(out_of_range(
                "Trunc",
                format!("length {length}; expected 0..={}", self.length()),
            ));
        }
        self.set_length(length as usize);
        Ok(())
    }
    /// `Split`: keeps the first `length` bytes and returns the rest, copied through its first
    /// NUL; `let tail = text.split(5)?`.
    pub fn split(&mut self, length: i32) -> Result<Self, Failure> {
        if length < 0 || length > self.length() {
            return Err(out_of_range(
                "Split",
                format!("index {length}; expected 0..={}", self.length()),
            ));
        }
        let tail = Self::from_bytes(&self.as_bytes()[length as usize..]);
        self.set_length(length as usize);
        Ok(tail)
    }
    /// `Copy` and `operator=` from a byte span, embedded NULs included; `text.copy("replacement")`.
    pub fn copy(&mut self, input: impl AsRef<[u8]>) {
        let bytes = input.as_ref();
        self.terminated.clear();
        self.terminated.extend_from_slice(bytes);
        self.terminated.push(0);
    }
    /// `Copy(const char*, int)` with a pointer into this string: moves the zero-based `source`
    /// span to the front. A span starting at offset 0 is OCCT's `myString == theString` early
    /// return and leaves the string unchanged; `text.copy_self(2..4)`.
    pub fn copy_self(&mut self, source: std::ops::Range<usize>) {
        assert!(
            source.start <= source.end && source.end <= self.len(),
            "AsciiString source {source:?}; expected a span within 0..{}",
            self.len()
        );
        if source.start == 0 {
            return;
        }
        let length = source.len();
        self.terminated.copy_within(source, 0);
        self.set_length(length);
    }
}
