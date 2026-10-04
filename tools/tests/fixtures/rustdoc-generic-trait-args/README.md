# Generic trait argument capture

This rustdoc JSON 61 fixture exercises distinct trait applications for two
`Mul` implementations, default/explicit `Self` applications, reference and
collection arguments, and a public generic trait reexport. Rustdoc records both
`Mul` and explicit `Mul<Self>` with `args: null`; the normalizer preserves that
compiler identity instead of guessing the defaulted argument. It also includes a
const-generic trait whose argument renderer is intentionally unsupported so the
normalizer must withhold that method path and report an unresolved item.

Regenerate with:

```sh
PATH=/home/vmiguel/.cargo/bin:$PATH rustdoc +nightly-2026-09-25 \
  --crate-name rustdoc_generic_trait_args \
  tools/tests/fixtures/rustdoc-generic-trait-args/src/lib.rs \
  --output-format json -Z unstable-options -o /tmp/rustdoc-generic-args
cp /tmp/rustdoc-generic-args/rustdoc_generic_trait_args.json rustdoc-61.json
```
