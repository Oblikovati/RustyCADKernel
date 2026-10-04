# FSD generic alias inventory correction

The pinned Rust source defines `fsd::File` as `TextFile<FsdDialect>` and
`fsd::CmpFile` as `TextFile<CmpDialect>`. Their missing Rust targets were a
rustdoc inventory limitation: methods on the generic `TextFile<D>` impls were
not projected onto either concrete alias.

`rustdoc_type_aliases.py` specializes only a public zero-parameter alias to a
one-type-parameter local struct when the concrete argument is a closed nominal
type and rustdoc proves every required trait bound. It also requires exact
`Self = TextFile<D>` links for impl members. Unsupported `where` clauses,
generic owners, unresolved type links, and unproven bounds remain unresolved.
Trait methods keep their fully qualified trait path; no method names are
guessed or collapsed between aliases.

The pinned `text_file.rs` source SHA-256 is
`f4bf7d26ba3c83c7a25b4c312d7913ad5e88b3d01d254b0167229c0075e908e3`.
The Rustdoc format-61 fixture in
`tools/tests/fixtures/rustdoc-type-specialized-alias/` records compiler
provenance and covers distinct aliases, trait impls, satisfied and unsatisfied
bounds, and unsupported `where` predicates.

The refreshed workspace capture normalized 58,610 declaration paths across
20 crates. Both FSD aliases resolve with no alias-specific unresolved links.
The parity ledger now points 83 FSD members at unique, source-pinned public
trait-qualified alias paths: 15 `HeaderSections`, 24 `DataSections`, 24
`TableSections`, 14 `DriverPrimitives`, and 6 `BaseDriver` members. API and
legacy-test status counts were unchanged; this corrects Rust declaration
identity in the crosswalk and does not change runtime test evidence.
