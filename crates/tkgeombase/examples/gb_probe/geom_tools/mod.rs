//! The GeomTools probe cases, the Rust twin of tools/oracle/tkgeombase_probe_geomtools.cxx
//! (texts and single-record reads) and tkgeombase_probe_geomtools_sets.cxx (sets.rs).
pub mod build;
pub mod console;
pub mod kinds;
pub mod sets;

use super::common::{Reader, end, int, real};
use console::{attempt_captured, captured, install_default_handler};
use kinds::{Curves, Curves2d, GtKind, Surfaces, summary};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkgeombase::geom_tools;
use std::io::{Read, Seek, SeekFrom};

/// `gt_text`: one `t<hex>` token.
pub fn text(bytes: &[u8]) {
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    print!("t{hex} ");
}

/// The bytes written to an in-memory stream.
pub fn written_bytes(stream: IoStream) -> Vec<u8> {
    let mut device = stream.into_device().expect("an open stream");
    device.seek(SeekFrom::Start(0)).expect("in-memory seek");
    let mut bytes = Vec::new();
    device.read_to_end(&mut bytes).expect("in-memory read");
    bytes
}

/// `gt_bytes`: a text given as byte codes.
pub fn bytes(r: &mut Reader<'_>) -> Vec<u8> {
    let n = r.integer();
    (0..n).map(|_| r.integer() as u8).collect()
}

/// `gt_state`: the state after a read, then the unread rest.
pub fn state(stream: &mut IoStream) {
    int(i32::from(stream.rdstate()));
    stream.clear(0);
    let mut rest = Vec::new();
    let mut byte = 0;
    loop {
        stream.get(&mut byte);
        if stream.fail() {
            break;
        }
        rest.push(byte);
    }
    text(&rest);
}

/// `gt_written`: the text `print` writes at `precision`, and its console output.
pub fn written(
    precision: usize,
    print: impl FnOnce(&mut IoStream) -> Result<(), Failure>,
) -> Vec<u8> {
    let mut stream = IoStream::from_bytes(Vec::new());
    stream.set_precision(precision);
    let console = captured(|| attempt_captured(|| print(&mut stream)));
    let bytes = written_bytes(stream);
    text(&bytes);
    text(&console);
    bytes
}

/// `gt_texts`: Dump and Write at 6 and 17, then the compact text read back.
pub fn texts<K: GtKind>(g: &Handle<K::Geometry>) {
    written(6, |s| K::dump(g, s));
    written(6, |s| K::write(g, s));
    written(17, |s| K::dump(g, s));
    let compact = written(17, |s| K::write(g, s));
    if compact.is_empty() {
        return;
    }
    let mut stream = IoStream::from_bytes(compact);
    let mut back = Handle::default();
    text(&captured(|| back = K::read(&mut stream)));
    summary::<K>(&back);
    state(&mut stream);
}

fn geometry_case<K: GtKind>(values: &[f64]) {
    install_default_handler();
    let mut r = Reader::new(values);
    match K::build(&mut r) {
        Ok(g) if g.is_some() => texts::<K>(&g),
        Ok(_) => {}
        Err(error) => print!("{} ", error.exception_type()),
    }
    end();
}

pub fn curve_case(values: &[f64]) {
    geometry_case::<Curves>(values);
}
pub fn curve2d_case(values: &[f64]) {
    geometry_case::<Curves2d>(values);
}
pub fn surface_case(values: &[f64]) {
    geometry_case::<Surfaces>(values);
}

fn read_one<K: GtKind>(stream: &mut IoStream) -> Vec<u8> {
    let mut g = Handle::default();
    let console = captured(|| g = K::read(stream));
    summary::<K>(&g);
    console
}

/// `gt_read kind bytes`.
pub fn read_case(values: &[f64]) {
    install_default_handler();
    let mut r = Reader::new(values);
    let kind = r.integer();
    let mut stream = IoStream::from_bytes(bytes(&mut r));
    let console = match kind {
        0 => read_one::<Curves>(&mut stream),
        1 => read_one::<Curves2d>(&mut stream),
        _ => read_one::<Surfaces>(&mut stream),
    };
    text(&console);
    state(&mut stream);
    end();
}

/// `gt_get_real count bytes`.
pub fn get_real_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let mut stream = IoStream::from_bytes(bytes(&mut r));
    for _ in 0..n {
        real(geom_tools::get_real(&mut stream));
        int(i32::from(stream.rdstate()));
    }
    state(&mut stream);
    end();
}
