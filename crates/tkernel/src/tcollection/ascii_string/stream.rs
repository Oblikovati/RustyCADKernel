//! `Print`/`operator<<` and `Read`/`operator>>` over injected byte streams.
use super::{AsciiString, c_string_length};
use crate::standard::character;
use std::io::{BufRead, Write};

/// `Read`'s stack buffer: `operator>>` stores at most this many bytes minus the NUL.
const READ_BUFFER_SIZE: usize = 8190;

impl AsciiString {
    /// `Print`/`operator<<`: writes the C string, so output stops at an embedded NUL;
    /// `text.print(&mut output)?`.
    pub fn print(&self, output: &mut dyn Write) -> std::io::Result<()> {
        output.write_all(&self.as_bytes()[..c_string_length(self.as_bytes())])
    }
    /// `Read`/`operator>>`: skips C-locale whitespace, then takes one word of at most 8189
    /// bytes, leaving the delimiter unread; the string keeps the word through its first NUL.
    /// Returns false, as the stream's failbit would, when no byte was taken;
    /// `text.read(&mut std::io::Cursor::new(b"  word rest"))?`.
    ///
    /// OCCT leaves its buffer uninitialized when the stream holds only whitespace; the
    /// string becomes empty here.
    pub fn read(&mut self, input: &mut dyn BufRead) -> std::io::Result<bool> {
        skip_spaces(input)?;
        let word = take_word(input, READ_BUFFER_SIZE - 1)?;
        *self = Self::from_bytes(&word);
        Ok(!word.is_empty())
    }
}

/// Consumes leading whitespace, as the `istream::sentry` does.
fn skip_spaces(input: &mut dyn BufRead) -> std::io::Result<()> {
    loop {
        let available = input.fill_buf()?;
        let spaces = available
            .iter()
            .take_while(|&&byte| character::is_space(byte))
            .count();
        let exhausted = spaces < available.len() || available.is_empty();
        input.consume(spaces);
        if exhausted {
            return Ok(());
        }
    }
}

/// Consumes up to `limit` non-space bytes and returns them.
fn take_word(input: &mut dyn BufRead, limit: usize) -> std::io::Result<Vec<u8>> {
    let mut word = Vec::new();
    while word.len() < limit {
        let available = input.fill_buf()?;
        let wanted = available
            .iter()
            .take(limit - word.len())
            .take_while(|&&byte| !character::is_space(byte))
            .count();
        word.extend_from_slice(&available[..wanted]);
        // A delimiter, the limit or the end of input inside this chunk ends the word.
        let stopped = available.is_empty() || wanted < available.len();
        input.consume(wanted);
        if stopped {
            break;
        }
    }
    Ok(word)
}
