Pinned nightly rustdoc fixture for a public trait and renamed alias exported from
an inaccessible module. The concrete implementation must use either public trait
name in the reachable API inventory, while retaining its canonical compiler
identity in `impl_context`.

From the repository root:

```sh
rustdoc +nightly-2026-09-25 --crate-name rustdoc_trait_reexports --crate-type lib \
  --edition=2024 -Z unstable-options --output-format json \
  tools/tests/fixtures/rustdoc-trait-reexports/src/lib.rs \
  -o /tmp/rustdoc-trait-reexports
cp /tmp/rustdoc-trait-reexports/rustdoc_trait_reexports.json \
  tools/tests/fixtures/rustdoc-trait-reexports/rustdoc-61.json
```

Format 61; private items are not enabled. This fixture also includes an explicit
`Clone` implementation to cover an external core trait.
