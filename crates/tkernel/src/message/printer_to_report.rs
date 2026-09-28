// Copyright (c) 2020 OPEN CASCADE SAS
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

//! `Message_PrinterToReport`: turns messenger output into alerts of a report.
use super::{
    AlertExtended, AttributeMeter, AttributeObject, AttributeStream, BaseAttribute, Gravity,
    Message, Printer, PrinterBase, PrinterType, Report,
};
use crate::standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient};
use crate::tcollection::AsciiString;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

/// Printer adding one extended alert per message; `PrinterToReport::new()`.
///
/// A stream without a `": "` key/value separator is held as the name of the next alert,
/// as `Standard_Dump` output arrives as a name line followed by a JSON line.
#[derive(Default)]
pub struct PrinterToReport {
    base: PrinterBase,
    name: Mutex<AsciiString>,
    report: Mutex<Option<Arc<Report>>>,
}
impl RuntimeTyped for PrinterToReport {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimeType::new::<Self>("Message_PrinterToReport", Some(PrinterType::static_type()))
        })
    }
}
fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
impl PrinterToReport {
    /// A printer targeting the default report; `PrinterToReport::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The target report, else the default one (created on demand); `printer.report()`.
    pub fn report(&self) -> Arc<Report> {
        locked(&self.report)
            .clone()
            .unwrap_or_else(|| Message::default_report(true).expect("created on request"))
    }
    /// Sets the target report (`None` targets the default one); `printer.set_report(Some(report))`.
    pub fn set_report(&self, report: Option<Arc<Report>>) {
        *locked(&self.report) = report;
    }
    fn take_name(&self) -> AsciiString {
        std::mem::take(&mut *locked(&self.name))
    }
    /// `sendMetricAlert`: an alert holding a meter named by the value.
    fn send_metric_alert(&self, value: &AsciiString, gravity: Gravity) {
        let meter = Arc::new(AttributeMeter::new(value));
        AlertExtended::add_alert(&self.report(), Some(meter), gravity);
    }
}
impl Printer for PrinterToReport {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// Flushes a pending name first, then adds a meter or a plain attribute alert.
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        let pending = self.take_name();
        if !pending.is_empty() {
            self.send(&pending, gravity);
        }
        let report = self.report();
        if !report.active_metrics().is_empty() {
            self.send_metric_alert(text, gravity);
            return;
        }
        AlertExtended::add_alert(&report, Some(Arc::new(BaseAttribute::new(text))), gravity);
    }
    /// Ignores the trace level (native override): JSON text becomes a stream alert named by
    /// the pending name, other text becomes the pending name.
    fn send_string_stream(&self, stream: &[u8], gravity: Gravity) {
        let report = self.report();
        let text = AsciiString::from_bytes(stream);
        if !report.active_metrics().is_empty() {
            self.send_metric_alert(&text, gravity);
            return;
        }
        if text.search(": ") >= 0 {
            let name = self.take_name();
            AlertExtended::add_alert(
                &report,
                Some(Arc::new(AttributeStream::new(stream, &name))),
                gravity,
            );
            return;
        }
        let pending = self.take_name();
        if !pending.is_empty() {
            self.send(&pending, gravity);
        }
        *locked(&self.name) = text;
    }
    /// Ignores the trace level and null objects (native override); the pending name is
    /// used but kept.
    fn send_object(&self, object: &Handle<StandardTransient>, gravity: Gravity) {
        let report = self.report();
        let name = locked(&self.name).clone();
        if !report.active_metrics().is_empty() {
            self.send_metric_alert(&name, gravity);
            return;
        }
        let attribute = Arc::new(AttributeObject::new(object.clone(), &name));
        AlertExtended::add_alert(&report, Some(attribute), gravity);
    }
}
