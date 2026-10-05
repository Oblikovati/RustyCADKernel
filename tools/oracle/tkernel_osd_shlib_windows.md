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

Rust source hashes for this implementation are:

- `shared_library.rs`: `3c6a63101d30d1c541392941f04fc656c17d4d22e17f46c374b68377a87d8c21`
- `shared_library/windows_backend.rs`: `add9a86a3468908062b4d75adb16d620006ee2a13d9511a35d9885d58147d176`
- `crates/tkernel/Cargo.toml`: `1e3bb49bd7104209041bffc9ad419edf888b8766d03bd31b014971d23b4d904c`

Validation used Rust 1.98.1. `cargo test -p rustycad-tkernel` aggregated 2,748
passes across 161 result groups with zero failures or ignored tests. The seven
Windows fake tests also passed directly. Host strict Clippy and Windows GNU
`cargo check` plus strict Clippy passed. Commands:

```sh
cargo test -p rustycad-tkernel
cargo clippy -p rustycad-tkernel --tests -- -D warnings
cargo check -p rustycad-tkernel --target x86_64-pc-windows-gnu
cargo clippy -p rustycad-tkernel --target x86_64-pc-windows-gnu --lib -- -D warnings
```

OCCT's alternate `OCCT_UWP` loader branch (`LoadPackagedLibrary`) is not
implemented or cross-compiled here.

Two Windows details remain outside exact parity. Native error formatting uses
static buffers and leaves old text in place when `FormatMessageW` or UTF-8
conversion fails; the Rust adapter returns an empty byte vector on those
failures. Native `Destroy` leaves its name pointer dangling and `DlClose`
retains its handle value; Rust avoids dangling memory and tracks the closed
state. These differences and the missing native Windows runtime evidence keep
the affected ledger rows partial.
