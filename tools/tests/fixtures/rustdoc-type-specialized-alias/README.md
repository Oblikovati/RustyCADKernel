# Concrete type-alias rustdoc fixture

This crate models `File = TextFile<Text>` and `CmpFile = TextFile<Compatible>`.
Its generic impls exercise inherited inherent methods, trait-qualified members,
an extra concrete trait bound, an unsatisfied bound, and an unsupported `where`
predicate. The normalizer must retain alias and trait identity and withhold cases
whose constraints cannot be proven from rustdoc records.

Capture with Rust `1.100.0-nightly (f7575a9da 2026-09-24)`:

```sh
FIXTURE=tools/tests/fixtures/rustdoc-type-specialized-alias
PATH=/home/vmiguel/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/target-rustdoc-type-alias \
  cargo +nightly-2026-09-25 rustdoc --manifest-path "$FIXTURE/Cargo.toml" \
  --lib -- -Zunstable-options --document-private-items --output-format json
cp /tmp/target-rustdoc-type-alias/doc/rustdoc_type_specialized_alias.json \
  "$FIXTURE/rustdoc-61.json"
```
