//! The Linux `/proc/<pid>/status` scan of `OSD_MemInfo::Update`.
use super::{MemCounter, UNKNOWN};

/// Applies each line to the counters in the native `else if` order: VmSize (virtual),
/// VmRSS (working set), VmHWM (peak), then VmData and VmStk summed into private memory.
/// Values are `atol` kB times 1024, with the native size_t wrap-around.
pub(super) fn apply(text: &str, active: &[bool; 7], counters: &mut [usize; 7]) {
    use MemCounter::*;
    let is_active = |counter: MemCounter| active[counter as usize];
    for line in text.split('\n').filter(|line| !line.is_empty()) {
        let line: &[u8] = line.as_bytes();
        if is_active(Virtual) && line.starts_with(b"VmSize:") {
            counters[Virtual as usize] = kib_bytes(&line[7..]);
        } else if is_active(WorkingSet) && line.starts_with(b"VmRSS:") {
            counters[WorkingSet as usize] = kib_bytes(&line[6..]);
        } else if is_active(WorkingSetPeak) && line.starts_with(b"VmHWM:") {
            counters[WorkingSetPeak as usize] = kib_bytes(&line[6..]);
        } else if is_active(Private) && line.starts_with(b"VmData:") {
            add_private(counters, kib_bytes(&line[7..]));
        } else if is_active(Private) && line.starts_with(b"VmStk:") {
            add_private(counters, kib_bytes(&line[6..]));
        }
    }
}

fn add_private(counters: &mut [usize; 7], bytes: usize) {
    let slot: &mut usize = &mut counters[MemCounter::Private as usize];
    if *slot == UNKNOWN {
        *slot = 0; // Native ++size_t(-1).
    }
    *slot = slot.wrapping_add(bytes);
}

/// `atol(text) * 1024` stored in a size_t.
fn kib_bytes(text: &[u8]) -> usize {
    (atol(text).wrapping_mul(1024)) as usize
}

/// C `atol`: leading white space, an optional sign, then decimal digits; 0 without digits.
pub(super) fn atol(text: &[u8]) -> i64 {
    let mut rest: &[u8] = text;
    while let [first, tail @ ..] = rest
        && matches!(first, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
    {
        rest = tail;
    }
    let negative: bool = rest.first() == Some(&b'-');
    if matches!(rest.first(), Some(b'-' | b'+')) {
        rest = &rest[1..];
    }
    let magnitude: i64 =
        rest.iter()
            .take_while(|byte| byte.is_ascii_digit())
            .fold(0i64, |sum, &digit| {
                sum.saturating_mul(10)
                    .saturating_add(i64::from(digit - b'0'))
            });
    if negative { -magnitude } else { magnitude }
}
