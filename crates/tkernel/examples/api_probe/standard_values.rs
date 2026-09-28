//! Rust side of the differential cases for Standard scalar, character, hashing, time, GUID and atomic helpers.
//! The C++ twin is tools/oracle/tkernel_standard_values_probe.cxx and the files it includes.
mod guid;
mod hash;
mod scalar;

use super::common::Case;

/// Operation name and handler of every case this area answers.
pub const CASES: &[(&str, Case)] = &[
    ("standard_values_real1", scalar::real1_case),
    ("standard_values_real2", scalar::real2_case),
    ("standard_values_real_to_int", scalar::real_to_int_case),
    (
        "standard_values_real_to_short_real",
        scalar::real_to_short_real_case,
    ),
    ("standard_values_int_to_real", scalar::int_to_real_case),
    ("standard_values_short_real", scalar::short_real_case),
    ("standard_values_integer", scalar::integer_case),
    ("standard_values_character", scalar::character_case),
    ("standard_values_char_equal", scalar::char_equal_case),
    (
        "standard_values_to_ext_character",
        scalar::to_ext_character_case,
    ),
    ("standard_values_ext_character", scalar::ext_character_case),
    ("standard_values_limits", scalar::limits_case),
    ("standard_values_hash_bytes", hash::hash_bytes_case),
    ("standard_values_hash_combine", hash::hash_combine_case),
    ("standard_values_hash_value", hash::hash_value_case),
    ("standard_values_hash_seeds", hash::hash_seeds_case),
    ("standard_values_c_string_equal", hash::c_string_equal_case),
    ("standard_values_atof", hash::atof_case),
    ("standard_values_guid_text", guid::guid_text_case),
    ("standard_values_guid_utf16", guid::guid_utf16_case),
    (
        "standard_values_guid_components",
        guid::guid_components_case,
    ),
    ("standard_values_guid_compare", guid::guid_compare_case),
    ("standard_values_time_equal", guid::time_equal_case),
    ("standard_values_atomic", guid::atomic_case),
    ("standard_values_version", guid::version_case),
];
