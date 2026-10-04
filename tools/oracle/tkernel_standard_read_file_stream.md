# `Standard_ReadLineBuffer::readStream(FILE*)` oracle

This fixture derives from `Standard_ReadLineBuffer` to expose the protected FILE overload.
A `fopencookie` stream returns one byte, raises EIO, later returns four bytes, then EOF.
It captures partial count/buffer contents, confirms that `ferror` stays set while later
`fread` calls still read, and checks repeated EOF with both status indicators set.
The Rust adapter `FreadStream` mirrors these indicators without exposing a C `FILE*`.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256: `346f21ca3b3e42bff10428aeb4f20c25ec10e5965e90bc532f28d27d1e60c449`.
Expected output is in `tkernel_standard_read_file_stream.expected`.

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_standard_read_file_stream.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-standard-read-file-stream
/tmp/tkernel-standard-read-file-stream
```

This is a distinct overload from `readStream(std::istream&)`. Their bounded read behavior
shares Rust's line buffer, while FILE status flags are modeled by `FreadStream`. A request
larger than the allocated buffer is rejected in Rust; native behavior for that request is
outside the defined buffer domain.
