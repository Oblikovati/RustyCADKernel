//! Encoding, capture and printing helpers (twin of `tkernel_units_probe_common.cxx`).
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::resource::{ResourceConsole, ResourceEnvironment, ResourceHost, ResourceIo};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::units::{Dimensions, ShiftedToken, Token, UnitsDiagnostics};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::str::SplitWhitespace;
use std::sync::{Arc, Mutex};

/// Records console text and messages the way the native capture does.
#[derive(Clone, Default)]
pub struct Capture {
    console: Arc<Mutex<Vec<u8>>>,
    messages: Arc<Mutex<Vec<u8>>>,
}

impl Capture {
    pub fn clear(&self) {
        self.console.lock().unwrap().clear();
        self.messages.lock().unwrap().clear();
    }
    pub fn console(&self) -> Vec<u8> {
        self.console.lock().unwrap().clone()
    }
    pub fn messages(&self) -> Vec<u8> {
        self.messages.lock().unwrap().clone()
    }
    /// A writer appending to the console text (`std::cout`).
    pub fn writer(&self) -> ConsoleWriter {
        ConsoleWriter(self.console.clone())
    }
    fn message(&self, gravity: &[u8], text: &[u8]) {
        let mut messages = self.messages.lock().unwrap();
        messages.extend_from_slice(gravity);
        messages.extend_from_slice(text);
        messages.push(b'\n');
    }
}

impl UnitsDiagnostics for Capture {
    fn print(&self, text: &[u8]) {
        self.console.lock().unwrap().extend_from_slice(text);
    }
    fn warning(&self, text: &[u8]) {
        self.message(b"W:", text);
    }
    fn info(&self, text: &[u8]) {
        self.message(b"I:", text);
    }
}

/// Resource traces are native `std::cout` text.
impl ResourceConsole for Capture {
    fn write(&self, bytes: &[u8]) {
        self.console.lock().unwrap().extend_from_slice(bytes);
    }
}

/// The resource host of both probes: no variables, no files, the captured console.
pub fn resource_host(capture: &Capture) -> ResourceHost {
    let console = std::sync::Arc::new(capture.clone());
    ResourceHost::new(
        std::sync::Arc::new(NoResourceIo),
        std::sync::Arc::new(EmptyEnvironment),
        console,
    )
}

/// `std::cout` of the probe.
pub struct ConsoleWriter(Arc<Mutex<Vec<u8>>>);

impl Write for ConsoleWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// The native probe unsets every `CSF_*` resource variable it could read.
pub struct EmptyEnvironment;
impl ResourceEnvironment for EmptyEnvironment {
    fn directory(&self, _name: &str) -> Option<PathBuf> {
        None
    }
}

/// Without resource variables no file is ever read or written.
pub struct NoResourceIo;
impl ResourceIo for NoResourceIo {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        panic!("unexpected resource read of {path:?}");
    }
    fn create_directories(&self, path: &Path) -> io::Result<()> {
        panic!("unexpected resource directory {path:?}");
    }
    fn open_write(&self, path: &Path) -> io::Result<Box<dyn Write>> {
        panic!("unexpected resource write to {path:?}");
    }
}

pub fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn unhex(text: &str) -> Vec<u8> {
    if text == "-" {
        return Vec::new();
    }
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}

pub fn arg(input: &mut SplitWhitespace<'_>) -> Vec<u8> {
    unhex(input.next().expect("hex field"))
}

pub fn real(input: &mut SplitWhitespace<'_>) -> f64 {
    input
        .next()
        .expect("real field")
        .parse()
        .expect("real input")
}

pub fn word(input: &mut SplitWhitespace<'_>) -> String {
    input.next().expect("word field").to_string()
}

pub fn bits(value: f64) -> u64 {
    value.to_bits()
}

pub fn flag(value: bool) -> u8 {
    u8::from(value)
}

/// Hides the address after "Units_Token::Dump of ".
pub fn mask(text: &[u8]) -> Vec<u8> {
    const MARK: &[u8] = b"Units_Token::Dump of ";
    let mut out = Vec::with_capacity(text.len());
    let mut rest = text;
    while let Some(position) = rest.windows(MARK.len()).position(|w| w == MARK) {
        out.extend_from_slice(&rest[..position + MARK.len()]);
        out.push(b'@');
        rest = &rest[position + MARK.len()..];
        let end = rest.iter().position(|b| *b == b'\n').unwrap_or(rest.len());
        rest = &rest[end..];
    }
    out.extend_from_slice(rest);
    out
}

pub fn dims(dimensions: &Handle<Dimensions>) -> String {
    let Some(dimensions) = dimensions.get() else {
        return "null".into();
    };
    let parts: Vec<String> = dimensions
        .exponents()
        .iter()
        .map(|v| bits(*v).to_string())
        .collect();
    parts.join(",")
}

pub fn token(token: &Handle<Token>) -> String {
    let Some(object) = token.get() else {
        return "null".into();
    };
    let shifted = token.cast::<ShiftedToken>();
    let mut text = format!(
        "{} {} {} {} {}",
        if shifted.is_some() { 'S' } else { 'P' },
        hex(object.word().as_bytes()),
        hex(object.mean().as_bytes()),
        bits(object.value()),
        dims(&object.dimensions())
    );
    if let Some(shifted) = shifted.get() {
        text.push_str(&format!(" {}", bits(shifted.movement())));
    }
    text
}

pub fn tokens(sequence: &Handle<HSequence<Handle<Token>>>) -> Result<String, Failure> {
    let Some(items) = sequence.get() else {
        return Ok("nullseq".into());
    };
    Ok(items.sequence(|values| {
        let mut text = values.len().to_string();
        for value in values.iter() {
            text.push_str(&format!(" [{}]", token(value)));
        }
        text
    }))
}
