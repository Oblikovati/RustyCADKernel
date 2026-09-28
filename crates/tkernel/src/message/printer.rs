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

//! `Message_Printer`: an output context for text messages filtered by gravity.
use super::Gravity;
use crate::standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient};
use crate::tcollection::{AsciiString, ExtendedString};
use std::any::Any;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicI32, Ordering};

/// Shared printer handle, the Rust `occ::handle<Message_Printer>`; printers are never null.
pub type PrinterHandle = std::sync::Arc<dyn Printer>;

/// State every printer embeds: the trace level (`myTraceLevel`), default `Gravity::Info`.
#[derive(Debug)]
pub struct PrinterBase {
    trace_level: AtomicI32,
}
impl Default for PrinterBase {
    fn default() -> Self {
        Self::new(Gravity::Info)
    }
}
impl PrinterBase {
    /// Creates the state with a trace level; `PrinterBase::new(Gravity::Info)`.
    pub fn new(trace_level: Gravity) -> Self {
        Self {
            trace_level: AtomicI32::new(trace_level as i32),
        }
    }
    /// Messages with a lower gravity are ignored; `base.trace_level()`.
    pub fn trace_level(&self) -> Gravity {
        Gravity::ALL[self.trace_level.load(Ordering::Relaxed) as usize]
    }
    /// Changes the filter threshold; `base.set_trace_level(Gravity::Trace)`.
    pub fn set_trace_level(&self, level: Gravity) {
        self.trace_level.store(level as i32, Ordering::Relaxed);
    }
}

/// Marker type carrying the `Message_Printer` runtime descriptor.
pub struct PrinterType;
impl RuntimeTyped for PrinterType {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimeType::new::<Self>("Message_Printer", Some(StandardTransient::static_type()))
        })
    }
}

/// A message output; implement `send` (the native protected pure virtual) and embed a
/// [`PrinterBase`]; the public `send_*` overloads filter by trace level first.
///
/// ```
/// use rustycad_tkernel::message::{Gravity, Printer, PrinterBase, PrinterType};
/// use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
/// use rustycad_tkernel::tcollection::AsciiString;
/// struct Silent(PrinterBase);
/// impl Printer for Silent {
///     fn printer_base(&self) -> &PrinterBase { &self.0 }
///     fn dynamic_type(&self) -> &'static RuntimeType { PrinterType::static_type() }
///     fn send(&self, _: &AsciiString, _: Gravity) {}
/// }
/// Silent(PrinterBase::default()).send_c_str(b"ignored", Gravity::Fail);
/// ```
pub trait Printer: Any + Send + Sync {
    /// The embedded trace-level state; `printer.printer_base()`.
    fn printer_base(&self) -> &PrinterBase;
    /// The most-derived descriptor, used by `Messenger::remove_printers`; `printer.dynamic_type()`.
    fn dynamic_type(&self) -> &'static RuntimeType;
    /// Receives one message that passed the filter (native protected `send`).
    fn send(&self, text: &AsciiString, gravity: Gravity);

    /// `GetTraceLevel`; `printer.trace_level()`.
    fn trace_level(&self) -> Gravity {
        self.printer_base().trace_level()
    }
    /// `SetTraceLevel`; `printer.set_trace_level(Gravity::Trace)`.
    fn set_trace_level(&self, level: Gravity) {
        self.printer_base().set_trace_level(level);
    }
    /// `Send(const TCollection_ExtendedString&, ...)`: converts to UTF-8; `printer.send_extended(&text, g)`.
    fn send_extended(&self, text: &ExtendedString, gravity: Gravity) {
        if gravity >= self.trace_level() {
            self.send(&AsciiString::from(text), gravity);
        }
    }
    /// `Send(const char*, ...)`: bytes up to the first NUL; `printer.send_c_str(b"text", g)`.
    fn send_c_str(&self, text: &[u8], gravity: Gravity) {
        if gravity >= self.trace_level() {
            self.send(&AsciiString::from_bytes(text), gravity);
        }
    }
    /// `Send(const TCollection_AsciiString&, ...)`; `printer.send_ascii(&text, g)`.
    fn send_ascii(&self, text: &AsciiString, gravity: Gravity) {
        if gravity >= self.trace_level() {
            self.send(text, gravity);
        }
    }
    /// `SendStringStream`: the stream's C string (cut at a NUL); `printer.send_string_stream(b"x", g)`.
    fn send_string_stream(&self, stream: &[u8], gravity: Gravity) {
        if gravity >= self.trace_level() {
            self.send(&AsciiString::from_bytes(stream), gravity);
        }
    }
    /// `SendObject`: `"<type name>: <pointer>"` for a non-null object; `printer.send_object(&object, g)`.
    fn send_object(&self, object: &Handle<StandardTransient>, gravity: Gravity) {
        if object.is_some() && gravity >= self.trace_level() {
            self.send(&object_description(object), gravity);
        }
    }
}

/// `DynamicType()->Name() + ": " + Standard_Dump::GetPointerInfo(object)`.
pub(crate) fn object_description(object: &Handle<StandardTransient>) -> AsciiString {
    let name = object.dynamic_type().map_or("", RuntimeType::name);
    let mut text = AsciiString::new(name);
    text.assign_cat(": ");
    text.assign_cat(super::json::pointer_info(object.as_ptr().cast()).as_str());
    text
}
