//! Delivery identities, filtering, copied streams and exact empty-flush behavior.
use rustycad_tkernel::message::{Gravity, Messenger, Printer};
use rustycad_tkernel::tcollection::AsciiString;
use std::{cell::RefCell, fmt::Write, rc::Rc};
#[derive(Default)]
struct RecordingPrinter {
    messages: RefCell<Vec<(Vec<u8>, Gravity)>>,
}
impl Printer for RecordingPrinter {
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        self.messages
            .borrow_mut()
            .push((text.as_bytes().to_vec(), gravity));
    }
}
#[test]
fn printer_identity_and_gravity_filtering() {
    let printer: Rc<RecordingPrinter> = Rc::default();
    let erased: Rc<dyn Printer> = printer.clone();
    let mut messenger = Messenger::with_printer(erased.clone());
    assert!(!messenger.add_printer(erased.clone()));
    messenger.send_at(&AsciiString::new("trace"), Gravity::Trace);
    for gravity in [
        Gravity::Info,
        Gravity::Warning,
        Gravity::Alarm,
        Gravity::Fail,
    ] {
        messenger.send_at(&AsciiString::new("kept"), gravity);
    }
    assert_eq!(printer.messages.borrow().len(), 4);
    assert!(messenger.remove_printer(&erased));
    assert!(!messenger.remove_printer(&erased));
    messenger.send("absent");
    assert_eq!(printer.messages.borrow().len(), 4);
}
#[test]
fn empty_forced_flush_and_formal_copy_contents() {
    let printer: Rc<RecordingPrinter> = Rc::default();
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
        printer.messages.borrow().as_slice(),
        [
            (Vec::new(), Gravity::Info),
            (b"copy".to_vec(), Gravity::Info),
            (b"original".to_vec(), Gravity::Info),
        ]
    );
}
