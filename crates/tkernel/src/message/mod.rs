// Created on: 2001-01-06
// Created by: OCC Team
// Copyright (c) 2001-2014 OPEN CASCADE SAS
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

//! Ordered message delivery with injected printers and scoped stream buffers.
use crate::tcollection::AsciiString;
use std::rc::Rc;
mod stream;
pub use stream::MessageStream;
/// Ordered OCCT severity levels; `Gravity::Warning > Gravity::Info`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum Gravity {
    /// Low-level algorithm detail.
    Trace = 0,
    /// Informational output.
    Info = 1,
    /// Warning condition.
    Warning = 2,
    /// Nonfatal error.
    Alarm = 3,
    /// Fatal error.
    Fail = 4,
}
/// Application-owned output destination; implement `send` for a console or collector.
pub trait Printer {
    /// Minimum accepted gravity; the native default is `Gravity::Info`.
    fn trace_level(&self) -> Gravity {
        Gravity::Info
    }
    /// Receive one complete message after gravity filtering; `send(&text, Gravity::Info)`.
    fn send(&self, text: &AsciiString, gravity: Gravity);
}
/// Ordered shared printers with identity-based duplicate detection.
/// Example: `Messenger::with_printer(printer)`.
pub struct Messenger {
    printers: Vec<Rc<dyn Printer>>,
}
impl Messenger {
    /// Inject a printer instead of selecting process-global output; `Messenger::with_printer(printer)`.
    pub fn with_printer(printer: Rc<dyn Printer>) -> Self {
        Self {
            printers: vec![printer],
        }
    }
    /// Borrow the output sequence; `messenger.printers().len()`.
    pub fn printers(&self) -> &[Rc<dyn Printer>] {
        &self.printers
    }
    /// Edit the output sequence explicitly; `messenger.printers_mut().clear()`.
    pub fn printers_mut(&mut self) -> &mut Vec<Rc<dyn Printer>> {
        &mut self.printers
    }
    /// Append unless the same printer is already present; `messenger.add_printer(printer)`.
    pub fn add_printer(&mut self, printer: Rc<dyn Printer>) -> bool {
        if self
            .printers
            .iter()
            .any(|existing| Rc::ptr_eq(existing, &printer))
        {
            return false;
        }
        self.printers.push(printer);
        true
    }
    /// Remove the first printer with the same identity; `messenger.remove_printer(&printer)`.
    pub fn remove_printer(&mut self, printer: &Rc<dyn Printer>) -> bool {
        let Some(index) = self
            .printers
            .iter()
            .position(|existing| Rc::ptr_eq(existing, printer))
        else {
            return false;
        };
        self.printers.remove(index);
        true
    }
    /// Send a C-compatible string with OCCT's default Warning gravity; `messenger.send("text")`.
    pub fn send(&self, text: &str) {
        self.send_at(&AsciiString::new(text), Gravity::Warning);
    }
    /// Deliver to each eligible printer in order; `messenger.send_at(&text, Gravity::Info)`.
    pub fn send_at(&self, text: &AsciiString, gravity: Gravity) {
        for printer in &self.printers {
            if gravity >= printer.trace_level() {
                printer.send(text, gravity);
            }
        }
    }
    /// Create a buffer whose destruction sends nonempty contents; `messenger.stream(Gravity::Info)`.
    pub fn stream(&self, gravity: Gravity) -> MessageStream<'_> {
        MessageStream::new(self, gravity)
    }
}
mod lazy_progress_scope;
mod progress_indicator;
mod progress_range;
mod progress_scope;
mod progress_sentry;
pub use lazy_progress_scope::LazyProgressScope;
pub use progress_indicator::{ProgressIndicator, ProgressIndicatorHooks};
pub use progress_range::ProgressRange;
pub use progress_scope::ProgressScope;
pub use progress_sentry::ProgressSentry;
