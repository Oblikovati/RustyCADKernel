//! OCCT string storage; interfaces are added alongside legacy suites.
mod ascii_string;
pub use ascii_string::{AsciiString, CatOperand};
mod extended_string;
pub use extended_string::ExtendedString;
mod h_ascii_string;
pub use h_ascii_string::HAsciiString;
mod primes;
pub use primes::next_prime_for_map;
