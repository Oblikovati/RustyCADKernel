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

//! `Message_PrinterOStream`: prints to standard output, standard error or a file.
use super::{ConsoleColor, Gravity, Printer, PrinterBase, PrinterType};
use crate::standard::{RuntimeType, RuntimeTyped};
use crate::tcollection::AsciiString;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
mod console;

/// The destination of a stream printer; `None` once `close` ran.
struct Output {
    writer: Option<Box<dyn Write + Send>>,
    is_file: bool,
}

/// Printer writing one line per message, colorized on consoles; `PrinterOStream::new(Gravity::Info)`.
pub struct PrinterOStream {
    base: PrinterBase,
    output: Mutex<Output>,
    to_colorize: AtomicBool,
}
message_runtime_type!(PrinterOStream, "Message_PrinterOStream", PrinterType);
impl PrinterOStream {
    /// Prints to standard output with colors; `PrinterOStream::new(Gravity::Info)`.
    pub fn new(trace_level: Gravity) -> Self {
        Self::with_writer(Box::new(io::stdout()), false, trace_level)
    }
    /// Prints to an injected writer; a file writer is never colorized;
    /// `PrinterOStream::with_writer(Box::new(Vec::new()), true, Gravity::Info)`.
    pub fn with_writer(writer: Box<dyn Write + Send>, is_file: bool, trace_level: Gravity) -> Self {
        Self {
            base: PrinterBase::new(trace_level),
            output: Mutex::new(Output {
                writer: Some(writer),
                is_file,
            }),
            to_colorize: AtomicBool::new(!is_file),
        }
    }
    /// `cerr`/`cout` (any case) select a console stream; any other name opens a file,
    /// appending or truncating, and falls back to standard output when that fails;
    /// `PrinterOStream::from_file_name("log.txt", false, Gravity::Info)`.
    pub fn from_file_name(file_name: &str, to_append: bool, trace_level: Gravity) -> Self {
        if file_name.eq_ignore_ascii_case("cerr") {
            return Self::with_writer(Box::new(io::stderr()), false, trace_level);
        }
        if file_name.eq_ignore_ascii_case("cout") {
            return Self::new(trace_level);
        }
        let path = if cfg!(windows) {
            file_name.replace('/', "\\")
        } else {
            file_name.to_owned()
        };
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(to_append)
            .truncate(!to_append)
            .open(path);
        match file {
            Ok(file) => Self::with_writer(Box::new(io::BufWriter::new(file)), true, trace_level),
            Err(_) => Self::new(trace_level),
        }
    }
    fn output(&self) -> std::sync::MutexGuard<'_, Output> {
        self.output
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// Flushes and releases the destination; later messages are dropped; `printer.close()`.
    pub fn close(&self) {
        let mut output = self.output();
        if let Some(mut writer) = output.writer.take() {
            // `std::ostream::flush` reports failures through state bits nobody reads.
            let _ = writer.flush();
        }
        output.is_file = false;
    }
    /// Runs `action` on the destination (`GetStream`), `None` after `close`;
    /// `printer.with_stream(|stream| stream.is_some())`.
    pub fn with_stream<R>(
        &self,
        action: impl FnOnce(Option<&mut (dyn Write + Send + 'static)>) -> R,
    ) -> R {
        let mut output = self.output();
        action(output.writer.as_deref_mut())
    }
    /// Whether console output is colorized; `printer.to_colorize()`.
    pub fn to_colorize(&self) -> bool {
        self.to_colorize.load(Ordering::Relaxed)
    }
    /// Enables or disables console colors; `printer.set_to_colorize(false)`.
    pub fn set_to_colorize(&self, to_colorize: bool) {
        self.to_colorize.store(to_colorize, Ordering::Relaxed);
    }
    /// Emits the color switch for a console stream; `PrinterOStream::set_console_text_color(Some(&mut out), ConsoleColor::Red, true)`.
    pub fn set_console_text_color(
        stream: Option<&mut dyn Write>,
        color: ConsoleColor,
        is_intense: bool,
    ) {
        console::set_text_color(stream, color, is_intense);
    }
}

/// Console color of a gravity when colorizing: `(color, intense)`.
fn gravity_color(gravity: Gravity) -> (ConsoleColor, bool) {
    match gravity {
        Gravity::Trace => (ConsoleColor::Yellow, false),
        Gravity::Info => (ConsoleColor::Green, true),
        Gravity::Warning => (ConsoleColor::Yellow, true),
        Gravity::Alarm | Gravity::Fail => (ConsoleColor::Red, true),
    }
}

impl Printer for PrinterOStream {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        let mut output = self.output();
        let colorize = self.to_colorize() && !output.is_file;
        let Some(writer) = output.writer.as_deref_mut() else {
            return;
        };
        if gravity < self.trace_level() {
            return;
        }
        let (color, intense) = if colorize {
            gravity_color(gravity)
        } else {
            (ConsoleColor::Default, false)
        };
        // `std::ostream <<` failures only set state bits; the native printer never checks them.
        if intense || color != ConsoleColor::Default {
            console::set_text_color(Some(writer), color, intense);
            let _ = writer.write_all(text.as_bytes());
            console::set_text_color(Some(writer), ConsoleColor::Default, false);
        } else {
            let _ = writer.write_all(text.as_bytes());
        }
        let _ = writer.write_all(b"\n");
    }
}
impl Drop for PrinterOStream {
    fn drop(&mut self) {
        self.close();
    }
}
