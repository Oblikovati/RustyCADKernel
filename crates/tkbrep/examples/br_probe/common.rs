//! Shared printers and readers, the Rust twin of tools/oracle/tkbrep_probe_common.cxx.
pub use super::print::{Reader, attempt, end, flag, int, real, show};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools::ConsoleOutput;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

pub fn word(text: &str) {
    print!("{text} ");
}
pub fn long(value: u64) {
    print!("{value} ");
}
pub fn failure(error: &Failure) {
    print!("{} ", error.exception_type());
}
/// `br_text`: a text as one `t<hex bytes>` token.
pub fn text(bytes: &[u8]) {
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    print!("t{hex} ");
}
/// `br_masked`: every `0x` pointer text becomes `0xP`.
pub fn masked(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        if !text[i..].starts_with(b"0x") {
            out.push(text[i]);
            i += 1;
            continue;
        }
        out.extend_from_slice(b"0xP");
        i += 2;
        while i < text.len() && (text[i].is_ascii_hexdigit() || text[i] == b'x') {
            i += 1;
        }
    }
    out
}

impl Reader<'_> {
    /// `count byte...`: a text given as byte codes.
    pub fn bytes(&mut self) -> Vec<u8> {
        let n = self.integer();
        (0..n).map(|_| self.integer() as u8).collect()
    }
}

/// The console the readers write to, captured as the native probe captures `std::cout`.
#[derive(Default)]
pub struct CapturedConsole {
    bytes: Mutex<Vec<u8>>,
}
impl ConsoleOutput for CapturedConsole {
    fn write(&self, bytes: &[u8]) {
        self.bytes.lock().unwrap().extend_from_slice(bytes);
    }
}
impl CapturedConsole {
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::default())
    }
    pub fn taken(&self) -> Vec<u8> {
        std::mem::take(&mut self.bytes.lock().unwrap())
    }
}

/// A writable in-memory stream of the given precision (`std::ostringstream`).
pub fn out_stream(precision: usize) -> IoStream {
    let mut stream = IoStream::from_bytes(Vec::new());
    stream.set_precision(precision);
    stream
}

/// The bytes written to an in-memory stream.
pub fn written_bytes(stream: IoStream) -> Vec<u8> {
    let mut device = stream.into_device().expect("an open stream");
    device.seek(SeekFrom::Start(0)).expect("in-memory seek");
    let mut bytes = Vec::new();
    device.read_to_end(&mut bytes).expect("in-memory read");
    bytes
}

/// The unread rest of a stream after clearing its state.
pub fn rest(stream: &mut IoStream) -> Vec<u8> {
    stream.clear(0);
    let mut rest = Vec::new();
    let mut byte = 0;
    loop {
        stream.get(&mut byte);
        if stream.fail() {
            return rest;
        }
        rest.push(byte);
    }
}
