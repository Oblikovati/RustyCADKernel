//! OCC31189 and the complete legacy stream-buffer suite.
#![allow(non_snake_case)]
use rustycad_tkernel::message::{
    Gravity, Message, Printer, PrinterBase, PrinterHandle, PrinterType,
};
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

/// Custom message printer that collects messages into a shared string.
struct TestMessagePrinter {
    base: PrinterBase,
    output: Arc<Mutex<String>>,
}
impl Printer for TestMessagePrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        if gravity >= Gravity::Info {
            let mut output = self.output.lock().unwrap();
            output.push_str(&String::from_utf8_lossy(text.as_bytes()));
            output.push('\n');
        }
    }
}

/// Both upstream tests swap the printers of the process-wide messenger; serialize them.
static DEFAULT_MESSENGER_GUARD: Mutex<()> = Mutex::new(());

/// Installs a collecting printer on the default messenger; returns the saved printers.
fn install_test_printer(output: &Arc<Mutex<String>>) -> Vec<PrinterHandle> {
    let printer: PrinterHandle = Arc::new(TestMessagePrinter {
        base: PrinterBase::default(),
        output: output.clone(),
    });
    let messenger = Message::default_messenger();
    // `anOriginalPrinters.Append(ChangePrinters())` moves the printers out.
    let original = std::mem::take(&mut *messenger.printers_mut());
    messenger.printers_mut().clear();
    messenger.add_printer(printer);
    original
}

fn restore_printers(original: Vec<PrinterHandle>) {
    let messenger = Message::default_messenger();
    messenger.printers_mut().clear();
    messenger.printers_mut().extend(original);
}

#[test]
fn Message_Messenger_Test__OCC31189_StreamBufferMessageOrdering() {
    let _guard = DEFAULT_MESSENGER_GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let output: Arc<Mutex<String>> = Arc::default();
    let original = install_test_printer(&output);
    let messenger = Message::default_messenger();
    {
        let mut sender = Message::send_info();
        write!(sender, "Sender message 1: start ...").unwrap();
        messenger.send("Direct message 1");
        write!(sender, "... end").unwrap();
        sender.flush(true);
        drop(Message::send_info());
        messenger.send("Direct message 2");
        let mut empty_sender = Message::send_info();
        empty_sender.flush(true);
        write!(sender, "Sender message 2").unwrap();
    }
    restore_printers(original);
    let output = output.lock().unwrap();
    assert!(output.contains("Direct message 1"));
    assert!(output.contains("Sender message 1: start ...... end"));
    assert!(output.contains("Direct message 2"));
    assert!(output.contains("Sender message 2"));
    let direct1: usize = output.find("Direct message 1").unwrap();
    let sender1: usize = output.find("Sender message 1").unwrap();
    let direct2: usize = output.find("Direct message 2").unwrap();
    let sender2: usize = output.find("Sender message 2").unwrap();
    assert!(direct1 < sender1);
    assert!(sender1 < direct2);
    assert!(direct2 < sender2);
}

#[test]
fn Message_Messenger_Test__StreamBufferBasicUsage() {
    let _guard = DEFAULT_MESSENGER_GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let output: Arc<Mutex<String>> = Arc::default();
    let original = install_test_printer(&output);
    {
        let mut buffer = Message::send_info();
        writeln!(buffer, "Test message").unwrap();
    }
    restore_printers(original);
    assert!(output.lock().unwrap().contains("Test message"));
}
