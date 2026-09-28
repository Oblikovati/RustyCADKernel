//! Rust side of the differential cases for Standard memory managers, synchronization, locale, buffered readers, dump and Plugin.
use super::common::Case;
mod array_stream;
mod locale;
mod memory;
mod mmgr;
mod readers;
mod sync;

/// Operation name and handler of every case this area answers.
pub const CASES: &[(&str, Case)] = &[
    ("services_standard_allocate", memory::allocate_case),
    (
        "services_standard_allocate_optimal",
        memory::allocate_optimal_case,
    ),
    ("services_standard_reallocate", memory::reallocate_case),
    ("services_standard_aligned", memory::aligned_case),
    ("services_standard_misc", memory::misc_case),
    ("services_mmgr", mmgr::script_case),
    ("services_mutex", sync::mutex_case),
    ("services_condition", sync::condition_case),
    ("services_clocale", locale::clocale_case),
    ("services_read_buffer", readers::read_buffer_case),
    ("services_read_line", readers::read_line_case),
    ("services_array_stream", array_stream::array_stream_case),
];
