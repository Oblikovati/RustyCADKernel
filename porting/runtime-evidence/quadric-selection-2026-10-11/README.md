# Martensite analytic quadric selection receipt

Base main: `157a46e5e79465c91d89d9b7b516d74aeca94707`.
Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The directory date identifies the batch start; final checks completed 2026-10-12.

This bounded slice credits eighteen public, API-visible captured declarations:
Circle's two constructors and matching/connected/bounds/center operations;
Cylinder's constructor and those four operations; Sphere's constructor and those
four operations plus its live detected-point reference and reset. No private
helpers, copied-point convenience, remaining property getters or class-wide
completion are credited. `claim-audit.json` verifies unique pinned declaration
IDs, source hashes, default port scope and eighteen unique canonical Rustdoc
paths using nightly-2026-09-25 format 61 and the project's normalizer.

The native differential corpus contains 4,140 real selecting-volume snapshots,
30 virtual-volume fault/repair snapshots and 11 retained-reference snapshots.
All three native generators repeated byte-identically. Nine bounded chunks
reconstruct the complete real-volume corpus. Fixture provenance records native
source/generator hashes and exception-enabled existing OCCT library settings.
No OCCT library rebuild, GPU submission, Windows/macOS run or CI oracle was used.

Checks completed:

- `cargo test -p rustycad-tkv3d --all-targets --locked`: 663 tests passed.
- `cargo clippy -p rustycad-tkv3d --all-targets --locked -- -D warnings`: passed.
- Repository Rust formatting check, toolkit discovery and 13 discovery regressions: passed.
- Pinned nightly Rustdoc capture and canonical path normalization: eighteen of eighteen found.
- Focused function/file bounds, complete fixture/source hash audit and `git diff --check`: passed.

Compact builds used debug level zero, incremental disabled and two jobs.
Compressed logs preserve original bytes; `log-hashes.json` records compressed
and uncompressed hashes. Rustdoc reports four inherited warnings: two TKMath
future standard-library name collisions and two pre-existing TKV3d macro links.
Stable Clippy passed with warnings denied. Source behavior is covered by three
Rust-only replay tests; all fixtures load without native OCCT or external I/O.

Local native reproduction reuses the existing checked OCCT libraries:

```sh
native_root=../RustyCADKernel/target/occt-all-full
native_lib="$native_root/lin64/gcc/libi"
for suite in selection faults reference; do
  c++ -std=c++17 -O2 -I"$native_root/include/opencascade" \
    "tools/oracle/martensite_sensitive_quadric_${suite}.cxx" \
    -L"$native_lib" -Wl,-rpath,"$native_lib" \
    -lTKV3d -lTKService -lTKMath -lTKernel -o "/tmp/quadric-$suite"
  "/tmp/quadric-$suite" > "/tmp/quadric-$suite.txt"
done
```

Behavior notes and exact exclusions are in `docs/sensitive-quadric-selection.md`.
Arbitrary malformed radii/heights, singular transforms, invalid native pointers
and references dangling after destruction remain unclaimed. Native connected
cylinders dropping hollow mode and sphere reset/failure assignment order are
preserved deliberately. Safe borrowed aliases permit selection/reset mutation
without retaining a lock and keep connected detection state independent.
