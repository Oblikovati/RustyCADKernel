# POSIX directory cursor parity

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`OSD_FileIterator::More` opens the directory and calls `Next` only once;
`Next` reads until a matching entry or EOF. `Initialize` closes the previous
stream; `Destroy` deliberately does nothing. The directory iterator uses the
same protocol. Previously Rust eagerly read every name during `More`.

The supplemental C++ probe interposes `opendir`, `readdir`, and `closedir`
against the unchanged native library. It creates two temporary files and
checks lazy consumption, repeated `More`, EOF close, `Destroy`, and restart.
Compile from the repository root (adjust the native build path if necessary):

```sh
g++ -std=c++17 -O0 -g0 tools/oracle/tkernel_directory_cursor.cxx \
  -Itarget/occt-modeling-full/include/opencascade \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -lTKernel -ldl -o /tmp/tkernel_directory_cursor
/tmp/tkernel_directory_cursor
```

The output matches `tkernel_directory_cursor.expected`. Rust tests use named
`FakeDirectoryCursor` and `FakePosixCalls`; their counters prove demand-driven
reads and one close on EOF, reset, or drop. The production Unix cursor owns the
POSIX stream, copies each returned name, and serializes access through a shared
cursor. Safe clones share cursor position but retain independent copied current
names; native copied `dirent` pointers can alias overwritten or freed storage.
No parity claim is made for those unsafe pointer-lifetime observations.

The existing `read_directory` backend method remains available. The new
`open_directory` method defaults to a buffered adapter for existing injected
backends, while Unix overrides it with the lazy stream. Windows behavior remains
partial and this change does not implement its native iterator APIs.

Validation:

- `cargo test -p rustycad-tkernel`: 2566 passed, no failures.
- `cargo clippy -p rustycad-tkernel --all-targets -- -D warnings`: passed.
- `cargo fmt --all`: passed.
- All 120 existing `tkernel_osd_files_corpus.cases()` requests matched the
  native `tkernel_probe` and Rust `oracle_probe` outputs byte for byte.
  Input SHA-256: `909af2e5e8ed907d9d11ab95d17d809c4b6ebc22131e5e91f98d6f7fe004094e`.
  Output SHA-256: `4c27f7c4642a730c294238a2308ed57990b5b33c60b1bf4347d2aef59db99423`.

These are supplemental checks; no legacy test declarations were added or skipped.

## Retained lifecycle difference

The pinned Unix destructors call `Destroy`, which is a no-op, so abandoning an
iterator before EOF leaves its directory stream open. The supplemental probe's
`drop_keeps_native_stream 1` records this and explicitly closes that leaked native
stream afterward. Rust's owned cursor closes on last drop. This resource-lifetime
difference is recorded as partial destructor parity, not silently counted as
matching behavior. Windows `Destroy` also remains outside this Unix implementation.
