// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! OCC31189 and the complete legacy stream-buffer suite.
#![allow(non_snake_case)]
use rustycad_tkernel::message::{Gravity, Messenger, Printer};
use rustycad_tkernel::tcollection::AsciiString;
use std::{cell::RefCell, fmt::Write, rc::Rc};
struct TestMessagePrinter {
    output: Rc<RefCell<String>>,
}
impl Printer for TestMessagePrinter {
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        if gravity >= Gravity::Info {
            self.output
                .borrow_mut()
                .push_str(&String::from_utf8_lossy(text.as_bytes()));
            self.output.borrow_mut().push('\n');
        }
    }
}
fn fixture() -> (Messenger, Rc<RefCell<String>>) {
    let output: Rc<RefCell<String>> = Rc::default();
    let printer: Rc<dyn Printer> = Rc::new(TestMessagePrinter {
        output: output.clone(),
    });
    (Messenger::with_printer(printer), output)
}
#[test]
fn Message_Messenger_Test__OCC31189_StreamBufferMessageOrdering() {
    let (mut messenger, output) = fixture();
    let original = messenger.printers().to_vec();
    let printer = original[0].clone();
    messenger.printers_mut().clear();
    messenger.add_printer(printer);
    {
        let mut sender = messenger.stream(Gravity::Info);
        write!(sender, "Sender message 1: start ...").unwrap();
        messenger.send("Direct message 1");
        write!(sender, "... end").unwrap();
        sender.flush(true);
        drop(messenger.stream(Gravity::Info));
        messenger.send("Direct message 2");
        let mut empty = messenger.stream(Gravity::Info);
        empty.flush(true);
        write!(sender, "Sender message 2").unwrap();
    }
    messenger.printers_mut().clear();
    messenger.printers_mut().extend(original);
    check_message_order(&output.borrow());
}
fn check_message_order(output: &str) {
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
    let (mut messenger, output) = fixture();
    let original = messenger.printers().to_vec();
    let printer = original[0].clone();
    messenger.printers_mut().clear();
    messenger.add_printer(printer);
    {
        let mut buffer = messenger.stream(Gravity::Info);
        writeln!(buffer, "Test message").unwrap();
    }
    messenger.printers_mut().clear();
    messenger.printers_mut().extend(original);
    assert!(output.borrow().contains("Test message"));
}
