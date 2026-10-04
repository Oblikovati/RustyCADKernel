//! Signed override behavior compared with a derived native stream buffer.
use rustycad_tkernel::standard::{
    ArrayStreamBuffer, ArrayStreamBufferHooks, ArrayStreamBufferState,
};
use std::io::{self, Read};

#[derive(Default)]
struct SignedReadHook {
    calls: usize,
}

impl ArrayStreamBufferHooks for SignedReadHook {
    fn xsgetn_signed(
        &mut self,
        state: &mut ArrayStreamBufferState<'_>,
        output: &mut [u8],
        count: isize,
    ) -> io::Result<isize> {
        self.calls += 1;
        if count < 0 {
            return Ok(count - 4);
        }
        if count == 0 {
            return Ok(-11);
        }
        Ok(state.xsgetn(&mut output[..count as usize]) as isize)
    }
}

#[test]
fn signed_virtual_requests_and_results_match_native() {
    let mut buffer = ArrayStreamBuffer::with_hooks(b"ABCD", SignedReadHook::default());
    let mut output = [0; 4];
    let negative = buffer.xsgetn_signed(&mut output, -3).unwrap();
    let first = format!("{negative} {}\n", buffer.hooks_mut().calls);
    let zero = buffer.xsgetn_signed(&mut output, 0).unwrap();
    let second = format!("{zero} {}\n", buffer.hooks_mut().calls);
    let copied = buffer.xsgetn_signed(&mut output, 2).unwrap();
    let bytes = std::str::from_utf8(&output[..copied as usize]).unwrap();
    let calls = buffer.hooks_mut().calls;
    let next = buffer.underflow();
    let third = format!("{copied} {calls} {bytes}\n{next}\n");
    assert_eq!(
        format!("{first}{second}{third}"),
        include_str!("../../../tools/oracle/array_stream_signed_reads.expected")
    );
}

#[test]
fn signed_default_bounds_reject_invalid_reads_without_consuming() {
    let mut buffer = ArrayStreamBuffer::new(b"ABCD");
    let mut output = [0; 2];
    for count in [-1, 3] {
        let error = buffer.xsgetn_signed(&mut output, count).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains(&count.to_string()));
        assert_eq!(buffer.available(), 4);
        assert_eq!(output, [0; 2]);
    }
    assert_eq!(buffer.xsgetn_signed(&mut output, 2).unwrap(), 2);
    assert_eq!(&output, b"AB");
    assert_eq!(buffer.xsgetn_signed(&mut output, 3).unwrap(), 2);
    assert_eq!(&output, b"CD");
    assert_eq!(buffer.xsgetn_signed(&mut output, 0).unwrap(), 0);
}

#[test]
fn byte_and_read_adapters_dispatch_the_signed_override() {
    let mut buffer = ArrayStreamBuffer::with_hooks(b"ABCD", SignedReadHook::default());
    assert_eq!(buffer.xsgetn(&mut []), 0);
    assert_eq!(
        buffer.read(&mut []).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    let mut output = [0; 2];
    assert_eq!(buffer.read(&mut output).unwrap(), 2);
    assert_eq!(&output, b"AB");
    assert_eq!(buffer.hooks_mut().calls, 3);
}
