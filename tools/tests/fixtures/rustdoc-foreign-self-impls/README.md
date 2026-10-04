# Foreign-Self rustdoc fixture

These JSON files were emitted by pinned rustdoc JSON format 61 from the two
small crate sources in this directory using `cargo +nightly-2026-09-25
rustdoc --manifest-path <crate>/Cargo.toml --lib -- -Z unstable-options
--output-format json`. The base crate has three public `TryFrom` impls whose
foreign `Self` is `Circle3`; their local trait arguments differ by reference,
value, and source type. Both crates place public types behind private modules
and reexport them, and the base crate implements a local trait for the foreign
type. Its two `GenericCircle` impls verify that parameterized foreign `Self`
stays an explicit gap until its full type identity is supported. The fixture
checks public alias resolution, UFCS trait renaming, distinct trait arguments,
and stable identities after rustdoc item IDs are renumbered.
