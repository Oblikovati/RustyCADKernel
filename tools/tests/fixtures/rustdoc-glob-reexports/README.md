This raw rustdoc capture was produced with the pinned nightly compiler:

```sh
cd tools/tests/fixtures/rustdoc-glob-reexports
rustdoc +nightly-2026-09-25 --crate-name rustdoc_glob_reexports --crate-type lib \
  --edition=2024 -Z unstable-options --output-format json src/lib.rs -o /tmp/rustdoc-glob-reexports
```

Compiler: `rustdoc 1.100.0-nightly (f7575a9da 2026-09-24)`, JSON format 61.
The command uses the same unstable JSON switches as the workspace capture and does
not enable `--document-private-items`.
