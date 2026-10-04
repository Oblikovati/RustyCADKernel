# Const-specialized generic alias fixture

This compiler fixture places `CompactBounds` and its implementation in private
modules, then reexports only `CompactBox2` and `CompactBox3`. It records both
default rustdoc output and output captured with private items. The normalizer
must use private target impl metadata to resolve methods through those public
aliases while emitting no private module paths.

Capture with Rust `1.100.0-nightly (f7575a9da 2026-09-24)`:

```sh
FIXTURE=tools/tests/fixtures/rustdoc-const-specialized-alias
PATH=/home/vmiguel/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/target-rustdoc-alias-private \
  cargo +nightly-2026-09-25 rustdoc --manifest-path "$FIXTURE/Cargo.toml" \
  --lib -- -Zunstable-options --document-private-items --output-format json
cp /tmp/target-rustdoc-alias-private/doc/rustdoc_const_specialized_alias.json \
  "$FIXTURE/rustdoc-61.json"
PATH=/home/vmiguel/.cargo/bin:$PATH CARGO_TARGET_DIR=/tmp/target-rustdoc-alias-default \
  cargo +nightly-2026-09-25 rustdoc --manifest-path "$FIXTURE/Cargo.toml" \
  --lib -- -Zunstable-options --output-format json
cp /tmp/target-rustdoc-alias-default/doc/rustdoc_const_specialized_alias.json \
  "$FIXTURE/rustdoc-61-default.json"
```

Without private-item capture, rustdoc omits the private target's inherent impl
index. The normalizer reports that missing evidence as unresolved instead of
claiming the alias has no methods. With private-item capture, only methods
reachable through public aliases are emitted. The fixture also checks exact
const binding, rejects const-dependent `where` clauses, and mutates compiler
output to reject nonliteral aliases and unrelated const-specialized impls.
