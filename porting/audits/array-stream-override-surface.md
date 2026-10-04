# Standard_ArrayStreamBuffer override adapter

Audit and native probe are pinned to OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. The native header declares virtual
`Init` and the protected virtual `underflow`, `uflow`, `pbackfail`, `showmanyc`,
`seekoff`, and `seekpos` hooks, plus public virtual `xsgetn`
(`Standard_ArrayStreamBuffer.hxx:60–98`).

The Rust port now exposes those callbacks through `ArrayStreamBufferHooks` and
`ArrayStreamBuffer::with_hooks`. One `ArrayStreamBufferState` owns the borrowed
bytes and cursor and is lent to every callback. The callback results for character
hooks use `i32` so overrides can return a non-byte `int_type`; byte-oriented
conveniences such as `uflow() -> Option<u8>` remain available. `ArrayStreamSeekDirection::Other`
models the native `seekoff` default direction branch (`.cxx:118–123`).

The derived native probe at `tools/oracle/array_stream_override_dispatch.cxx`
checks dispatch through base pointers and standard streambuf calls. It verifies
that overridden `underflow` can mutate the shared cursor, `seekpos` invokes the
virtual `seekoff`, all hook counters increment, and an overridden underflow may
return 300. The Rust dispatch test compares the corresponding receipt.

This is a Rust trait adapter, not a C++ ABI-compatible streambuf or vtable. The
existing `peek` and `available` methods are state inspectors; `BufRead` is a
byte-storage adapter and does not dispatch virtual underflow/showmanyc. Native
`streamsize`/`pos_type` have platform-specific representation; the implemented
hook surface uses Rust `usize`/`isize` and `u64`/`io::Result` adapters for bounded
valid-position calls. It does not claim exact subclasses that return arbitrary
position state. Signed bulk-read requests/results are now covered by
`xsgetn_signed`; see `array-stream-signed-reads.json`. Input pointer arithmetic outside the
backing range remains unsupported because the native implementation has undefined
behavior there (`.cxx:94–116,137–151`).

Reproduce native evidence with GCC 13.3.0 and the pinned build libraries:

```sh
c++ -std=c++17 -O2 \
  -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/array_stream_override_dispatch.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/array_stream_override_dispatch
/tmp/array_stream_override_dispatch | diff - tools/oracle/array_stream_override_dispatch.expected
```
