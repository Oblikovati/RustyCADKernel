// Copyright (c) 2019 OPEN CASCADE SAS
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
