# Semantic-name dictionary V1 checkpoint

Source: `4c9511d80bf02ff9a9f6e9d554c7a4c57f313622`.

Pinned Rust 1.99.0, jobs 2: 14 focused Cargo tests and 3 existing frozen replay
tests passed. Strict library/all-feature Clippy and workspace formatting passed.
The same pure source extracted with `rustc --test` passed 14 tests. Two processes
produced identical 3,500-byte envelopes matching the independently encoded
literal fixture; eight independent threads matched serial construction.

All tracked crate/source manifests before and after the frozen Cargo matrix
match. Exact commands, source manifests, logs and comparison results are in
`checks.tar.gz`; hashes, platform and exclusions are in `receipt.json`.
The earlier 12-test Cargo run overlaps final additions and is diagnostic only.

Regenerate the independent fixture from the worktree root:

```sh
python3 docs/evidence/prism-semantic-names-v1/generate_expected.py
/home/vmiguel/.cargo/bin/rustup run 1.99.0 cargo test -p rustycad-tkbo --lib owned_request::names --jobs 2
```

This is a bounded review checkpoint. No live correspondence adapter, complete
operation, public projection or release authority is provided. The full TKBO
suite and ThreadSanitizer were not run for this prefix. Existing child replay
bytes are preserved; wrapping arbitrary bytes in this envelope certifies no
geometry. Generic use tokens require full role-bearing compound keys and parent
ancestry before any application-facing semantic projection.
