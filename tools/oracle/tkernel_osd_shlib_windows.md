# `OSD_SharedLibrary` Windows loader audit

The pinned Windows implementation in `OSD_SharedLibrary.cxx` uses
`GetModuleHandleW` for a cached basename, `LoadLibraryExW` with
`LOAD_WITH_ALTERED_SEARCH_PATH`, `GetProcAddress`, `FreeLibrary`, and
`FormatMessageW` followed by `WideCharToMultiByte(CP_UTF8)`. The Rust Windows
backend follows this call order through an injected `WindowsCalls` seam. The
Linux unit tests use `FakeWindowsCalls`; the backend also cross-compiles for
`x86_64-pc-windows-gnu`. No Windows runtime oracle was available, so these
checks do not establish native Windows execution parity.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Source hashes:

- `OSD_SharedLibrary.cxx`: `c1db312f5f4e1214ed25ed2a15d1b81fc9b86ff003bc49b7d8b5436b50cfbcb2`
- `OSD_SharedLibrary.hxx`: `7056c7ca0d05c5b787569bba3a4b95b1049465a30c094a6d854f17dbb428799e`
- `OSD_Path.cxx`: `431435518fb608b406a24f1d62efee553fc2aaf77e39d2d71e8e039c6fb0d5fc`
- `TCollection_ExtendedString.cxx`: `bdcbe42ede4a95a5b1928ad37cc7388e9d5ce2fb84f32f78e24a62c7681fc22c`

The fake tests cover cached-module reuse, wide path conversion, symbol NUL
prefix handling, close forwarding, failure-code retention, and a successful
load after an earlier error. The implementation intentionally keeps Rust's
safe null-name behavior: `DlOpen` returns false instead of reproducing the
native null-pointer conversion exception. Rust `Destroy` clears its owned name
instead of leaving the native dangling pointer, and closed handles are guarded
from later symbol lookup.

Rust source hashes after the static-buffer follow-up are:

- `shared_library.rs`: `4d141d003ad2c4f24319bd8bb9babafcb492fa4610eeb985b6c0545ccdc4b0a7`
- `shared_library/windows_backend.rs`: `252c6a0bc07d7c87cc42166080dc05c1d975365ef8cc11a0bb14d79d282fad99`
- `shared_library/windows_backend/tests.rs`: `5414bf6fad83dede188cc5d26b01656a6ef9467c48597e9a0c8ef228433b14e8`
- `crates/tkernel/Cargo.toml`: `1e3bb49bd7104209041bffc9ad419edf888b8766d03bd31b014971d23b4d904c`

Validation used Rust 1.98.1. `cargo test -p rustycad-tkernel` aggregated 2,757
passes across 162 result groups with zero failures or ignored tests. The ten
Windows fake tests also passed directly. Host strict Clippy and Windows GNU
`cargo check` plus strict Clippy passed. Raw logs and a source-bound receipt are
stored under `porting/evidence/tkernel-osd-shlib-windows/`. Commands:

```sh
cargo test -p rustycad-tkernel
cargo clippy -p rustycad-tkernel --tests -- -D warnings
cargo check -p rustycad-tkernel --target x86_64-pc-windows-gnu
cargo clippy -p rustycad-tkernel --target x86_64-pc-windows-gnu --lib -- -D warnings
```

OCCT's alternate `OCCT_UWP` loader branch (`LoadPackagedLibrary`) is not
implemented or cross-compiled here.

The later error-buffer follow-up stores the 1024-wide-unit and 1024-byte output
buffers behind a mutex and passes those persistent buffers directly to the
formatter and converter, ignoring both return values as native does. Named fake
tests mutate each destination before reporting failure, proving partial writes
remain visible, and cover buffer-capacity boundaries. Native `Destroy` still
leaves its name pointer dangling and `DlClose` retains its handle value; Rust
avoids dangling memory and tracks the closed state. Windows runtime evidence is
still unavailable, and UWP remains unimplemented, so the affected ledger rows
stay partial.
