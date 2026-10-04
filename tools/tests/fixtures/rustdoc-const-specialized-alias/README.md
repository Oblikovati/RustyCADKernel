# Const-specialized generic alias fixture

This fixture records rustdoc JSON format 61 for public aliases that bind the
const parameter of a two-parameter generic struct. It checks that only methods
from the exact specialized inherent impl are reachable from each alias.

Capture with Rust `1.100.0-nightly (f7575a9da 2026-09-24)`:

```sh
PATH=/home/vmiguel/.cargo/bin:$PATH \
CARGO_TARGET_DIR=/tmp/target-rustdoc-const-alias \
cargo +nightly-2026-09-25 rustdoc \
  --manifest-path tools/tests/fixtures/rustdoc-const-specialized-alias/Cargo.toml \
  --lib -- -Zunstable-options --output-format json
cp /tmp/target-rustdoc-const-alias/doc/rustdoc_const_specialized_alias.json \
  tools/tests/fixtures/rustdoc-const-specialized-alias/rustdoc-61.json
```

The test mutates copies of this compiler output to verify rejection of
non-literal alias const arguments and unrelated const-specialized impls.
