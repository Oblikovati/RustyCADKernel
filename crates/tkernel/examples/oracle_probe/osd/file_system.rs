//! OSD file_system probe operations (OSD_FileSystem, OSD_LocalFileSystem,
//! OSD_CachedFileSystem, OSD_FileSystemSelector, OSD_StreamBuffer, OSD_OpenFile); twin of
//! `tkernel_osd_file_system_probe.cxx`, which documents each request.
use super::{bit, escape, integer, token};
use rustycad_tkernel::{
    osd::{
        FileBuffer, FileSystem, FileSystemHandle, IStream, IoStreamBuffer, IosOpenMode,
        LocalFileSystem, SeekDir, SharedIStream, SharedStreamBuf, default_file_system,
        share_stream_buf,
    },
    standard::Failure,
    tcollection::AsciiString,
};
use std::{
    path::Path,
    str::SplitWhitespace,
    sync::{Arc, Mutex},
};

mod open_file_ops;
mod selector_ops;

const DIR: &str = "osdfs_tmp";

/// The output of a file_system operation, or `None` when `op` belongs to another family.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    Some(match op {
        "osd_fs_local_buffer" => local_buffer(input),
        "osd_fs_istream" => istream(input),
        "osd_fs_ostream" => ostream(input),
        "osd_fs_supported" => supported(input),
        "osd_fs_selector" => selector_ops::selector(input),
        "osd_fs_cached" => selector_ops::cached(input),
        "osd_fs_default" => selector_ops::default_protocols(),
        "osd_fs_fopen" => open_file_ops::fopen(input),
        "osd_fs_fd" => open_file_ops::descriptor(input),
        "osd_fs_ctime" => open_file_ops::ctime(input),
        "osd_fs_filebuf" => open_file_ops::filebuf(input),
        "osd_fs_fstream" => open_file_ops::fstream(input),
        _ => return None,
    })
}

fn path(index: usize) -> String {
    format!("{DIR}/f{index}.bin")
}

fn url(index: usize) -> AsciiString {
    AsciiString::from(path(index).as_str())
}

fn setup(input: &mut SplitWhitespace<'_>, count: usize) {
    let _ = std::fs::remove_dir_all(DIR);
    std::fs::create_dir(DIR).expect("probe directory");
    for index in 1..=count {
        let content: String = token(input);
        if content != "NONE" {
            std::fs::write(path(index), content).expect("probe file");
        }
    }
}

fn content(index: usize) -> String {
    std::fs::read(path(index)).map_or_else(
        |_| "NOFILE".to_string(),
        |bytes| escape(&String::from_utf8_lossy(&bytes)),
    )
}

fn finish(mut out: String, count: usize) -> String {
    for index in 1..=count {
        out += &format!(" {}", content(index));
    }
    let _ = std::fs::remove_dir_all(DIR);
    out
}

fn mode(input: &mut SplitWhitespace<'_>) -> IosOpenMode {
    IosOpenMode::from_bits(integer(input) as u32)
}

fn raise(failure: &Failure) -> String {
    let text: &str = failure.message();
    let tail: &str = text.find("': \n").map_or(text, |cut| &text[cut + 4..]);
    format!("raise {}{}", failure.exception_type(), escape(tail))
}

fn read_buffer(buffer: &SharedStreamBuf, count: usize) -> String {
    let mut bytes: Vec<u8> = vec![0; count];
    let read: usize = buffer.lock().unwrap().sgetn(&mut bytes);
    escape(&String::from_utf8_lossy(&bytes[..read]))
}

fn read_stream(stream: &mut dyn IStream, count: usize) -> String {
    let mut bytes: Vec<u8> = vec![0; count];
    let read: usize = stream.read(&mut bytes);
    format!(
        "{}{}{}",
        stream.gcount(),
        escape(&String::from_utf8_lossy(&bytes[..read])),
        stream.rdstate().bits()
    )
}

fn local_buffer(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let mode: IosOpenMode = mode(input);
    let (offset, with_size) = (integer(input), integer(input) != 0);
    let mut size: i64 = -7;
    let opened = LocalFileSystem::new().open_stream_buffer(
        &url(1),
        mode,
        offset,
        with_size.then_some(&mut size),
    );
    let out: String = match opened {
        Err(failure) => raise(&failure),
        Ok(None) => format!("null {size}"),
        Ok(Some(buffer)) => {
            let read: String = read_buffer(&buffer, 64);
            let mut guard = buffer.lock().unwrap();
            let written: usize = guard.sputn(b"WX");
            let position: i64 = guard.pubseekoff(0, SeekDir::Cur, IosOpenMode::IN);
            format!("buf {size} {read} {written} {position}")
        }
    };
    finish(out, 1)
}

fn istream(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let mode: IosOpenMode = mode(input);
    let (offset, offset2) = (integer(input), integer(input));
    let fs: LocalFileSystem = LocalFileSystem::new();
    let mut out: String = String::new();
    // The native request prints as it goes, so a raise keeps the text printed before it.
    if let Err(failure) = istream_steps(&fs, mode, offset, offset2, &mut out) {
        out += &raise(&failure);
    }
    finish(out, 1)
}

fn istream_steps(
    fs: &LocalFileSystem,
    mode: IosOpenMode,
    offset: i64,
    offset2: i64,
    out: &mut String,
) -> Result<(), Failure> {
    let url: AsciiString = url(1);
    let stream: Option<SharedIStream> = fs.open_istream(&url, mode, offset, None)?;
    *out += if stream.is_some() { "s" } else { "null" };
    if let Some(stream) = &stream {
        let read: String = read_stream(&mut *stream.lock().unwrap(), 4);
        let tell: i64 = stream.lock().unwrap().tellg();
        let state: u8 = stream.lock().unwrap().rdstate().bits();
        let open: String = format!(
            "{}{}",
            bit(fs.is_open_istream(stream)),
            bit(default_file_system().is_open_istream(stream))
        );
        *out += &format!(" {read} {tell} {state} {open}");
        let second = fs.open_istream(&url, mode, offset2, Some(stream))?;
        let same: bool = second.as_ref().is_some_and(|s| Arc::ptr_eq(s, stream));
        let text: String = second.map_or("null".to_string(), |s| {
            read_stream(&mut *s.lock().unwrap(), 2)
        });
        *out += &format!(" {} {text}", bit(same));
    }
    *out += &foreign_streams(fs);
    Ok(())
}

/// A plain file stream and an OSD_IOStreamBuffer are not OSD_IStreamBuffer streams.
fn foreign_streams(fs: &LocalFileSystem) -> String {
    let mut plain = rustycad_tkernel::osd::FileStream::input();
    plain.open(Path::new(&path(1)), IosOpenMode::NONE);
    let plain: SharedIStream = Arc::new(Mutex::new(plain));
    let mut buffer: FileBuffer = FileBuffer::new();
    buffer.open(Path::new(&path(1)), IosOpenMode::IN);
    let io: IoStreamBuffer = IoStreamBuffer::new("io-url", share_stream_buf(buffer));
    let url: String = escape(io.url());
    let io = Arc::new(Mutex::new(io));
    let as_in: SharedIStream = io.clone();
    let as_out: rustycad_tkernel::osd::SharedOStream = io;
    format!(
        " {}{}{}{url}",
        bit(fs.is_open_istream(&plain)),
        bit(fs.is_open_istream(&as_in)),
        bit(fs.is_open_ostream(&as_out))
    )
}

fn ostream(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let mode: IosOpenMode = mode(input);
    let fs: LocalFileSystem = LocalFileSystem::new();
    let stream = fs
        .open_ostream(&url(1), mode)
        .expect("OpenOStream never raises");
    let mut out: String = if stream.is_some() { "s" } else { "null" }.to_string();
    if let Some(stream) = &stream {
        let mut guard = stream.lock().unwrap();
        guard.write(b"HELLO");
        guard.flush();
        let (state, tell) = (guard.rdstate().bits(), guard.tellp());
        drop(guard);
        out += &format!(
            " {state} {tell} {}{}",
            bit(fs.is_open_ostream(stream)),
            bit(default_file_system().is_open_ostream(stream))
        );
        let mut guard = stream.lock().unwrap();
        guard.seekp(1, SeekDir::Beg);
        guard.write(b"Z");
        out += &format!(" {} {}", guard.rdstate().bits(), guard.tellp());
    }
    drop(stream);
    finish(out, 1)
}

fn supported(input: &mut SplitWhitespace<'_>) -> String {
    let url: AsciiString = AsciiString::from(token(input).as_str());
    let cached = rustycad_tkernel::osd::CachedFileSystem::new(None);
    let empty = rustycad_tkernel::osd::FileSystemSelector::new();
    let default: FileSystemHandle = default_file_system();
    format!(
        "{}{}{}{}",
        bit(LocalFileSystem::new().is_supported_path(&url)),
        bit(default.is_supported_path(&url)),
        bit(cached.is_supported_path(&url)),
        bit(empty.is_supported_path(&url))
    )
}
