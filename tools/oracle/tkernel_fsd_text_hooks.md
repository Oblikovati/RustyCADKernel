# `FSD_File` text-hook oracle

The fixture derives from `FSD_File` and `FSD_CmpFile`, exposes their protected
primitives, then exercises character/word/line/string reads, extended-line read
and write, virtual flush/tag lookup, and the compatible format's four overrides.
The output uses hex for byte strings so carriage returns and line feeds are
visible. `CmpDialect` dispatch matches the native virtual overrides.

Pinned OCCT source: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

| Source | SHA-256 |
| --- | --- |
| `FSD_File.hxx` | `175ceb78d367166624bf87f96ffc6b577744da43e43ab4e95aca87339648ab69` |
| `FSD_CmpFile.hxx` | `a2d976ce0d4d32429aa3b03b2617c9986ed58e37c69a21dc2f36b046f2e95ee4` |
| `FSD_File.cxx` | `52a2a856d67b53ddf027af9fb73f4ba9c7e17538e019237397df345bbd8053a5` |
| `FSD_CmpFile.cxx` | `cff486f740d492e8f856d2c1b0dad6b46f8d5a822b94141704b2572e94f4332d` |
| Oracle `libTKernel.so` | `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b` |

Reproduce against the foundation profile:

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_fsd_text_hooks.cxx \
  "$occt_build/lin64/gcc/libi/libTKernel.so" \
  -Wl,-rpath,"$occt_build/lin64/gcc/libi" -o /tmp/tkernel-fsd-text-hooks
/tmp/tkernel-fsd-text-hooks
```

These methods are protected in C++; Rust makes the corresponding text primitives
callable on `TextFile<D>`. `FsdDialect` and `CmpDialect` preserve the native
virtual selection for line/string/extended-line operations. The rows remain
partial because a generic Rust dialect is not C++ subclass override dispatch.
