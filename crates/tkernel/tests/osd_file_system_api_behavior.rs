//! OSD_FileSystem, OSD_FileSystemSelector, OSD_CachedFileSystem and OSD_StreamBuffer
//! through named in-memory fakes, plus the iostream state rules they rely on.
use rustycad_tkernel::{
    osd::{
        CachedFileSystem, FileSystem, FileSystemHandle, FileSystemSelector, IStream, IStreamBuffer,
        IosOpenMode, OStream, OStreamBuffer, SeekDir, SharedIStream, SharedStreamBuf, StreamAccess,
        StreamBuf, StreamState, add_default_protocol, default_file_system, default_open_istream,
        remove_default_protocol, share_stream_buf,
    },
    standard::Failure,
    tcollection::AsciiString,
};
use std::sync::{Arc, Mutex};

/// A `std::stringbuf`-like in-memory buffer with one read/write position.
struct MemoryStreamBuf {
    bytes: Vec<u8>,
    position: usize,
    sync_result: i32,
}
impl MemoryStreamBuf {
    fn shared(text: &str) -> SharedStreamBuf {
        share_stream_buf(Self {
            bytes: text.as_bytes().to_vec(),
            position: 0,
            sync_result: 0,
        })
    }
}
impl StreamBuf for MemoryStreamBuf {
    fn sgetn(&mut self, out: &mut [u8]) -> usize {
        let count: usize = out.len().min(self.bytes.len() - self.position);
        out[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        count
    }
    fn sputn(&mut self, bytes: &[u8]) -> usize {
        self.bytes.truncate(self.position);
        self.bytes.extend_from_slice(bytes);
        self.position = self.bytes.len();
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
        self.sync_result
    }
}

/// A protocol fake: supports URLs with its prefix, opens a buffer holding its name, and
/// logs every call so the tests can check delegation order.
struct FakeProtocolFileSystem {
    name: &'static str,
    prefix: &'static str,
    opens: bool,
    log: Arc<Mutex<Vec<String>>>,
}
impl FakeProtocolFileSystem {
    fn handle(
        name: &'static str,
        prefix: &'static str,
        opens: bool,
        log: &Arc<Mutex<Vec<String>>>,
    ) -> FileSystemHandle {
        Arc::new(Self {
            name,
            prefix,
            opens,
            log: Arc::clone(log),
        })
    }
    fn note(&self, call: &str) {
        self.log
            .lock()
            .unwrap()
            .push(format!("{}.{call}", self.name));
    }
}
impl FileSystem for FakeProtocolFileSystem {
    fn is_supported_path(&self, url: &AsciiString) -> bool {
        self.note("supported");
        url.as_bytes().starts_with(self.prefix.as_bytes())
    }
    fn is_open_istream(&self, _stream: &SharedIStream) -> bool {
        self.note("is_open_i");
        self.opens
    }
    fn is_open_ostream(&self, _stream: &rustycad_tkernel::osd::SharedOStream) -> bool {
        self.note("is_open_o");
        self.opens
    }
    fn open_stream_buffer(
        &self,
        _url: &AsciiString,
        _mode: IosOpenMode,
        _offset: i64,
        size: Option<&mut i64>,
    ) -> Result<Option<SharedStreamBuf>, Failure> {
        self.note("buffer");
        if let Some(size) = size {
            *size = self.name.len() as i64;
        }
        Ok(self.opens.then(|| MemoryStreamBuf::shared(self.name)))
    }
}

fn read_all(stream: &SharedIStream) -> String {
    let mut bytes: [u8; 16] = [0; 16];
    let count: usize = stream.lock().unwrap().read(&mut bytes);
    String::from_utf8_lossy(&bytes[..count]).into_owned()
}

#[test]
fn ios_open_mode_uses_libstdcxx_bits() {
    let mode: IosOpenMode = IosOpenMode::IN | IosOpenMode::BINARY;
    assert_eq!(mode.bits(), 12);
    assert!(mode.contains(IosOpenMode::IN));
    assert!(!mode.intersects(IosOpenMode::OUT | IosOpenMode::APP));
    assert_eq!(IosOpenMode::from_bits(255).bits(), 63);
    assert_eq!((mode & IosOpenMode::BINARY), IosOpenMode::BINARY);
}

#[test]
fn stream_buffer_keeps_url_and_follows_sentry_rules() {
    let mut stream: IStreamBuffer = IStreamBuffer::new("mem://a", MemoryStreamBuf::shared("abc"));
    assert_eq!(stream.url(), "mem://a");
    let mut bytes: [u8; 2] = [0; 2];
    assert_eq!(stream.read(&mut bytes), 2);
    assert!(stream.good());
    assert_eq!(stream.read(&mut bytes), 1, "short read");
    assert_eq!(
        stream.rdstate().bits(),
        StreamState::EOF.bits() | StreamState::FAIL.bits()
    );
    assert_eq!(stream.tellg(), -1);
    stream.seekg(0, SeekDir::Beg);
    assert!(stream.fail(), "seekg clears only eof; fail stays");
    stream.clear();
    stream.seekg(1, SeekDir::Beg);
    assert_eq!(stream.tellg(), 1);
    stream.seekg(-5, SeekDir::Cur);
    assert!(stream.fail() && !stream.bad());
    assert_eq!(stream.gcount(), 1);
}

#[test]
fn output_stream_reports_short_writes_and_failed_sync_as_bad() {
    let buffer: SharedStreamBuf = share_stream_buf(MemoryStreamBuf {
        bytes: Vec::new(),
        position: 0,
        sync_result: -1,
    });
    let mut stream: OStreamBuffer = OStreamBuffer::new("mem://o", buffer);
    stream.write(b"xy");
    assert_eq!(stream.tellp(), 2);
    stream.flush();
    assert!(stream.bad());
    stream.write(b"z");
    assert_eq!(
        stream.rdstate().bits(),
        StreamState::BAD.bits() | StreamState::FAIL.bits()
    );
    assert_eq!(stream.tellp(), -1);
    assert!(!stream.eof());
}

#[test]
fn default_open_istream_reuses_only_an_open_stream_of_the_same_url() {
    let log: Arc<Mutex<Vec<String>>> = Arc::default();
    let fake: FileSystemHandle = FakeProtocolFileSystem::handle("mem", "mem:", true, &log);
    let url: AsciiString = AsciiString::from("mem://one");
    let first: SharedIStream = fake
        .open_istream(&url, IosOpenMode::NONE, 1, None)
        .unwrap()
        .unwrap();
    assert_eq!(read_all(&first), "em");
    let again = default_open_istream(fake.as_ref(), &url, IosOpenMode::NONE, 0, Some(&first))
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&again, &first), "reused");
    assert!(again.lock().unwrap().good(), "reuse clears the eof state");
    assert_eq!(read_all(&again), "mem");
    let other = fake
        .open_istream(
            &AsciiString::from("mem://two"),
            IosOpenMode::NONE,
            0,
            Some(&first),
        )
        .unwrap()
        .unwrap();
    assert!(!Arc::ptr_eq(&other, &first));
    let failure = fake
        .open_istream(&url, IosOpenMode::NONE, -2, None)
        .err()
        .unwrap();
    assert_eq!(failure.exception_type(), "Standard_ProgramError");
    assert!(failure.message().ends_with("(theOffset >= -1)"));
}

#[test]
fn selector_orders_protocols_and_delegates_lazily() {
    let log: Arc<Mutex<Vec<String>>> = Arc::default();
    let a: FileSystemHandle = FakeProtocolFileSystem::handle("a", "", false, &log);
    let b: FileSystemHandle = FakeProtocolFileSystem::handle("b", "", true, &log);
    let selector: FileSystemSelector = FileSystemSelector::new();
    selector.add_protocol(Arc::clone(&a), false);
    selector.add_protocol(Arc::clone(&b), true);
    selector.add_protocol(Arc::clone(&a), true);
    let order: Vec<bool> = selector
        .protocols()
        .iter()
        .map(|p| Arc::ptr_eq(p, &a))
        .collect();
    assert_eq!(order, vec![true, false], "re-adding moves the protocol");
    let url: AsciiString = AsciiString::from("any");
    let mut size: i64 = -1;
    let buffer = selector
        .open_stream_buffer(&url, IosOpenMode::IN, 0, Some(&mut size))
        .unwrap();
    assert!(buffer.is_some());
    assert_eq!(size, 1, "the last attempt (b) wrote the size");
    assert_eq!(
        *log.lock().unwrap(),
        ["a.supported", "a.buffer", "b.supported", "b.buffer"]
    );
    log.lock().unwrap().clear();
    assert!(selector.is_supported_path(&url));
    assert_eq!(*log.lock().unwrap(), ["a.supported"]);
    selector.remove_protocol(&a);
    selector.remove_protocol(&a);
    assert_eq!(selector.protocols().len(), 1);
    let plain: SharedIStream = Arc::new(Mutex::new(rustycad_tkernel::osd::FileStream::input()));
    assert!(
        !selector.is_open_istream(&plain),
        "not an OSD_IStreamBuffer"
    );
}

#[test]
fn cached_file_system_keeps_one_stream_per_url() {
    let log: Arc<Mutex<Vec<String>>> = Arc::default();
    let fake: FileSystemHandle = FakeProtocolFileSystem::handle("mem", "mem:", true, &log);
    let cached: CachedFileSystem = CachedFileSystem::new(Some(Arc::clone(&fake)));
    assert!(Arc::ptr_eq(&cached.linked_file_system(), &fake));
    let url: AsciiString = AsciiString::from("mem://x");
    let first = cached
        .open_istream(&url, IosOpenMode::NONE, 0, None)
        .unwrap()
        .unwrap();
    let second = cached
        .open_istream(&url, IosOpenMode::NONE, 2, None)
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(read_all(&second), "m");
    let third = cached
        .open_istream(&AsciiString::from("mem://y"), IosOpenMode::NONE, 0, None)
        .unwrap()
        .unwrap();
    assert!(!Arc::ptr_eq(&first, &third));
    assert!(cached.is_supported_path(&url));
    assert!(
        cached
            .open_ostream(&url, IosOpenMode::NONE)
            .unwrap()
            .is_some()
    );
    let default_linked: CachedFileSystem = CachedFileSystem::default();
    assert!(Arc::ptr_eq(
        &default_linked.linked_file_system(),
        &default_file_system()
    ));
    default_linked.set_linked_file_system(Arc::clone(&fake));
    assert!(Arc::ptr_eq(&default_linked.linked_file_system(), &fake));
}

#[test]
fn default_file_system_is_one_selector_with_registrable_protocols() {
    let log: Arc<Mutex<Vec<String>>> = Arc::default();
    let fake: FileSystemHandle = FakeProtocolFileSystem::handle("dflt", "dflt-test://", true, &log);
    let url: AsciiString = AsciiString::from("dflt-test://item");
    let default: FileSystemHandle = default_file_system();
    assert!(Arc::ptr_eq(&default, &default_file_system()));
    assert!(
        !default.is_supported_path(&url),
        "the local protocol refuses remote URLs"
    );
    add_default_protocol(Arc::clone(&fake), true);
    assert!(default.is_supported_path(&url));
    remove_default_protocol(&fake);
    assert!(!default.is_supported_path(&url));
    assert!(default.is_supported_path(&AsciiString::from("local.brep")));
}
