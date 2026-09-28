//! Selector, cached and default file-system requests, with the logging fake protocol
//! `FspFake` of the native probe.
use super::{finish, raise, read_buffer, read_stream, setup, url};
use crate::osd::{bit, token};
use rustycad_tkernel::{
    osd::{
        CachedFileSystem, FileSystem, FileSystemHandle, FileSystemSelector, IStreamBuffer,
        IosOpenMode, LocalFileSystem, OStreamBuffer, SeekDir, SharedIStream, SharedOStream,
        SharedStreamBuf, StreamBuf, add_default_protocol, default_file_system,
        remove_default_protocol, share_stream_buf,
    },
    standard::Failure,
    tcollection::AsciiString,
};
use std::{
    any::Any,
    str::SplitWhitespace,
    sync::{Arc, Mutex},
};

/// `std::stringbuf` over the fake's name (only reading is observed by the probe).
struct NameBuffer {
    bytes: Vec<u8>,
    position: usize,
}
impl StreamBuf for NameBuffer {
    fn sgetn(&mut self, out: &mut [u8]) -> usize {
        let count: usize = out.len().min(self.bytes.len() - self.position);
        out[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        count
    }
    fn sputn(&mut self, bytes: &[u8]) -> usize {
        self.bytes.extend_from_slice(bytes);
        bytes.len()
    }
    fn pubseekoff(&mut self, offset: i64, direction: SeekDir, _which: IosOpenMode) -> i64 {
        let base: i64 = match direction {
            SeekDir::Beg => 0,
            SeekDir::Cur => self.position as i64,
            SeekDir::End => self.bytes.len() as i64,
        };
        let target: i64 = base + offset;
        if target < 0 || target > self.bytes.len() as i64 {
            return -1;
        }
        self.position = target as usize;
        target
    }
    fn pubsync(&mut self) -> i32 {
        0
    }
}

/// The native `FspFake`: logs every call, supports its prefix ("*" all) and opens a
/// buffer holding its name when allowed.
struct FspFake {
    name: String,
    prefix: String,
    opens: bool,
    log: Arc<Mutex<String>>,
}
impl FspFake {
    fn note(&self, call: &str) {
        self.log
            .lock()
            .unwrap()
            .push_str(&format!("{}{call} ", self.name));
    }
}
impl FileSystem for FspFake {
    fn is_supported_path(&self, url: &AsciiString) -> bool {
        self.note("s");
        self.prefix == "*" || url.as_bytes().starts_with(self.prefix.as_bytes())
    }
    fn is_open_istream(&self, stream: &SharedIStream) -> bool {
        self.note("i");
        let guard = stream.lock().unwrap();
        let any: &dyn Any = &*guard;
        self.opens && any.is::<IStreamBuffer>()
    }
    fn is_open_ostream(&self, stream: &SharedOStream) -> bool {
        self.note("o");
        let guard = stream.lock().unwrap();
        let any: &dyn Any = &*guard;
        self.opens && any.is::<OStreamBuffer>()
    }
    fn open_stream_buffer(
        &self,
        _url: &AsciiString,
        _mode: IosOpenMode,
        _offset: i64,
        size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        self.note("b");
        if !self.opens {
            return Ok(None);
        }
        if let Some(size) = size {
            *size = self.name.len() as i64;
        }
        Ok(Some(share_stream_buf(NameBuffer {
            bytes: self.name.clone().into_bytes(),
            position: 0,
        })))
    }
}

fn fake(spec: &str, log: &Arc<Mutex<String>>) -> FileSystemHandle {
    let fields: Vec<&str> = spec.split('/').collect();
    Arc::new(FspFake {
        name: fields[0].to_string(),
        prefix: fields[1].to_string(),
        opens: fields[2] == "1",
        log: Arc::clone(log),
    })
}

fn take_log(log: &Arc<Mutex<String>>) -> String {
    format!("|{}", std::mem::take(&mut *log.lock().unwrap()))
}

fn order(selector: &FileSystemSelector, fakes: &[FileSystemHandle], specs: &[&str]) -> String {
    let names: String = selector
        .protocols()
        .iter()
        .map(|item| {
            let index: usize = fakes.iter().position(|f| Arc::ptr_eq(f, item)).unwrap();
            specs[index].split('/').next().unwrap().to_string()
        })
        .collect();
    if names.is_empty() {
        "-".to_string()
    } else {
        names
    }
}

/// `osd_fs_selector <fakes> <ops> <url>`.
pub fn selector(input: &mut SplitWhitespace<'_>) -> String {
    let (specs, ops, url) = (token(input), token(input), token(input));
    let url: AsciiString = AsciiString::from(url.as_str());
    let log: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let specs: Vec<&str> = specs.split(',').collect();
    let fakes: Vec<FileSystemHandle> = specs.iter().map(|spec| fake(spec, &log)).collect();
    let selector: FileSystemSelector = FileSystemSelector::new();
    for op in ops.split(',').filter(|op| !op.is_empty()) {
        let item: &FileSystemHandle = &fakes[op[1..].parse::<usize>().unwrap()];
        match op.as_bytes()[0] {
            b'-' => selector.remove_protocol(item),
            kind => selector.add_protocol(Arc::clone(item), kind == b'^'),
        }
    }
    let mut out: String = format!(
        "{} {}",
        order(&selector, &fakes, &specs),
        bit(selector.is_supported_path(&url))
    );
    out += &take_log(&log);
    out += &selector_streams(&selector, &url, &log);
    out
}

fn selector_streams(
    selector: &FileSystemSelector,
    url: &AsciiString,
    log: &Arc<Mutex<String>>,
) -> String {
    let mut size: i64 = -7;
    let buffer = selector
        .open_stream_buffer(url, IosOpenMode::IN, 0, Some(&mut size))
        .unwrap();
    let mut out: String =
        buffer.map_or("null".to_string(), |b| read_buffer(&b, 8)) + &size.to_string();
    out += &take_log(log);
    let stream = selector
        .open_istream(url, IosOpenMode::IN, 0, None)
        .unwrap();
    out += &stream.as_ref().map_or("null".to_string(), |s| {
        read_stream(&mut *s.lock().unwrap(), 8)
    });
    out += &take_log(log);
    out += &bit(stream.as_ref().is_some_and(|s| selector.is_open_istream(s))).to_string();
    out += &take_log(log);
    let output = selector.open_ostream(url, IosOpenMode::OUT).unwrap();
    out += if output.is_some() { "s" } else { "null" };
    out += &take_log(log);
    out += &bit(output.as_ref().is_some_and(|s| selector.is_open_ostream(s))).to_string();
    out + &take_log(log)
}

/// `osd_fs_cached <c1> <c2> <ops>`.
pub fn cached(input: &mut SplitWhitespace<'_>) -> String {
    setup(input, 2);
    let ops: String = token(input);
    let local: FileSystemHandle = Arc::new(LocalFileSystem::new());
    let default_cached: CachedFileSystem = CachedFileSystem::new(None);
    let mut out: String = bit(Arc::ptr_eq(
        &default_cached.linked_file_system(),
        &default_file_system(),
    ))
    .to_string();
    default_cached.set_linked_file_system(Arc::clone(&local));
    out += &bit(Arc::ptr_eq(&default_cached.linked_file_system(), &local)).to_string();
    let cached: CachedFileSystem = CachedFileSystem::new(Some(local));
    let mut previous: Option<SharedIStream> = None;
    for op in ops.split(',') {
        out += &format!(" {op}=");
        out += &match cached_step(&cached, op, &mut previous) {
            Ok(text) => text,
            Err(failure) => raise(&failure),
        };
    }
    drop(previous);
    finish(out, 2)
}

fn cached_step(
    cached: &CachedFileSystem,
    op: &str,
    previous: &mut Option<SharedIStream>,
) -> Result<String, Failure> {
    let url: AsciiString = url(usize::from(op.as_bytes()[1] - b'0'));
    let offset: i64 = if op.len() > 3 {
        op[3..].parse().unwrap()
    } else {
        0
    };
    match op.as_bytes()[0] {
        b'i' => {
            let stream = cached.open_istream(&url, IosOpenMode::IN, offset, None)?;
            let same: bool = match (&stream, &*previous) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            };
            let text: String = stream.as_ref().map_or("null".to_string(), |s| {
                read_stream(&mut *s.lock().unwrap(), 3)
            });
            *previous = stream;
            Ok(format!("{}{text}", bit(same)))
        }
        kind @ (b'b' | b'w') => {
            let mut size: i64 = -7;
            let mode: IosOpenMode = if kind == b'b' {
                IosOpenMode::IN
            } else {
                IosOpenMode::OUT
            };
            let buffer = cached.open_stream_buffer(&url, mode, offset, Some(&mut size))?;
            let text: String = buffer.map_or("null".to_string(), |b| {
                if kind == b'b' {
                    read_buffer(&b, 3)
                } else {
                    b.lock().unwrap().sputn(b"Q").to_string()
                }
            });
            Ok(format!("{size}{text}"))
        }
        _ => {
            let stream = cached.open_ostream(&url, IosOpenMode::APP)?;
            Ok(stream.map_or("9".to_string(), |s| {
                let mut guard = s.lock().unwrap();
                guard.write(b"P");
                guard.rdstate().bits().to_string()
            }))
        }
    }
}

/// `osd_fs_default`.
pub fn default_protocols() -> String {
    let default: FileSystemHandle = default_file_system();
    let log: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let fake: FileSystemHandle = fake("z/z:/1", &log);
    let url: AsciiString = AsciiString::from("z://item");
    let default_any: Arc<dyn Any + Send + Sync> = default.clone();
    let mut out: String = format!(
        "{}{}{}",
        bit(Arc::ptr_eq(&default, &default_file_system())),
        bit(default_any.downcast::<FileSystemSelector>().is_ok()),
        bit(default.is_supported_path(&url))
    );
    add_default_protocol(Arc::clone(&fake), true);
    out += &bit(default.is_supported_path(&url)).to_string();
    let stream = default
        .open_istream(&url, IosOpenMode::IN, 0, None)
        .unwrap();
    out += &format!(
        " {}",
        stream.map_or("null".to_string(), |s| read_stream(
            &mut *s.lock().unwrap(),
            4
        ))
    );
    add_default_protocol(Arc::clone(&fake), false);
    add_default_protocol(Arc::clone(&fake), false);
    out += &bit(default.is_supported_path(&AsciiString::from("local.bin"))).to_string();
    remove_default_protocol(&fake);
    remove_default_protocol(&fake);
    out += &bit(default.is_supported_path(&url)).to_string();
    let text: String = log.lock().unwrap().clone();
    format!("{out} {text}")
}
