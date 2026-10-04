# Derived stream-buffer hook dispatch

This fixture compiles a C++ subclass of `Standard_ArrayStreamBuffer` against
OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`. It overrides virtual
`Init`, all six protected streambuf hooks, and public virtual `xsgetn`. Each
override changes or observes shared cursor state and delegates to the base where
appropriate. The output verifies dispatch through base pointers and the standard
`std::streambuf` entrypoints, including the virtual call from `seekpos` to
`seekoff`.

The Rust adapter models these hooks with `ArrayStreamBufferHooks` and one
`ArrayStreamBufferState`. Rust overrides use safe methods on that state instead
of protected raw pointers. `peek` and `available` remain direct state inspectors;
call `underflow` and `showmanyc` when override dispatch is required.

Reproduce with the pinned native libraries:

```sh
c++ -std=c++17 -O2 \
  -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/array_stream_override_dispatch.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/array_stream_override_dispatch
/tmp/array_stream_override_dispatch | diff - tools/oracle/array_stream_override_dispatch.expected
```

The bounded fixture covers valid storage and positions. Native pointer arithmetic
outside the byte range remains unsupported because it is undefined behavior.
