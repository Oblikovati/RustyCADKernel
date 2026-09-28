//! OSD_LocalFileSystem, std::filebuf and the OSD_OpenFile helpers on real files in a
//! per-test scratch directory (the native behavior under test is the file system's).
use rustycad_tkernel::{
    osd::{
        FileBuffer, FileStream, FileSystem, IStream, IosOpenMode, LocalFileSystem, OStream,
        SeekDir, StreamAccess, StreamBuf, file_stat_ctime, open_file, open_file_descriptor,
        open_file_stream, open_file_stream_utf8, open_file_utf8, open_stream,
    },
    tcollection::{AsciiString, ExtendedString},
};
use std::{
    io::{Read, Write},
    path::PathBuf,
};

/// A scratch directory removed when the test ends.
struct ScratchDirectory {
    root: PathBuf,
}
impl ScratchDirectory {
    fn new(test: &str) -> Self {
        let root: PathBuf =
            std::env::temp_dir().join(format!("osd-fs-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }
    fn file(&self, content: &str) -> PathBuf {
        let path: PathBuf = self.root.join("f.bin");
        std::fs::write(&path, content).unwrap();
        path
    }
    fn text(&self) -> String {
        std::fs::read_to_string(self.root.join("f.bin")).unwrap_or_else(|_| "NOFILE".into())
    }
}
impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn url(path: &std::path::Path) -> AsciiString {
    AsciiString::from(path.to_str().unwrap())
}

#[test]
fn local_file_system_refuses_only_remote_protocols() {
    let local: LocalFileSystem = LocalFileSystem::new();
    assert!(local.is_supported_path(&AsciiString::from("C:/part.step")));
    assert!(local.is_supported_path(&AsciiString::from("a:b")));
    assert!(!local.is_supported_path(&AsciiString::from("https://host/part")));
}

#[test]
fn local_buffer_reports_size_and_refuses_negative_offsets() {
    let scratch: ScratchDirectory = ScratchDirectory::new("buffer");
    let path: PathBuf = scratch.file("abcdef");
    let local: LocalFileSystem = LocalFileSystem::new();
    let mut size: i64 = -7;
    let buffer = local
        .open_stream_buffer(&url(&path), IosOpenMode::IN, 2, Some(&mut size))
        .unwrap()
        .unwrap();
    assert_eq!(size, 6);
    let mut bytes: [u8; 8] = [0; 8];
    assert_eq!(buffer.lock().unwrap().sgetn(&mut bytes), 4);
    let failure = local
        .open_stream_buffer(&url(&path), IosOpenMode::IN, -1, None)
        .err()
        .unwrap();
    assert!(failure.message().ends_with("(theOffset >= 0)"));
    let missing = scratch.root.join("missing.bin");
    assert!(
        local
            .open_stream_buffer(&url(&missing), IosOpenMode::IN, 0, None)
            .unwrap()
            .is_none()
    );
    let stream = local
        .open_istream(&url(&path), IosOpenMode::NONE, 0, None)
        .unwrap()
        .unwrap();
    assert!(local.is_open_istream(&stream));
    let output = local
        .open_ostream(&url(&path), IosOpenMode::APP)
        .unwrap()
        .unwrap();
    assert!(local.is_open_ostream(&output));
}

#[test]
fn filebuf_accepts_only_the_libstdcxx_mode_table() {
    let scratch: ScratchDirectory = ScratchDirectory::new("modes");
    let path: PathBuf = scratch.file("abc");
    let mut buffer: FileBuffer = FileBuffer::new();
    assert!(
        !buffer.open(&path, IosOpenMode::TRUNC),
        "trunc alone is invalid"
    );
    assert!(!buffer.open(&path, IosOpenMode::IN | IosOpenMode::TRUNC));
    assert!(buffer.open(&path, IosOpenMode::IN | IosOpenMode::ATE));
    assert!(!buffer.open(&path, IosOpenMode::IN), "already open");
    assert_eq!(buffer.pubseekoff(0, SeekDir::Cur, IosOpenMode::IN), 3);
    assert_eq!(buffer.sputn(b"x"), 0, "read-only buffer");
    assert_eq!(buffer.pubseekoff(-1, SeekDir::Beg, IosOpenMode::IN), -1);
    assert!(buffer.close());
    assert!(!buffer.close());
    assert_eq!(buffer.pubseekoff(0, SeekDir::Cur, IosOpenMode::IN), -1);
}

#[test]
fn append_filebuf_starts_at_the_end_and_reports_the_pending_position() {
    let scratch: ScratchDirectory = ScratchDirectory::new("append");
    let path: PathBuf = scratch.file("abcdef");
    let mut buffer: FileBuffer = FileBuffer::new();
    assert!(open_stream(
        &mut buffer,
        &ExtendedString::from_utf8(path.to_str().unwrap()),
        IosOpenMode::APP
    ));
    assert_eq!(
        buffer.pubseekoff(0, SeekDir::Cur, IosOpenMode::OUT),
        6,
        "glibc 'a' starts at the end"
    );
    assert_eq!(buffer.pubseekoff(1, SeekDir::Beg, IosOpenMode::OUT), 1);
    assert_eq!(buffer.sputn(b"Z"), 1);
    assert_eq!(
        buffer.pubseekoff(0, SeekDir::Cur, IosOpenMode::OUT),
        2,
        "native put-area arithmetic"
    );
    assert_eq!(buffer.pubsync(), 0);
    drop(buffer);
    assert_eq!(scratch.text(), "abcdefZ");
}

#[test]
fn fopen_modes_follow_glibc() {
    let scratch: ScratchDirectory = ScratchDirectory::new("fopen");
    let path: PathBuf = scratch.file("abc");
    let name: &str = path.to_str().unwrap();
    assert!(open_file_utf8(name, "q").is_none());
    assert!(
        open_file_utf8(name, "wx").is_none(),
        "exclusive create of an existing file"
    );
    let mut file = open_file(&ExtendedString::from_utf8(name), "a").unwrap();
    file.write_all(b"XY").unwrap();
    drop(file);
    assert_eq!(scratch.text(), "abcXY");
    let mut text: String = String::new();
    open_file_utf8(name, "rbbbbb+")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    assert_eq!(text, "abcXY");
    assert!(file_stat_ctime(name) > 0);
    assert_eq!(
        file_stat_ctime(scratch.root.join("none").to_str().unwrap()),
        0
    );
}

#[test]
fn descriptor_with_out_is_write_only_and_truncating() {
    let scratch: ScratchDirectory = ScratchDirectory::new("fd");
    let path: PathBuf = scratch.file("abcdef");
    let name: ExtendedString = ExtendedString::from_utf8(path.to_str().unwrap());
    let mut file = open_file_descriptor(&name, IosOpenMode::IN | IosOpenMode::OUT).unwrap();
    let mut bytes: [u8; 4] = [0; 4];
    assert!(
        file.read(&mut bytes).is_err(),
        "O_RDONLY | O_WRONLY is write-only"
    );
    drop(file);
    assert_eq!(scratch.text(), "");
    let reader = open_file_descriptor(&name, IosOpenMode::NONE);
    assert!(reader.is_some(), "no in/out opens read-only");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let fresh = scratch.root.join("fresh.bin");
        drop(open_file_descriptor(
            &ExtendedString::from_utf8(fresh.to_str().unwrap()),
            IosOpenMode::OUT,
        ));
        let mode: u32 = std::fs::metadata(fresh).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode & !0o600, 0, "created with 0600 (less the umask)");
    }
}

#[test]
fn file_streams_add_their_implied_mode_and_track_state() {
    let scratch: ScratchDirectory = ScratchDirectory::new("fstream");
    let path: PathBuf = scratch.file("abc");
    let mut input: FileStream = FileStream::input();
    open_file_stream_utf8(&mut input, path.to_str().unwrap(), IosOpenMode::NONE);
    assert!(input.is_open() && input.good());
    let mut bytes: [u8; 8] = [0; 8];
    assert_eq!(IStream::read(&mut input, &mut bytes), 3);
    input.close();
    assert!(!input.is_open());
    input.close();
    assert!(input.fail(), "closing a closed stream sets failbit");
    let mut output: FileStream = FileStream::output();
    open_file_stream(
        &mut output,
        &ExtendedString::from_utf8(path.to_str().unwrap()),
        IosOpenMode::APP,
    );
    OStream::write(&mut output, b"XY");
    assert_eq!(output.tellp(), 5);
    output.close();
    assert_eq!(scratch.text(), "abcXY");
    let mut both: FileStream = FileStream::input_output();
    open_file_stream_utf8(&mut both, path.to_str().unwrap(), IosOpenMode::TRUNC);
    assert!(!both.is_open() && both.fail(), "trunc alone is refused");
}
