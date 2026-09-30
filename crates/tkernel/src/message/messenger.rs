//! `Message_Messenger`: dispatches each message to every printer of an ordered sequence.
use super::{Gravity, MessageStream, PrinterHandle, PrinterOStream};
use crate::standard::dump::DumpStream;
use crate::standard::{Handle, RuntimeType, StandardTransient};
use crate::tcollection::{AsciiString, ExtendedString};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Ordered printers with identity-based duplicate detection; `Messenger::with_printer(printer)`.
///
/// The sequence sits behind a lock because messengers are shared (`Message::default_messenger`
/// is process-wide). Delivery iterates a snapshot, so a printer may use its messenger.
pub struct Messenger {
    printers: RwLock<Vec<PrinterHandle>>,
}
message_runtime_type!(Messenger, "Message_Messenger", StandardTransient);
impl Default for Messenger {
    /// One `PrinterOStream` on standard output, so messages are never lost by default.
    fn default() -> Self {
        Self::with_printer(Arc::new(PrinterOStream::new(Gravity::Info)))
    }
}
impl Messenger {
    /// `Message_Messenger()`: a single printer on standard output; `Messenger::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Creates a messenger with one printer; `Messenger::with_printer(printer)`.
    pub fn with_printer(printer: PrinterHandle) -> Self {
        Self {
            printers: RwLock::new(vec![printer]),
        }
    }
    /// Borrows the printer sequence; `messenger.printers().len()`.
    pub fn printers(&self) -> RwLockReadGuard<'_, Vec<PrinterHandle>> {
        self.printers
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// Edits the printer sequence (`ChangePrinters`); `messenger.printers_mut().clear()`.
    pub fn printers_mut(&self) -> RwLockWriteGuard<'_, Vec<PrinterHandle>> {
        self.printers
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    fn snapshot(&self) -> Vec<PrinterHandle> {
        self.printers().clone()
    }
    /// Appends unless the same printer is present; `messenger.add_printer(printer)`.
    pub fn add_printer(&self, printer: PrinterHandle) -> bool {
        let mut printers = self.printers_mut();
        if printers
            .iter()
            .any(|existing| same_printer(existing, &printer))
        {
            return false;
        }
        printers.push(printer);
        true
    }
    /// Removes the first printer with the same identity; `messenger.remove_printer(&printer)`.
    pub fn remove_printer(&self, printer: &PrinterHandle) -> bool {
        let mut printers = self.printers_mut();
        let Some(index) = printers
            .iter()
            .position(|existing| same_printer(existing, printer))
        else {
            return false;
        };
        printers.remove(index);
        true
    }
    /// Removes printers of a type or its descendants; returns the count; `messenger.remove_printers(t)`.
    pub fn remove_printers(&self, kind: &RuntimeType) -> i32 {
        let mut printers = self.printers_mut();
        let before = printers.len();
        printers.retain(|printer| !printer.dynamic_type().is_subtype(kind));
        (before - printers.len()) as i32
    }
}

/// Handle identity: the same allocation, whatever the trait-object metadata.
pub(crate) fn same_printer(first: &PrinterHandle, second: &PrinterHandle) -> bool {
    std::ptr::addr_eq(Arc::as_ptr(first), Arc::as_ptr(second))
}

impl Messenger {
    /// `Send(const char*, Message_Warning)`: the native default gravity; `messenger.send("text")`.
    pub fn send(&self, text: &str) {
        self.send_c_str(text.as_bytes(), Gravity::Warning);
    }
    /// `Send(const char*, gravity)`: bytes up to the first NUL; `messenger.send_c_str(b"text", Gravity::Info)`.
    pub fn send_c_str(&self, text: &[u8], gravity: Gravity) {
        for printer in self.snapshot() {
            printer.send_c_str(text, gravity);
        }
    }
    /// `Send(const Standard_SStream&, gravity)`; `messenger.send_string_stream(b"text", g)`.
    pub fn send_string_stream(&self, stream: &[u8], gravity: Gravity) {
        for printer in self.snapshot() {
            printer.send_string_stream(stream, gravity);
        }
    }
    /// `Send(const TCollection_AsciiString&, gravity)`; `messenger.send_ascii(&text, g)`.
    pub fn send_ascii(&self, text: &AsciiString, gravity: Gravity) {
        for printer in self.snapshot() {
            printer.send_ascii(text, gravity);
        }
    }
    /// `Send(const TCollection_ExtendedString&, gravity)`; `messenger.send_extended(&text, g)`.
    pub fn send_extended(&self, text: &ExtendedString, gravity: Gravity) {
        for printer in self.snapshot() {
            printer.send_extended(text, gravity);
        }
    }
    /// `Send(const occ::handle<Standard_Transient>&, gravity)`; `messenger.send_object(&object, g)`.
    pub fn send_object(&self, object: &Handle<StandardTransient>, gravity: Gravity) {
        for printer in self.snapshot() {
            printer.send_object(object, gravity);
        }
    }
    /// `Send(Message_Gravity)`: a buffer sent when dropped or flushed; `messenger.stream(Gravity::Info)`.
    pub fn stream(&self, gravity: Gravity) -> MessageStream<'_> {
        MessageStream::new(self, gravity)
    }
    /// `SendFail()`; `write!(messenger.send_fail(), "text")`.
    pub fn send_fail(&self) -> MessageStream<'_> {
        self.stream(Gravity::Fail)
    }
    /// `SendAlarm()`; `write!(messenger.send_alarm(), "text")`.
    pub fn send_alarm(&self) -> MessageStream<'_> {
        self.stream(Gravity::Alarm)
    }
    /// `SendWarning()`; `write!(messenger.send_warning(), "text")`.
    pub fn send_warning(&self) -> MessageStream<'_> {
        self.stream(Gravity::Warning)
    }
    /// `SendInfo()`; `write!(messenger.send_info(), "text")`.
    pub fn send_info(&self) -> MessageStream<'_> {
        self.stream(Gravity::Info)
    }
    /// `SendTrace()`; `write!(messenger.send_trace(), "text")`.
    pub fn send_trace(&self) -> MessageStream<'_> {
        self.stream(Gravity::Trace)
    }
    /// `SendFail(const TCollection_AsciiString&)`; `messenger.send_fail_text(&text)`.
    pub fn send_fail_text(&self, text: &AsciiString) {
        self.send_ascii(text, Gravity::Fail);
    }
    /// `SendAlarm(const TCollection_AsciiString&)`; `messenger.send_alarm_text(&text)`.
    pub fn send_alarm_text(&self, text: &AsciiString) {
        self.send_ascii(text, Gravity::Alarm);
    }
    /// `SendWarning(const TCollection_AsciiString&)`; `messenger.send_warning_text(&text)`.
    pub fn send_warning_text(&self, text: &AsciiString) {
        self.send_ascii(text, Gravity::Warning);
    }
    /// `SendInfo(const TCollection_AsciiString&)`; `messenger.send_info_text(&text)`.
    pub fn send_info_text(&self, text: &AsciiString) {
        self.send_ascii(text, Gravity::Info);
    }
    /// `SendTrace(const TCollection_AsciiString&)`; `messenger.send_trace_text(&text)`.
    pub fn send_trace_text(&self, text: &AsciiString) {
        self.send_ascii(text, Gravity::Trace);
    }
    /// `DumpJson`: class name and `Printers.Size`; `messenger.dump_json(&mut stream, -1)`.
    pub fn dump_json(&self, output: &mut DumpStream, _depth: i32) {
        super::json::class_begin(output, "Message_Messenger");
        super::json::field_integer(output, "Printers.Size", self.printers().len() as i32);
    }
}
