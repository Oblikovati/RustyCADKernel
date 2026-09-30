//! The deprecated `TCollection` class: `NextPrimeForMap` over `int`.
use crate::ncollection::next_map_prime;

/// `TCollection::NextPrimeForMap`: `NCollection_Primes::NextPrimeForMap` called with the
/// `int` converted to `size_t` and the result narrowed back to `int`; `next_prime_for_map(100) == 101`.
///
/// The conversions are part of the result: -1 becomes `SIZE_MAX`, whose `+ 1` wraps to 0,
/// so it maps to 101; other negative values and values above the last table prime come
/// back as `value + 1`, which for `i32::MAX` wraps to `i32::MIN`.
pub fn next_prime_for_map(value: i32) -> i32 {
    let size = value as isize as usize;
    // `NCollection_Primes` looks for the first table prime >= size + 1 in wrapping size_t
    // arithmetic; `next_map_prime(n)` answers "first prime > n", i.e. ">= n + 1".
    let wanted = size.wrapping_add(1);
    next_map_prime(wanted.saturating_sub(1)) as i32
}
