# Cross-crate trait reexport fixture

This fixture captures two crates with pinned nightly Rustdoc JSON format 61.
The consumer crate publicly aliases a dependency trait and reexports its type.
The regression runs both compiler captures through the production reexport and
trait-path resolvers, so the method path must use the consumer's public alias
while its implementation context retains the provider's canonical trait path.

Regenerate the captures from this directory with `nightly-2026-09-25`:

```sh
mkdir -p target provider-doc consumer-doc
rustc +nightly-2026-09-25 --crate-name trait_provider --crate-type lib \
  --edition=2024 provider/src/lib.rs --out-dir target
rustc +nightly-2026-09-25 --crate-name trait_consumer --crate-type lib \
  --edition=2024 --extern trait_provider=target/libtrait_provider.rlib \
  consumer/src/lib.rs --out-dir target
rustdoc +nightly-2026-09-25 --crate-name trait_provider --crate-type lib \
  --edition=2024 -Z unstable-options --output-format json \
  provider/src/lib.rs -o provider-doc
rustdoc +nightly-2026-09-25 --crate-name trait_consumer --crate-type lib \
  --edition=2024 -Z unstable-options --output-format json \
  --extern trait_provider=target/libtrait_provider.rlib \
  consumer/src/lib.rs -o consumer-doc
cp provider-doc/trait_provider.json provider-rustdoc-61.json
cp consumer-doc/trait_consumer.json consumer-rustdoc-61.json
```

Both checked captures report `format_version: 61`; the fixture sources are
hashed by the inventory normalizer when the test builds their declarations.
