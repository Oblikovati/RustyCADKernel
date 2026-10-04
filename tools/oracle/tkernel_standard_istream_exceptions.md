# `Standard_ReadLineBuffer` and `std::istream` exception-mask oracle

Oracle probe SHA-256: `e8596e8e8de2e2879d2bd1e756110606b477f82ee3a08ddcfaef8708f166efd7`.

Pinned OCCT source revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`Standard_ReadLineBuffer.hxx` SHA-256: `346f21ca3b3e42bff10428aeb4f20c25ec10e5965e90bc532f28d27d1e60c449`.
`Standard_ReadBuffer.hxx` SHA-256: `f343381ffd14894aa2b626e6be7d666510a820f43fc4ce0db5f79993fe946178`.
Oracle `libTKernel.so` SHA-256: `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b`.

The fixture verifies nine transitions. Both masked and unmasked streambuf exceptions write two bytes into the supplied buffer. Native `gcount()` is zero in both cases; masked reads throw before `readStream` assigns its output count, while the unmasked hook returns false and assigns zero. A first badbit read under a `failbit`-only mask does not throw because badbit and failbit are separate rdstate bits; the next sentry adds failbit and throws before reading. A zero-length read still checks an already-failed stream state. A `failbit`-masked short read sets `eofbit|failbit`, exposes the two-byte `gcount()`, but also leaves the hook's output count untouched. Setting an exception mask over an already-set bit stores the mask before throwing. `clear(state)` similarly installs the state before throwing, while `clear()` to `goodbit` recovers without throwing. Finally, a thrown `ReadLine` read zeroes its output length/count first, preserves the reader's prior cursor/count state, and keeps partial buffer writes.

Reproduce with the pinned foundation profile:

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_standard_istream_exceptions.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-standard-istream-exceptions
/tmp/tkernel-standard-istream-exceptions
```

Expected output is in `tkernel_standard_istream_exceptions.expected`. Rust exposes fallible `_istream` methods for `ReadLineBuffer` and `ReadBuffer`, plus `IstreamStream::read_block`; their generic `Read` convenience methods remain available but cannot surface C++-style exception-mask output-assignment ordering.
