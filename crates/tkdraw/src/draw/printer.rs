//! `Draw_Printer`: a message printer that appends to the interpreter result.
use super::interpretor::DrawInterpretor;
use super::interpretor_link::InterpretorLink;
use rustycad_tkernel::message::{Gravity, Printer, PrinterBase, PrinterType};
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;

/// `Draw_Printer`: each message that passes the trace level becomes `text\n` in the
/// result of the interpreter (see [`InterpretorLink`] for messages from other threads).
///
/// ```
/// use rustycad_tkdraw::draw::{DrawInterpretor, DrawPrinter, RecordingConsole};
/// use rustycad_tkernel::message::{Gravity, Printer};
/// use std::rc::Rc;
/// let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
/// di.init();
/// let printer = DrawPrinter::new(&di);
/// printer.send_c_str(b"hello", Gravity::Warning);
/// assert_eq!(di.result().unwrap(), "hello\n");
/// ```
pub struct DrawPrinter {
    base: PrinterBase,
    link: InterpretorLink,
}

/// Marker type carrying the `Draw_Printer` runtime descriptor.
pub struct DrawPrinterType;

rustycad_tkernel::message_runtime_type!(DrawPrinterType, "Draw_Printer", PrinterType);

impl DrawPrinter {
    /// `Draw_Printer(theTcl)`: a printer at the default trace level (info).
    pub fn new(di: &DrawInterpretor) -> Self {
        Self {
            base: PrinterBase::default(),
            link: InterpretorLink::new(di),
        }
    }
}

impl Printer for DrawPrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }

    fn dynamic_type(&self) -> &'static RuntimeType {
        DrawPrinterType::static_type()
    }

    /// The protected `send`: ignores messages below the trace level (checked again, as
    /// upstream), then appends the text and a newline.
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        if gravity < self.trace_level() {
            return;
        }
        let line = format!("{}\n", String::from_utf8_lossy(text.as_bytes()));
        self.link.append(&line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::RecordingConsole;
    use std::rc::Rc;

    #[test]
    fn messages_below_the_trace_level_are_dropped() {
        let di = DrawInterpretor::new(Rc::new(RecordingConsole::default()));
        di.init();
        let printer = DrawPrinter::new(&di);
        printer.set_trace_level(Gravity::Fail);
        printer.send(&AsciiString::new("ignored"), Gravity::Info);
        assert_eq!(di.result().unwrap(), "");
        assert_eq!(printer.dynamic_type().name(), "Draw_Printer");
    }
}
