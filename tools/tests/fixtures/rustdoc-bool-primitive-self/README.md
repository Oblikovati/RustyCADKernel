# Primitive foreign-Self rustdoc fixture

`rustdoc-61.json` was emitted by `rustdoc 1.92.0-nightly (2026-09-25)` from
`src/lib.rs` using:

```sh
rustup run nightly-2026-09-25 rustdoc \
  --edition=2024 --crate-name bool_fixture --crate-type lib src/lib.rs \
  --output-format json -Z unstable-options --out-dir target
```

The fixture defines `From<&ResultRecord> for bool` while the record is in a
private module and is reachable through `api::ResultRecord`. It pins the case
where rustdoc represents primitive `Self` as `{"primitive":"bool"}` and
lists the impl on its public local trait-argument type. Source spans are
relative to this fixture root so the normalizer can verify the captured source
hashes without rewriting provenance paths.
