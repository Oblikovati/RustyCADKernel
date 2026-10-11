# Martensite sensitive-set traversal extraction

Base main: `cdd2363ef`. This refactor adds no API claims or ledger changes.

The helpers preserve native call order and policy behavior: overlap policy is
queried per element; one scratch pick survives each range; nearest selection
retains the original strict `>` predicate including NaN behavior; child box
checks occur left before right; right siblings are pushed before descending left;
inclusion failures stop traversal immediately; taken BVH trees are restored
when the callback returns `Failure`. Every original traversal comment remains.

Local Linux checks passed:

- Full `cargo test -p rustycad-tkv3d --all-targets --locked`: 599 tests
  (128 library, 1 isolated, 470 toolkit), including unchanged committed native
  matching (9,360 snapshots) and lifecycle/map (129 snapshots) corpora.
- Four targeted regressions prove policy mutation, scratch reuse, NaN nearest
  comparisons and restoration after callback failure.
- `cargo clippy -p rustycad-tkv3d --all-targets --locked -- -D warnings`.
- Focused `rustfmt --check --edition 2024` on traversal and child test modules.
- Canonical toolkit discovery and its 13 regression tests.
- All affected files under 500 lines; functions 4–20 formatted lines and at most
  two nested control-flow levels. See `style-audit.log`.

Global `tools/check_rust_format.py` fails on inherited Vulkan test formatting at
`crates/vulkan/tests/surface_blending.rs:69`, outside this change. The coordinator
must resolve that formatting issue before the final global check. Its failure
receipt is retained in `format.log`; all touched paths pass focused formatting.

Builds used disabled debug information/incremental compilation and two jobs.
No oracle regeneration, GPU work, Windows/macOS checks, pushes or pull requests.
Long logs are compressed with deterministic gzip; hashes are recorded separately.
