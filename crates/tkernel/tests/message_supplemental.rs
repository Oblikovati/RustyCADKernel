//! Delivery identities, filtering, copied streams and exact empty-flush behavior.
use rustycad_tkernel::message::{
    Gravity, Messenger, Printer, PrinterBase, PrinterHandle, PrinterType,
};
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

/// Printer recording every delivered message with its gravity.
#[derive(Default)]
struct RecordingPrinter {
    base: PrinterBase,
    messages: Mutex<Vec<(Vec<u8>, Gravity)>>,
}
impl Printer for RecordingPrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        self.messages
            .lock()
            .unwrap()
            .push((text.as_bytes().to_vec(), gravity));
    }
}
#[test]
fn printer_identity_and_gravity_filtering() {
    let printer: Arc<RecordingPrinter> = Arc::default();
    let erased: PrinterHandle = printer.clone();
    let messenger = Messenger::with_printer(erased.clone());
    assert!(!messenger.add_printer(erased.clone()));
    messenger.send_ascii(&AsciiString::new("trace"), Gravity::Trace);
    for gravity in [
        Gravity::Info,
        Gravity::Warning,
        Gravity::Alarm,
        Gravity::Fail,
    ] {
        messenger.send_ascii(&AsciiString::new("kept"), gravity);
    }
    assert_eq!(printer.messages.lock().unwrap().len(), 4);
    assert!(messenger.remove_printer(&erased));
    assert!(!messenger.remove_printer(&erased));
    messenger.send("absent");
    assert_eq!(printer.messages.lock().unwrap().len(), 4);
}
#[test]
fn empty_forced_flush_and_formal_copy_contents() {
    let printer: Arc<RecordingPrinter> = Arc::default();
    let messenger = Messenger::with_printer(printer.clone());
    let mut stream = messenger.stream(Gravity::Info);
    assert!(std::ptr::eq(stream.messenger(), &messenger));
    stream.flush(false);
    stream.flush(true);
    stream.buffer_mut().push_str("original");
    let mut copied = stream.clone();
    write!(copied, "copy").unwrap();
    copied.flush(false);
    drop(copied);
    write!(stream, "\0not delivered").unwrap();
    drop(stream);
    assert_eq!(
        printer.messages.lock().unwrap().as_slice(),
        [
            (Vec::new(), Gravity::Info),
            (b"copy".to_vec(), Gravity::Info),
            (b"original".to_vec(), Gravity::Info),
        ]
    );
}
