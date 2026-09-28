//! Named fakes for the Message behavior tests.
// Shared by two test binaries, each of which uses only some of the fakes.
#![allow(dead_code)]
use rustycad_tkernel::message::{
    Gravity, MetricSampler, MsgFileSource, Printer, PrinterBase, PrinterType,
};
use rustycad_tkernel::osd::MemCounter;
use rustycad_tkernel::standard::{RuntimeType, RuntimeTyped};
use rustycad_tkernel::tcollection::AsciiString;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Records every delivered message with its gravity.
#[derive(Default)]
pub struct RecordingPrinter {
    base: PrinterBase,
    pub messages: Mutex<Vec<(String, Gravity)>>,
}
impl Printer for RecordingPrinter {
    fn printer_base(&self) -> &PrinterBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        PrinterType::static_type()
    }
    fn send(&self, text: &AsciiString, gravity: Gravity) {
        let text = String::from_utf8_lossy(text.as_bytes()).into_owned();
        self.messages.lock().unwrap().push((text, gravity));
    }
}
impl RecordingPrinter {
    /// The texts received so far.
    pub fn texts(&self) -> Vec<String> {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .map(|(text, _)| text.clone())
            .collect()
    }
}

/// In-memory files and environment for `MsgFile`.
#[derive(Default)]
pub struct FakeMsgFileSource {
    pub files: HashMap<String, Vec<u8>>,
    pub variables: HashMap<String, String>,
}
impl MsgFileSource for FakeMsgFileSource {
    fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files.get(path).cloned()
    }
    fn variable(&self, name: &str) -> Option<String> {
        self.variables.get(name).cloned()
    }
}

/// Deterministic metric values: every sample returns the next value of a counter.
#[derive(Default)]
pub struct FakeMetricSampler {
    next: Mutex<f64>,
}
impl FakeMetricSampler {
    fn tick(&self) -> f64 {
        let mut next = self.next.lock().unwrap();
        *next += 1.0;
        *next
    }
}
impl MetricSampler for FakeMetricSampler {
    fn wall_clock_time(&self) -> f64 {
        self.tick()
    }
    fn process_cpu(&self) -> (f64, f64) {
        (self.tick(), 0.5)
    }
    fn thread_cpu(&self) -> (f64, f64) {
        (self.tick(), 0.0)
    }
    fn memory_mib(&self, counters: &[MemCounter]) -> Vec<f64> {
        counters
            .iter()
            .map(|counter| {
                if *counter == MemCounter::SwapUsage {
                    -1.0
                } else {
                    2.0
                }
            })
            .collect()
    }
}

/// A `Write` destination whose bytes stay readable after the writer is moved away.
#[derive(Clone, Default)]
pub struct SharedBuffer(pub Arc<Mutex<Vec<u8>>>);
impl Write for SharedBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl SharedBuffer {
    /// Everything written so far.
    pub fn contents(&self) -> Vec<u8> {
        self.0.lock().unwrap().clone()
    }
}
