use super::{Gravity, Messenger};
use std::fmt;
/// Scoped buffered message; `write!(messenger.stream(Gravity::Info), "text")`.
pub struct MessageStream<'a> {
    messenger: &'a Messenger,
    gravity: Gravity,
    text: String,
}
impl<'a> MessageStream<'a> {
    pub(super) fn new(messenger: &'a Messenger, gravity: Gravity) -> Self {
        Self {
            messenger,
            gravity,
            text: String::new(),
        }
    }
    /// Send accumulated contents, optionally sending an empty message; `stream.flush(true)`.
    ///
    /// Passing a manipulator (`std::endl`, `std::ends`, `std::flush`) is `flush(true)`
    /// natively: the manipulator itself never reaches the text.
    pub fn flush(&mut self, force: bool) {
        if force || !self.text.is_empty() {
            self.messenger
                .send_string_stream(self.text.as_bytes(), self.gravity);
            self.text.clear();
        }
    }
    /// Borrow the underlying UTF-8 formatting buffer; `stream.buffer_mut().push_str("text")`.
    pub fn buffer_mut(&mut self) -> &mut String {
        &mut self.text
    }
    /// Borrow the delivery destination; `stream.messenger().printers()`.
    pub fn messenger(&self) -> &Messenger {
        self.messenger
    }
}
impl Clone for MessageStream<'_> {
    fn clone(&self) -> Self {
        // Native formal copy preserves destination/gravity but deliberately discards contents.
        Self::new(self.messenger, self.gravity)
    }
}
impl fmt::Write for MessageStream<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.text.push_str(text);
        Ok(())
    }
}
impl Drop for MessageStream<'_> {
    fn drop(&mut self) {
        self.flush(false);
    }
}
