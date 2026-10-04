//! State setters retain the previous gcount, unlike a new unformatted read.
use super::IstreamStream;
use crate::standard::IstreamExceptionMask;

#[test]
fn exception_mask_and_clear_state_preserve_the_previous_short_read_count() {
    let mut stream = IstreamStream::new(&b"ab"[..]);
    assert_eq!(stream.read_block(&mut [0; 4]).unwrap().gcount(), 2);
    let mask = IstreamExceptionMask::new(false, true, false);
    assert_eq!(stream.set_exceptions(mask).unwrap_err().gcount(), 2);
    assert_eq!(
        stream.clear_state(false, true, false).unwrap_err().gcount(),
        2
    );
    stream.clear();
    assert_eq!(stream.gcount(), 2);
    assert_eq!(stream.read_block(&mut []).unwrap().gcount(), 0);
    assert_eq!(stream.gcount(), 0);
}

#[test]
fn each_native_block_read_resets_and_then_updates_its_count() {
    let mut stream = IstreamStream::new(&b"abcd"[..]);
    assert_eq!(stream.gcount(), 0);
    stream.read_block(&mut [0; 2]).unwrap();
    assert_eq!(stream.gcount(), 2);
    stream.read_block(&mut [0; 1]).unwrap();
    assert_eq!(stream.gcount(), 1);
    stream.set_status(false, true, false);
    assert_eq!(stream.read_block(&mut [0; 1]).unwrap().gcount(), 0);
    assert_eq!(stream.gcount(), 0);
}
