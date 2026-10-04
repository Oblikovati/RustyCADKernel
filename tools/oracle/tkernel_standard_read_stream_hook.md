# `Standard_ReadLineBuffer::readStream(std::istream&)` oracle

This fixture derives from `Standard_ReadLineBuffer` and exposes the protected,
non-virtual `readStream(std::istream&, size_t, size_t&)` overload. It verifies a
full read, a short EOF read, a repeated EOF read, and an already-bad stream.
The final case confirms a false result with count zero and an unchanged buffer.

Pinned OCCT source revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256: `346f21ca3b3e42bff10428aeb4f20c25ec10e5965e90bc532f28d27d1e60c449`.
Oracle `libTKernel.so` SHA-256: `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b`.

Reproduce from the RustyCADKernel checkout (the built OCCT profile is the same
foundation profile used by the root verifier):

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_standard_read_stream_hook.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-standard-read-stream-hook
/tmp/tkernel-standard-read-stream-hook
```

The protected hook is not virtual, so derived classes can call it but cannot
override its dispatch. The Rust callable counterpart accepts `Read` and safely
rejects lengths larger than its allocation; that out-of-range case is outside
the native method's defined buffer domain. The `FILE*` overload has its own
fixture, adapter, and API mapping in `tkernel_standard_read_file_stream.md`.
