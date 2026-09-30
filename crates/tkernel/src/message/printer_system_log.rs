//! `Message_PrinterSystemLog`: messages to syslog (Unix) or the Event Log (Windows).
use super::{Gravity, Printer, PrinterBase, PrinterType};
use crate::standard::{RuntimeType, RuntimeTyped};
use crate::tcollection::AsciiString;
mod system;

/// Printer to the operating system log; `PrinterSystemLog::new(&AsciiString::new("App"), Gravity::Info)`.
pub struct PrinterSystemLog {
    base: PrinterBase,
    event_source: system::EventSource,
}
message_runtime_type!(PrinterSystemLog, "Message_PrinterSystemLog", PrinterType);
impl PrinterSystemLog {
    /// Opens the log under a source name (`openlog`/`RegisterEventSourceW`);
    /// `PrinterSystemLog::new(&AsciiString::new("App"), Gravity::Info)`.
    pub fn new(event_source_name: &AsciiString, trace_level: Gravity) -> Self {
        Self {
            base: PrinterBase::new(trace_level),
            event_source: system::EventSource::open(event_source_name),
        }
    }
}
impl Printer for PrinterSystemLog {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        if gravity < self.trace_level() {
            return;
        }
        self.event_source.write(text, gravity);
    }
}
