# `OSD_SharedLibrary` C-string prefix oracle

The native constructors copy names with `strlen`/`strcpy`, while `DlSymb` passes the
provided pointer directly to `dlsym`. These operations observe only bytes before the first
NUL. On the pinned Linux build, the probe checks a `libm.so.6\0ignored` path, a
`cos\0ignored` symbol, a leading-NUL symbol, and an empty path prefix. `dlopen("")` resolves
the main program on this glibc build. The ordinary `libm.so.6`/`cos` case is the no-NUL
control.

Rust exposes UTF-8 `&str` inputs, so the change preserves prefix behavior for valid UTF-8
names; arbitrary non-UTF-8 C strings remain outside the adapter's representable input.
Windows loader behavior remains unimplemented. The separate `DlSymb`-after-`DlClose` state
is undefined in native code and remains a safe Rust `None` result.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Source: `FoundationClasses/TKernel/OSD/OSD_SharedLibrary.cxx`.

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_osd_shlib_nul.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-osd-shlib-nul
/tmp/tkernel-osd-shlib-nul | diff -u tools/oracle/tkernel_osd_shlib_nul.expected -
```
