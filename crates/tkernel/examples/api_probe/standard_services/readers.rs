//! Rust side of the Standard_ReadBuffer / Standard_ReadLineBuffer cases
//! (tkernel_standard_services_readers.cxx).
use crate::common::{guarded, hex, next, next_bytes};
use rustycad_tkernel::standard::{Failure, ReadBuffer, ReadLineBuffer};
use std::io::{self, Read};
use std::str::SplitWhitespace;

/// Serves bytes and fails after `fail_after` of them, like the native FailingBuffer.
struct FailingStream {
    bytes: Vec<u8>,
    position: usize,
    fail_after: usize,
}

impl Read for FailingStream {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.bytes.len() {
            return Ok(0);
        }
        if self.position >= self.fail_after {
            return Err(io::Error::other("probe stream failure"));
        }
        let end = self
            .bytes
            .len()
            .min(self.fail_after)
            .min(self.position + destination.len());
        let count = end - self.position;
        destination[..count].copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(count)
    }
}

fn stream(fail: &str, bytes: Vec<u8>) -> FailingStream {
    let fail_after = if fail == "-" {
        usize::MAX
    } else {
        fail.parse().expect("count")
    };
    FailingStream {
        bytes,
        position: 0,
        fail_after,
    }
}

/// "services_read_buffer <data len> <chunk> <partial> <fail after|-> <hex> <d|u>".
pub fn read_buffer_case(input: &mut SplitWhitespace<'_>) -> String {
    let data_len: i64 = next(input);
    let chunk: usize = next(input);
    let partial = next::<i32>(input) != 0;
    let fail = input.next().expect("fail count");
    let mut source = stream(fail, next_bytes(input));
    let as_u32 = input.next() == Some("u");
    guarded(
        ReadBuffer::new(data_len, chunk, partial).and_then(|mut buffer| {
            let mut text = format!("done={}", u8::from(buffer.is_done()));
            for _ in 0..5000 {
                let (piece, more) = next_piece(&mut buffer, &mut source, as_u32)?;
                text += &format!(" {piece}");
                if !more {
                    break;
                }
            }
            Ok(text + &format!(" done={}", u8::from(buffer.is_done())))
        }),
    )
}

fn next_piece(
    buffer: &mut ReadBuffer,
    source: &mut FailingStream,
    as_u32: bool,
) -> Result<(String, bool), Failure> {
    if as_u32 {
        let value = buffer.read_chunk::<u32>(source)?;
        return Ok(value.map_or(("null".into(), false), |v| (v.to_string(), true)));
    }
    let chunk = buffer.read_data_chunk(source);
    Ok(chunk.map_or(("null".into(), false), |bytes| (hex(bytes), true)))
}

/// "services_read_line <max> <multiline> <gap> <fail after|-> <clear after|-> <hex>".
pub fn read_line_case(input: &mut SplitWhitespace<'_>) -> String {
    let max: usize = next(input);
    let multiline = next::<i32>(input) != 0;
    let gap = next::<i32>(input) != 0;
    let fail = input.next().expect("fail count");
    let clear = input.next().expect("clear count").to_owned();
    let mut source = stream(fail, next_bytes(input));
    let mut buffer = ReadLineBuffer::new(max);
    buffer.set_multiline_mode(multiline, gap);
    let pieces = read_all_lines(&mut buffer, &mut source, &clear);
    format!(
        "{} multi={} gap={}",
        pieces.join(" "),
        u8::from(buffer.is_multiline_mode()),
        u8::from(buffer.to_put_gap_in_multiline())
    )
}

/// Every line as "hex:readData" up to and including the first null, clearing after `clear` lines.
fn read_all_lines(
    buffer: &mut ReadLineBuffer,
    source: &mut FailingStream,
    clear: &str,
) -> Vec<String> {
    let mut pieces = Vec::new();
    for call in 1..=2000 {
        let mut read = 0;
        let line = buffer.read_line_counted(source, &mut read).map(hex);
        let done = line.is_none();
        pieces.push(format!("{}:{read}", line.unwrap_or_else(|| "null".into())));
        if done {
            break;
        }
        if clear == call.to_string() {
            buffer.clear();
        }
    }
    pieces
}
