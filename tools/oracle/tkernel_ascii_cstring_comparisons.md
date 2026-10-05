# ASCII string C-pointer comparison overloads

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

The first 20 oracle rows are a four-by-five matrix. Stored strings are empty,
`a`, the full span `a\0b`, and byte `0xff`. Candidates are null, empty, `a\0tail`,
`b`, and byte `0xff`. Each row contains `IsEqual`, `IsDifferent`, `IsLess`,
`IsGreater`, and `StartsWith`, in that order. The final row distinguishes the
char-array overload from a full string view and checks literal suffix matching.

Pointer arguments stop at the first NUL. Null equality and StartsWith match
only empty stored strings; a non-null empty prefix matches every stored string.
Explicit-length and string-view overloads retain all bytes. Rust provides
nullable `*_c_string` methods and `ends_with_literal` for these distinct domains.
The `PartialEq<&str>` adapter now uses C-string equality, as its mapped native
operator does. Byte-span methods retain their full-span semantics.

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_ascii_cstring_comparisons.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -o /tmp/ascii-cstring-oracle
LD_LIBRARY_PATH=target/occt-modeling-full/lin64/gcc/libi \
  /tmp/ascii-cstring-oracle > /tmp/ascii-cstring.actual
diff -u tools/oracle/tkernel_ascii_cstring_comparisons.expected /tmp/ascii-cstring.actual
cargo test -p rustycad-tkernel --locked
```

The receipt is `porting/audits/ascii-c-string-comparisons.json`. Nineteen existing
API mappings are redirected; no original test declarations are added. C++
char-array selector resolution remains a separate compiler-crosswalk obligation.
Arrays without a NUL terminator are outside the native strlen-defined domain;
Rust safely limits reads to the provided byte span.
