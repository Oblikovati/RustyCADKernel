//! `OSD_OpenFile.hxx` requests: fopen, descriptors, change time, filebuf and file streams.
use super::{finish, mode, path, read_buffer, read_stream, setup};
use crate::osd::{bit, escape, integer, token};
use rustycad_tkernel::{
    osd::{
        FileBuffer, FileStream, IStream, IosOpenMode, OStream, SeekDir, SharedStreamBuf,
        StreamAccess, StreamBuf, file_stat_ctime, open_file, open_file_descriptor,
        open_file_stream, open_file_stream_utf8, open_file_utf8, open_stream,
    },
    tcollection::ExtendedString,
};
use std::{
    fs::File,
    io::{Read, Seek, Write},
    str::SplitWhitespace,
    sync::{Arc, Mutex},
};

fn name() -> ExtendedString {
    ExtendedString::from_utf8(&path(1))
}

fn read_file(file: &mut File, count: usize) -> Option<Vec<u8>> {
    let mut bytes: Vec<u8> = vec![0; count];
    let mut total: usize = 0;
    while total < count {
        match file.read(&mut bytes[total..]) {
            Ok(0) => break,
            Ok(read) => total += read,
            Err(_) if total == 0 => return None,
            Err(_) => break,
        }
    }
    bytes.truncate(total);
    Some(bytes)
}

/// `osd_fs_fopen <c> <mode> <utf8>`: fread errors read as nothing, like fread's 0.
pub fn fopen(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let mode: String = token(input);
    let file: Option<File> = if integer(input) != 0 {
        open_file_utf8(&path(1), &mode)
    } else {
        open_file(&name(), &mode)
    };
    let mut out: String = if file.is_some() { "file" } else { "null" }.to_string();
    if let Some(mut file) = file {
        let bytes: Vec<u8> = read_file(&mut file, 16).unwrap_or_default();
        out += &format!(" {}", escape(&String::from_utf8_lossy(&bytes)));
        let _ = file.stream_position();
        let _ = file.write_all(b"XY");
    }
    finish(out, 1)
}

/// `osd_fs_fd <c> <mode>`.
pub fn descriptor(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let mode: IosOpenMode = mode(input);
    let file: Option<File> = open_file_descriptor(&name(), mode);
    let mut out: String = if file.is_some() { "fd" } else { "-1" }.to_string();
    if let Some(mut file) = file {
        let read: String = read_file(&mut file, 16).map_or("-1".to_string(), |bytes| {
            escape(&String::from_utf8_lossy(&bytes))
        });
        let written: i64 = file.write(b"XY").map_or(-1, |count| count as i64);
        out += &format!(" {read} {written}");
    }
    out += &format!(" {}", permissions());
    finish(out, 1)
}

#[cfg(unix)]
fn permissions() -> i64 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path(1)).map_or(-1, |m| i64::from(m.permissions().mode() & 0o777))
}

#[cfg(not(unix))]
fn permissions() -> i64 {
    -1
}

/// `osd_fs_ctime <c>`.
pub fn ctime(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let out: String = format!(
        "{} {}",
        bit(file_stat_ctime(&path(1)) > 0),
        file_stat_ctime("osdfs_tmp/missing.bin")
    );
    finish(out, 1)
}

/// `osd_fs_filebuf <c> <mode>`.
pub fn filebuf(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let mode: IosOpenMode = mode(input);
    let buffer: Arc<Mutex<FileBuffer>> = Arc::new(Mutex::new(FileBuffer::new()));
    let opened: bool = open_stream(&mut buffer.lock().unwrap(), &name(), mode);
    let mut out: String = format!("{}{}", bit(opened), bit(buffer.lock().unwrap().is_open()));
    if opened {
        let start: i64 = buffer
            .lock()
            .unwrap()
            .pubseekoff(0, SeekDir::Cur, IosOpenMode::IN);
        let shared: SharedStreamBuf = buffer.clone();
        let read: String = read_buffer(&shared, 16);
        let mut guard = buffer.lock().unwrap();
        let written: usize = guard.sputn(b"XY");
        let end: i64 = guard.pubseekoff(0, SeekDir::End, IosOpenMode::IN);
        let negative: i64 = guard.pubseekoff(-1, SeekDir::Beg, IosOpenMode::IN);
        let again: bool = open_stream(&mut guard, &name(), mode);
        out += &format!(" {start} {read} {written} {end} {negative} {}", bit(again));
    }
    drop(buffer);
    finish(out, 1)
}

fn open_kind(stream: &mut FileStream, utf8: bool, mode: IosOpenMode) -> String {
    if utf8 {
        open_file_stream_utf8(stream, &path(1), mode);
    } else {
        open_file_stream(stream, &name(), mode);
    }
    format!("{} {}", bit(stream.is_open()), stream.rdstate().bits())
}

/// `osd_fs_fstream <c> <kind> <mode> <utf8>`.
pub fn fstream(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 1);
    let kind: String = token(input);
    let mode: IosOpenMode = mode(input);
    let utf8: bool = integer(input) != 0;
    let out: String = match kind.as_str() {
        "i" => {
            let mut stream: FileStream = FileStream::input();
            let mut out: String = open_kind(&mut stream, utf8, mode);
            out += &format!(" {}", read_stream(&mut stream, 16));
            stream.close();
            out += &format!(" {}", stream.rdstate().bits());
            stream.close();
            out + &format!(" {}", stream.rdstate().bits())
        }
        "o" => {
            let mut stream: FileStream = FileStream::output();
            let out: String = open_kind(&mut stream, utf8, mode);
            OStream::write(&mut stream, b"XY");
            format!("{out} {} {}", stream.rdstate().bits(), stream.tellp())
        }
        _ => {
            let mut stream: FileStream = FileStream::input_output();
            let mut out: String = open_kind(&mut stream, utf8, mode);
            out += &format!(" {}", read_stream(&mut stream, 16));
            stream.clear();
            stream.seekp(0, SeekDir::End);
            OStream::write(&mut stream, b"XY");
            format!(
                "{out} {} {}",
                stream.rdstate().bits(),
                IStream::tellg(&mut stream)
            )
        }
    };
    finish(out, 1)
}
