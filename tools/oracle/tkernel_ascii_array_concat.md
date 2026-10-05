# ASCII string array concatenation

OCCT's four `const char (&)[N]` concatenation overloads delegate to `strlen` in
`TCollection_AsciiString.lxx`. A valid NUL-terminated array therefore appends
only its prefix before the first NUL. The Rust `&[u8; N]` adapter now follows
that rule; `&[u8]` remains a length-delimited span and preserves embedded NULs.

Pinned OCCT source: revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The native header SHA-256 is
`5ce11a239588c7bcdfe62652b4152ea0f283207909c125772ed737e833fe6e3c`; the
inline implementation SHA-256 is
`47aa5de12a3e74380006b3d31d43bbbb41cc23bab8529f7c8cc99fc8f11d8188`.

Rebuild and compare the oracle output:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_ascii_array_concat.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKernel -o /tmp/tkernel_ascii_array_concat
/tmp/tkernel_ascii_array_concat > /tmp/tkernel_ascii_array_concat.out
diff -u tools/oracle/tkernel_ascii_array_concat.expected \
  /tmp/tkernel_ascii_array_concat.out
```

The native result is `headx` for AssignCat, `operator+=`, Cat, and `operator+`
with `{'x', '\0', 'y', '\0'}`. The pointer-plus-length overload still returns
all seven bytes of `headx\0y`. Rust regression
`array_concatenation_stops_at_nul_but_spans_keep_their_length` checks the same
four overload adapters and explicit-span behavior.

The Rust array adapter is bounded by the array extent. Inputs lacking a NUL
inside the extent do not reproduce native `strlen` reading beyond the object;
that native case is outside this verified domain.

## Rust validation

Base revision: `53db08add0c36cce10d4e2ead6f633dc7817d78d`. Compiler: Cargo
1.98.1 (`797e8a9bca276c1c9f9f738d2a20f484fa4eea9d`), rustc 1.98.1
(`48a229cea 2026-09-01`). The full `rustycad-tkernel` package suite passed
2,777 tests across 167 result groups, with no failures or ignored cases; the
merged doctests completed in 0.63 seconds. Strict all-target Clippy passed.
The complete outputs are archived at
`porting/runtime-evidence/tkernel-ascii-array-concat-tests.tar.gz`.
Archive SHA-256:
`f728997c89b3f20518dbccea67253dd0ec63366d9d00809cec1daca8e82c7afc`.

The validated Rust concatenation adapter source hashes are
`d6bc7f036fb73f88dd69578d1a19bb58fe6d4d221d552aa892c722a5fb97ef0f`
(`concatenation.rs`) and
`eac39057dd656685ca6fde647a98418b8554a346bf573fd96085bf3274e651ba`
(`traits.rs`). The regression fixture hash is
`ff368cff2d7f0578481ebe97b16105f5336efd92e0d9bb61ab535a4fa6cdde34`.

Commands used:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.98.1 test -p rustycad-tkernel --quiet
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo +1.98.1 clippy -p rustycad-tkernel --all-targets -- -D warnings
```

Focused test: `array_concatenation_stops_at_nul_but_spans_keep_their_length`.
