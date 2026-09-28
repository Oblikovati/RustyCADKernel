//! Behavior of the Standard scalar, character, hashing, time, GUID and atomic helpers.
//!
//! The expected values are the native OCCT results observed by the standard_values
//! differential probe (tools/verify_tkernel_api.py --only standard_values_), plus the
//! documented Rust choices for inputs whose C++ behavior is undefined.
#[path = "standard_values_api_behavior/c_string_and_hashing.rs"]
mod c_string_and_hashing;
#[path = "standard_values_api_behavior/character.rs"]
mod character;
#[path = "standard_values_api_behavior/guid.rs"]
mod guid;
#[path = "standard_values_api_behavior/integer.rs"]
mod integer;
#[path = "standard_values_api_behavior/real.rs"]
mod real;
#[path = "standard_values_api_behavior/short_real_time_atomic.rs"]
mod short_real_time_atomic;
