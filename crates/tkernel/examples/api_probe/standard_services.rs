//! Rust side of the differential cases for Standard memory managers, synchronization, locale, buffered readers, dump and Plugin.
use super::common::Case;
mod memory;
mod mmgr;

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
];
