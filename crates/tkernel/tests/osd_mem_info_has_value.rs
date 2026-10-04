//! Protected OSD_MemInfo::hasValue at OCCT 3d097a0; supplemental API tests.
use rustycad_tkernel::osd::{MemCounter, MemInfo, MemInfoSource};

struct FixedMemoryCounters {
    heap: usize,
}

impl MemInfoSource for FixedMemoryCounters {
    fn heap_usage(&self) -> usize {
        self.heap
    }

    fn proc_status(&self) -> Option<String> {
        Some("VmSize: 32 kB\nVmRSS: 16 kB\nVmHWM: 24 kB\nVmData: 8 kB\nVmStk: 4 kB\n".into())
    }

    fn task_counters(&self) -> Option<(usize, usize, usize)> {
        Some((32 * 1024, 16 * 1024, self.heap))
    }
}

#[test]
fn cleared_counters_have_no_value_even_when_active() {
    let mut counters = MemInfo::with_source(FixedMemoryCounters { heap: 0 }, false);
    for counter in MemCounter::ALL {
        assert!(!counters.has_value(counter));
    }
    counters.set_active(false);
    for counter in MemCounter::ALL {
        assert!(!counters.has_value(counter));
    }
}

#[test]
#[cfg(any(target_os = "linux", target_vendor = "apple"))]
fn zero_is_known_and_reactivation_preserves_the_sample() {
    let mut counters = MemInfo::with_source(FixedMemoryCounters { heap: 0 }, true);
    let heap = MemCounter::HeapUsage;
    assert!(counters.has_value(heap));
    assert_eq!(counters.value(heap), 0);
    counters.set_counter_active(heap, false);
    assert!(!counters.has_value(heap));
    counters.set_counter_active(heap, true);
    assert!(counters.has_value(heap));
    counters.clear();
    assert!(!counters.has_value(heap));
}

#[test]
fn unknown_sentinel_does_not_become_a_known_counter() {
    let counters = MemInfo::with_source(FixedMemoryCounters { heap: usize::MAX }, true);
    assert!(!counters.has_value(MemCounter::HeapUsage));
}

#[test]
#[cfg(target_os = "linux")]
fn linux_availability_transitions_match_native_receipt() {
    let mut counters = MemInfo::with_source(FixedMemoryCounters { heap: 0 }, false);
    let mut receipt = availability(&counters);
    counters.update();
    receipt.push_str(&availability(&counters));
    counters.set_active(false);
    receipt.push_str(&availability(&counters));
    counters.set_active(true);
    receipt.push_str(&availability(&counters));
    counters.clear();
    receipt.push_str(&availability(&counters));
    assert_eq!(
        receipt,
        include_str!("../../../tools/oracle/tkernel_mem_info_has_value.expected")
    );
}

#[cfg(target_os = "linux")]
fn availability(counters: &MemInfo<FixedMemoryCounters>) -> String {
    let mut result = String::new();
    for counter in MemCounter::ALL {
        result.push(if counters.has_value(counter) {
            '1'
        } else {
            '0'
        });
    }
    result.push('\n');
    result
}
