//! Channels and event handlers over a socket pair (Unix).
#![cfg(unix)]
use rustycad_tcl::{
    Channel, ChannelHandler, Completion, DString, FileHandler, Interp, NewChannel, StdChannel,
    do_one_event, sys,
};
use std::cell::Cell;
use std::io::Write;
use std::os::fd::{AsRawFd, IntoRawFd};
use std::os::unix::net::UnixStream;
use std::rc::Rc;

/// A connected pair whose reading end is handed to Tcl as a new channel.
fn new_channel() -> (NewChannel, UnixStream) {
    let (reader, writer) = UnixStream::pair().expect("socket pair");
    let channel = NewChannel::from_file_descriptor(reader.into_raw_fd(), sys::TCL_READABLE)
        .expect("Tcl accepts the descriptor");
    (channel, writer)
}

/// The reading end registered in `interp`.
fn readable_channel(interp: &Interp) -> (Channel, UnixStream) {
    let (channel, writer) = new_channel();
    (channel.register(interp), writer)
}

#[test]
fn gets_reads_lines_from_a_file_channel() {
    let interp = Interp::new();
    let (channel, mut writer) = readable_channel(&interp);
    writer.write_all(b"first line\nsecond\n").unwrap();
    let mut line = DString::new();
    assert_eq!(channel.gets(&mut line), Some(10));
    assert_eq!(line.to_text(), "first line");
    line.clear();
    assert_eq!(channel.gets(&mut line), Some(6));
    drop(writer);
    line.clear();
    assert_eq!(channel.gets(&mut line), None);
}

#[test]
fn options_are_validated_by_tcl() {
    let (channel, _writer) = new_channel();
    channel.set_option("-translation", "auto").unwrap();
    channel.set_option("-buffering", "line").unwrap();
    // Tcl quirk: with a NULL interpreter a bad -buffering value is ignored, not refused.
    channel.set_option("-buffering", "sometimes").unwrap();
    let error = channel.set_option("-bogus", "1").unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("Tcl_SetChannelOption(\"-bogus 1\") failed"),
        "{error}"
    );
    channel.close().unwrap();
}

#[test]
fn a_script_may_close_a_channel_rust_refers_to() {
    let interp = Interp::new();
    let (channel, mut writer) = readable_channel(&interp);
    let copy = channel.clone();
    writer.write_all(b"from script\n").unwrap();
    interp
        .eval("set c [lindex [file channels file*] 0]; gets $c")
        .unwrap();
    assert_eq!(interp.result(), "from script");
    assert_eq!(interp.eval("close $c").unwrap(), Completion::Ok);
    let mut line = DString::new();
    assert_eq!(channel.gets(&mut line), None);
    let error = copy.write(b"x").unwrap_err();
    assert_eq!(
        error.to_string(),
        "Tcl_Write(\"\") failed: the channel was closed"
    );
}

#[test]
fn a_file_handler_runs_when_the_descriptor_is_readable() {
    let (reader, mut writer) = UnixStream::pair().unwrap();
    let ready = Rc::new(Cell::new(0));
    let seen = Rc::clone(&ready);
    let handler = FileHandler::new(
        reader.as_raw_fd(),
        sys::TCL_READABLE,
        Box::new(move |mask| seen.set(mask)),
    );
    assert!(!do_one_event(sys::TCL_FILE_EVENTS | sys::TCL_DONT_WAIT));
    writer.write_all(b"x").unwrap();
    while !do_one_event(sys::TCL_FILE_EVENTS | sys::TCL_DONT_WAIT) {}
    assert_eq!(ready.get(), sys::TCL_READABLE);
    drop(handler);
    assert_eq!(Rc::strong_count(&ready), 1);
}

#[test]
fn a_channel_handler_runs_and_may_be_suspended() {
    let interp = Interp::new();
    let (channel, mut writer) = readable_channel(&interp);
    let calls = Rc::new(Cell::new(0));
    let counter = Rc::clone(&calls);
    let body = Box::new(move |_| counter.set(counter.get() + 1));
    let handler = ChannelHandler::new(&channel, sys::TCL_READABLE, body).unwrap();
    writer.write_all(b"line\n").unwrap();
    while !do_one_event(sys::TCL_FILE_EVENTS | sys::TCL_DONT_WAIT) {}
    assert_eq!(calls.get(), 1);
    handler.set_mask(0).unwrap();
    assert!(!do_one_event(sys::TCL_FILE_EVENTS | sys::TCL_DONT_WAIT));
    drop(handler);
    assert_eq!(Rc::strong_count(&calls), 1);
}

#[test]
fn a_handler_outlives_the_close_of_its_channel() {
    let interp = Interp::new();
    let (channel, _writer) = readable_channel(&interp);
    let handler = ChannelHandler::new(&channel, sys::TCL_READABLE, Box::new(|_| {})).unwrap();
    interp
        .eval("close [lindex [file channels file*] 0]")
        .unwrap();
    assert!(handler.set_mask(0).is_err());
    drop(handler);
}

#[test]
fn standard_output_accepts_writes() {
    let output = Channel::standard(StdChannel::Output).expect("stdout");
    assert_eq!(output.write(b"").unwrap(), 0);
    output.flush().unwrap();
}

#[test]
fn event_loop_initializes_each_foreign_thread_before_waiting() {
    rustycad_tcl::initialize_library();
    for _ in 0..3 {
        std::thread::spawn(|| {
            assert!(!do_one_event(sys::TCL_ALL_EVENTS | sys::TCL_DONT_WAIT));
        })
        .join()
        .unwrap();
    }
}

#[test]
fn recycled_foreign_threads_keep_file_notifiers_ready() {
    rustycad_tcl::initialize_library();
    for _ in 0..32 {
        std::thread::spawn(|| {
            let (reader, mut writer) = UnixStream::pair().unwrap();
            let ready = Rc::new(Cell::new(0));
            let seen = Rc::clone(&ready);
            let _registration = FileHandler::new(
                reader.as_raw_fd(),
                sys::TCL_READABLE,
                Box::new(move |mask| seen.set(mask)),
            );
            writer.write_all(b"x").unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
            while ready.get() == 0 && std::time::Instant::now() < deadline {
                do_one_event(sys::TCL_FILE_EVENTS | sys::TCL_DONT_WAIT);
                std::thread::yield_now();
            }
            assert_eq!(ready.get(), sys::TCL_READABLE);
        })
        .join()
        .unwrap();
    }
}
