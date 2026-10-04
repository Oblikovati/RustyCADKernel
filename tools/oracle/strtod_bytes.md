# `Standard_CString::Strtod` byte-domain fixture

The pinned C++ declaration takes `const char*` and `char**`; it accepts byte strings without a UTF-8 requirement. Rust's prior `strtod(&str)` API could not represent inputs containing invalid UTF-8, even when such bytes occur after a valid numeric prefix. `strtod_bytes(&[u8])` now shares its parser with the string API and preserves C NUL termination and byte offsets.

Fixture rows in `strtod_bytes.expected` contain input hex, native binary64 bits, and the native end-pointer offset. The native executable was built from OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871` with:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-foundation/include/opencascade \
  tools/oracle/strtod_probe.cxx \
  target/occt-foundation/lin64/gcc/libi/libTKernel.so \
  -Wl,-rpath,target/occt-foundation/lin64/gcc/libi \
  -o /tmp/tkernel-strtod-native-probe
```

Native source SHA-256: `b89543c6e50422513a6c2eb8fffe7cf5b7e8c20ea0fd96eb0e39a7e940b531c3`
Native `libTKernel.so` SHA-256: `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b`
Native probe SHA-256: `5a8c4e5f303a6a444e5da76e37f7f3eeac6d819c1fd17146a13de3f47fb18443`

The high bytes in the first three rows are suffix bytes (or occur after NUL) and must not affect numeric conversion. The final row begins with an invalid UTF-8 byte and follows native no-conversion behavior.
