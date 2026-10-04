# Foreign-Self rustdoc fixture

These JSON files were emitted by pinned rustdoc JSON format 61 from the two
small crate sources in this directory using `cargo +nightly-2026-09-25
rustdoc --manifest-path <crate>/Cargo.toml --lib -- -Z unstable-options
--output-format json`. The base crate has three public `TryFrom` impls whose
foreign `Self` is `Circle3`; their local trait arguments differ by reference,
value, and source type. The fixture is used to verify reachable-owner checks
and that generic trait arguments remain distinct in normalized paths.
