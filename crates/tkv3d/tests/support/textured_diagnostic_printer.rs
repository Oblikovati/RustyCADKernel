//! Named native message I/O fake retains raw bytes and gravity without writing stderr.
use rustycad_tkernel::{
    message::{Gravity, Printer, PrinterBase, PrinterType},
    standard::{RuntimeType, RuntimeTyped, SharedState},
    tcollection::AsciiString,
};
#[derive(Default)]
pub struct TexturedDiagnosticPrinter {
    base: PrinterBase,
    pub messages: SharedState<Vec<(Gravity, AsciiString)>>,
}
impl Printer for TexturedDiagnosticPrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        self.messages.write().push((gravity, text.clone()));
    }
}
