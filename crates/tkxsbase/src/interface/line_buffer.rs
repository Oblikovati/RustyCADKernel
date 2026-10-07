// Rust port of DataExchange/TKXSBase/Interface/Interface_LineBuffer (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_LineBuffer`: a line of text built piece by piece, with a maximum length, an
//! initial blank margin and a kept tail that starts the next line.
use rustycad_tkernel::{
    standard::{Failure, Handle, exceptions},
    tcollection::{AsciiString, HAsciiString},
};

/// `Interface_LineBuffer`. The characters live in a C string of `size + 1` places
/// (natively `NCollection_Array1<char>(1, size + 1)`, whose checked accesses raise
/// `Standard_OutOfRange`).
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceLineBuffer;
/// let mut line = InterfaceLineBuffer::new(20);
/// line.set_initial(2).unwrap();
/// line.add_text("abc").unwrap();
/// assert_eq!(line.moved().unwrap().string().as_bytes(), b"  abc");
/// ```
#[derive(Clone, Debug)]
pub struct InterfaceLineBuffer {
    my_line: Vec<u8>,
    my_max: i32,
    my_init: i32,
    my_keep: i32,
    my_get: i32,
    my_len: i32,
    my_friz: i32,
    my_kept: u8,
}

impl Default for InterfaceLineBuffer {
    fn default() -> Self {
        Self::new(10)
    }
}

impl InterfaceLineBuffer {
    /// `Interface_LineBuffer(size = 10)`.
    pub fn new(size: i32) -> Self {
        let places = usize::try_from(size + 1).unwrap_or(0);
        Self {
            my_line: vec![0; places],
            my_max: size,
            my_init: 0,
            my_keep: 0,
            my_get: 0,
            my_len: 0,
            my_friz: 0,
            my_kept: 0,
        }
    }
    /// `SetMax(max)`: a maximum over the place count raises `Standard_OutOfRange`; a
    /// non-positive one is the place count.
    pub fn set_max(&mut self, max: i32) -> Result<(), Failure> {
        let places = self.my_line.len() as i32;
        if max > places {
            return Err(exceptions::out_of_range(&format!(
                "Interface LineBuffer : SetMax: {max}; expected at most {places}"
            )));
        }
        self.my_max = if max <= 0 { places } else { max };
        Ok(())
    }
    /// `SetInitial(initial)`: the blank margin of the next line (ignored while frozen);
    /// `Standard_OutOfRange` when not below the maximum.
    pub fn set_initial(&mut self, initial: i32) -> Result<(), Failure> {
        if self.my_friz > 0 {
            return Ok(());
        }
        if initial >= self.my_max {
            return Err(exceptions::out_of_range(&format!(
                "Interface LineBuffer : SetInitial: {initial}; expected below {}",
                self.my_max
            )));
        }
        self.my_init = initial.max(0);
        Ok(())
    }
    /// `SetKeep`: what is added from now on is kept for the next line if this one is full.
    pub fn set_keep(&mut self) {
        self.my_keep = -self.my_len;
    }
    /// `CanGet(more)`: whether `more` characters fit; when not, the keep mark is armed.
    pub fn can_get(&mut self, more: i32) -> bool {
        self.my_get = more;
        if self.my_len + self.my_init + more <= self.my_max {
            return true;
        }
        if self.my_keep < 0 {
            self.my_keep = -self.my_keep;
        }
        false
    }
    /// `Content`: the text up to its NUL.
    pub fn content(&self) -> &[u8] {
        let end = self
            .my_line
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(self.my_line.len());
        &self.my_line[..end]
    }
    /// `Length`: the length with the margin.
    pub fn length(&self) -> i32 {
        self.my_len + self.my_init
    }
    /// `Clear`: also forgets a frozen margin (so a `Move` that clears the line does not
    /// restore it, a native quirk kept).
    pub fn clear(&mut self) -> Result<(), Failure> {
        self.my_get = 0;
        self.my_keep = 0;
        self.my_len = 0;
        self.my_friz = 0;
        self.set(1, 0)
    }
    /// `FreezeInitial`: the margin applies again only after the next move.
    pub fn freeze_initial(&mut self) {
        self.my_friz = self.my_init + 1;
        self.my_init = 0;
    }
    /// `Move(TCollection_AsciiString&)`: appends the line to `text` and starts the next.
    pub fn move_to(&mut self, text: &mut AsciiString) -> Result<(), Failure> {
        self.prepare()?;
        text.assign_cat(self.content());
        self.keep()
    }
    /// `Move(const occ::handle<TCollection_HAsciiString>&)`.
    pub fn move_to_h_string(&mut self, text: &HAsciiString) -> Result<(), Failure> {
        self.prepare()?;
        text.assign_cat(self.content());
        self.keep()
    }
    /// `Moved`: the line as a new string, then starts the next.
    pub fn moved(&mut self) -> Result<Handle<HAsciiString>, Failure> {
        self.prepare()?;
        let line = Handle::new(HAsciiString::new(self.content()));
        self.keep()?;
        Ok(line)
    }
    /// `Add(const char*)`: up to the place left.
    pub fn add_text(&mut self, text: &str) -> Result<(), Failure> {
        self.add_bytes(text.as_bytes(), text.len() as i32)
    }
    /// `Add(const char*, lntext)`: at most `lntext` bytes, up to the place left.
    pub fn add_bytes(&mut self, text: &[u8], lntext: i32) -> Result<(), Failure> {
        let room = self.my_max - self.my_len - self.my_init;
        let lnt = lntext.min(room);
        for i in 1..=lnt {
            let byte = text.get((i - 1) as usize).copied().unwrap_or(0);
            self.set(self.my_len + i, byte)?;
        }
        self.my_len += lnt.max(0);
        self.set(self.my_len + 1, 0)
    }
    /// `Add(const TCollection_AsciiString&)`.
    pub fn add_string(&mut self, text: &AsciiString) -> Result<(), Failure> {
        self.add_bytes(text.as_bytes(), text.length())
    }
    /// `Add(const char)`: without check of the maximum, as natively.
    pub fn add_char(&mut self, byte: u8) -> Result<(), Failure> {
        self.set(self.my_len + 1, byte)?;
        self.my_len += 1;
        self.set(self.my_len + 1, 0)
    }
    /// `Prepare` (private): inserts the margin and cuts the line at the keep mark.
    fn prepare(&mut self) -> Result<(), Failure> {
        if self.my_init > 0 && self.my_len + self.my_init <= self.my_max {
            for i in (1..=self.my_len + 1).rev() {
                let byte = self.get(i)?;
                self.set(i + self.my_init, byte)?;
            }
            (1..=self.my_init).try_for_each(|i| self.set(i, b' '))?;
        }
        if self.my_keep > 0 {
            self.my_keep += self.my_init + 1; // myInit, and +1 because Keep INCLUDED
        }
        if self.my_keep > 0
            && self.my_len + self.my_get + self.my_init - self.my_keep >= self.my_max
        {
            self.my_keep = 0;
        }
        if self.my_keep > 0 {
            self.my_kept = self.get(self.my_keep)?;
            self.set(self.my_keep, 0)?;
        }
        Ok(())
    }
    /// `Keep` (private): moves the kept tail to the start, or clears the line.
    fn keep(&mut self) -> Result<(), Failure> {
        if self.my_keep > 0 {
            self.set(1, self.my_kept)?;
            for i in self.my_keep + 1..=self.my_len + self.my_init + 1 {
                let byte = self.get(i)?;
                self.set(i - self.my_keep + 1, byte)?;
            }
            self.my_len = self.my_len + self.my_init - self.my_keep + 1;
        } else {
            self.clear()?;
        }
        self.my_get = 0;
        self.my_keep = 0;
        if self.my_friz > 0 {
            self.my_init = self.my_friz - 1;
            self.my_friz = 0;
        }
        Ok(())
    }
    fn get(&self, index: i32) -> Result<u8, Failure> {
        let slot = usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.my_line.get(i));
        slot.copied().ok_or_else(|| self.out_of_range(index))
    }
    fn set(&mut self, index: i32, byte: u8) -> Result<(), Failure> {
        let error = self.out_of_range(index);
        let slot = usize::try_from(index - 1)
            .ok()
            .and_then(|i| self.my_line.get_mut(i));
        *slot.ok_or(error)? = byte;
        Ok(())
    }
    fn out_of_range(&self, index: i32) -> Failure {
        exceptions::out_of_range(&format!(
            "Interface_LineBuffer: index {index}; expected [1, {}]",
            self.my_line.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_moves_the_tail_to_the_next_line() {
        let mut line = InterfaceLineBuffer::new(10);
        line.add_text("abcd").unwrap();
        line.set_keep();
        line.add_text("efg").unwrap();
        assert!(!line.can_get(5));
        let first = line.moved().unwrap();
        assert_eq!(first.string().as_bytes(), b"abcd");
        assert_eq!(line.content(), b"efg");
        assert_eq!(line.length(), 3);
    }

    #[test]
    fn limits_and_frozen_margin() {
        let mut line = InterfaceLineBuffer::new(5);
        assert!(line.set_max(7).is_err());
        assert!(line.set_initial(5).is_err());
        line.set_initial(1).unwrap();
        line.freeze_initial();
        line.set_initial(3).unwrap();
        line.add_text("123456789").unwrap();
        assert_eq!(line.content(), b"12345");
        let mut text = AsciiString::new(">");
        line.move_to(&mut text).unwrap();
        assert_eq!(text.as_bytes(), b">12345");
        // Native quirk kept: the move cleared the line, and Clear forgot the frozen
        // margin before Keep could restore it.
        line.add_text("abcde").unwrap();
        assert_eq!(line.content(), b"abcde");
        assert!(line.add_char(b'x').is_err());
        line.clear().unwrap();
        line.set_max(0).unwrap();
        line.add_string(&AsciiString::new("z")).unwrap();
        let shared = HAsciiString::new("");
        line.move_to_h_string(&shared).unwrap();
        assert_eq!(shared.string().as_bytes(), b"z");
    }
}
