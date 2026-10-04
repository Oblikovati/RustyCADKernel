//! Verifies trait overrides are dispatched over the adapter's shared buffer state.
use std::io::{self, BufRead, Read, Seek, SeekFrom};

use rustycad_tkernel::standard::{
    ArrayStreamBuffer, ArrayStreamBufferHooks, ArrayStreamBufferState, ArrayStreamSeekDirection,
};

#[derive(Default)]
struct HookCalls {
    init: usize,
    underflow: usize,
    uflow: usize,
    pbackfail: usize,
    showmanyc: usize,
    seekoff: usize,
    seekpos: usize,
    xsgetn: usize,
}

impl ArrayStreamBufferHooks for HookCalls {
    fn init<'a>(&mut self, state: &mut ArrayStreamBufferState<'a>, bytes: &'a [u8]) {
        self.init += 1;
        state.init(bytes);
    }

    fn underflow(&mut self, state: &mut ArrayStreamBufferState<'_>) -> i32 {
        self.underflow += 1;
        if state.underflow() == -1 {
            return -1;
        }
        state.set_position(state.position() + 1).unwrap();
        300
    }

    fn uflow(&mut self, state: &mut ArrayStreamBufferState<'_>) -> i32 {
        self.uflow += 1;
        state.uflow()
    }

    fn pbackfail(&mut self, state: &mut ArrayStreamBufferState<'_>, requested: i32) -> i32 {
        self.pbackfail += 1;
        if state.pbackfail(requested) == -1 {
            -1
        } else {
            511
        }
    }

    fn showmanyc(&mut self, state: &mut ArrayStreamBufferState<'_>) -> isize {
        self.showmanyc += 1;
        state.set_position(state.bytes().len()).unwrap();
        state.available() as isize
    }

    fn seekoff(
        &mut self,
        state: &mut ArrayStreamBufferState<'_>,
        offset: i64,
        direction: ArrayStreamSeekDirection,
    ) -> io::Result<u64> {
        self.seekoff += 1;
        state.seekoff(offset, direction)
    }

    fn seekpos(
        &mut self,
        state: &mut ArrayStreamBufferState<'_>,
        position: u64,
    ) -> io::Result<u64> {
        self.seekpos += 1;
        self.seekoff(
            state,
            i64::try_from(position).unwrap(),
            ArrayStreamSeekDirection::Begin,
        )
    }

    fn xsgetn(&mut self, state: &mut ArrayStreamBufferState<'_>, output: &mut [u8]) -> usize {
        self.xsgetn += 1;
        state.xsgetn(output)
    }
}

struct EofUnderflow;

impl ArrayStreamBufferHooks for EofUnderflow {
    fn underflow(&mut self, _state: &mut ArrayStreamBufferState<'_>) -> i32 {
        -1
    }
}

#[test]
fn overridden_hooks_dispatch_through_shared_cursor() {
    let (values, calls) = run_overridden_hooks();
    let expected = include_str!("../../../tools/oracle/array_stream_override_dispatch.expected");
    let observed = format!("{values}\n{}\n", call_line(calls));
    assert_eq!(observed, expected);
}

fn run_overridden_hooks() -> (String, [usize; 8]) {
    let mut buffer = ArrayStreamBuffer::with_hooks(b"ABC", HookCalls::default());
    let values = exercise_sequence(&mut buffer);
    buffer.init(b"XY");
    assert_eq!(buffer.peek(), Some(b'X'));
    (values, hook_counts(&mut buffer))
}

fn exercise_sequence(buffer: &mut ArrayStreamBuffer<'_, HookCalls>) -> String {
    let underflow = buffer.underflow();
    assert_eq!(buffer.peek(), Some(b'B'));
    let uflow = buffer.uflow_int();
    let putback = buffer.pbackfail(i32::from(b'B'));
    let available = buffer.showmanyc();
    let sought = buffer.seekoff(2, ArrayStreamSeekDirection::Begin).unwrap();
    let seekpos = buffer.seekpos(1).unwrap();
    let bytes = read_all(buffer);
    let count = bytes.len();
    let bytes = String::from_utf8(bytes).unwrap();
    format!("{underflow} {uflow} {putback} {available} {sought} {seekpos} {count} {bytes}")
}

fn hook_counts(buffer: &mut ArrayStreamBuffer<'_, HookCalls>) -> [usize; 8] {
    let hooks = buffer.hooks_mut();
    [
        hooks.init,
        hooks.underflow,
        hooks.uflow,
        hooks.pbackfail,
        hooks.showmanyc,
        hooks.seekoff,
        hooks.seekpos,
        hooks.xsgetn,
    ]
}

fn call_line(calls: [usize; 8]) -> String {
    calls.map(|call| call.to_string()).join(" ")
}

#[test]
fn bufread_remains_a_storage_adapter_for_custom_eof_hooks() {
    let mut buffer = ArrayStreamBuffer::with_hooks(b"A", EofUnderflow);
    assert_eq!(buffer.underflow(), -1);
    assert_eq!(buffer.fill_buf().unwrap(), b"A");
}

#[test]
fn rust_seek_keeps_existing_clamping_and_rejects_negative_positions() {
    let mut buffer = ArrayStreamBuffer::new(b"AB");
    assert_eq!(buffer.seek(SeekFrom::Start(u64::MAX)).unwrap(), 2);
    assert_eq!(buffer.seek(SeekFrom::End(1)).unwrap(), 2);
    assert_eq!(buffer.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert!(buffer.seek(SeekFrom::Current(-1)).is_err());
    assert_eq!(buffer.peek(), Some(b'A'));
}

#[test]
fn invalid_occt_seek_direction_is_a_dispatched_noop() {
    let mut buffer = ArrayStreamBuffer::new(b"AB");
    buffer.seekpos(1).unwrap();
    assert_eq!(
        buffer.seekoff(99, ArrayStreamSeekDirection::Other).unwrap(),
        1
    );
    assert_eq!(buffer.peek(), Some(b'B'));
}

#[test]
fn default_hooks_keep_native_cursor_behavior() {
    let mut buffer = ArrayStreamBuffer::new(b"A");
    assert_eq!(buffer.underflow(), 65);
    assert_eq!(buffer.showmanyc(), 1);
    assert_eq!(buffer.uflow_int(), 65);
    assert_eq!(buffer.pbackfail(-1), 65);
    assert_eq!(
        buffer.seekoff(0, ArrayStreamSeekDirection::Begin).unwrap(),
        0
    );
    assert_eq!(buffer.seekpos(1).unwrap(), 1);
}

fn read_all<H: ArrayStreamBufferHooks>(buffer: &mut ArrayStreamBuffer<'_, H>) -> Vec<u8> {
    let mut output = [0; 3];
    let count = buffer.read(&mut output).unwrap();
    output[..count].to_vec()
}
