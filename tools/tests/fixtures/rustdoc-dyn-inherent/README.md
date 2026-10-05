# Inherent methods on `dyn Trait`

This fixture defines public `put` and `get` methods in
`impl dyn StorageDriver + '_` and calls both from the public
`exercise_dyn_methods` function. Pinned rustdoc 1.100.0-nightly compiles that
call, but its JSON 61 capture omits the inherent impl and its two method items.
The normalized inventory therefore must not relabel trait methods as the
inherent dyn methods.

Regenerate the checked capture with:

```sh
PATH=/home/vmiguel/.cargo/bin:$PATH rustdoc +nightly-2026-09-25 \
  --crate-name rustdoc_dyn_inherent \
  --crate-type lib --edition 2024 \
  -Zunstable-options --output-format json --document-private-items \
  tools/tests/fixtures/rustdoc-dyn-inherent/lib.rs \
  -o /tmp/rustdoc-dyn-inherent-capture
```

The JSON fixture records the compiler output. This is an explicit capture gap,
not a compiler-resolved API row. The 12 `Storage_BaseDriver` operator rows
remain unresolved by the Rust API crosswalk until a capture represents the
inherent dyn self type.
