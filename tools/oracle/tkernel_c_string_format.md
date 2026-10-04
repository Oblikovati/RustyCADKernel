# `Standard_CString` formatting subset

The fixture compares typed Rust arguments and injected writer/buffer sinks against the pinned OCCT `Standard_CString.cxx` implementation. Its bounded conversion set is `%c`, `%d`, `%i`, `%o`, `%u`, `%x`, `%X`, `%f`, `%F`, `%e`, `%E`, `%g`, `%G`, `%s`, `%a`, `%A`, and bare `%%`, with literal or dynamic (`*`) width/precision and the private formatter's supported flags. `%c` accepts width and left alignment; its precision and other flags are rejected. Integer arguments use `Int`/`UInt`, platform-width `Long`/`ULong`, or 64-bit `LongLong`/`ULongLong` according to `h`/`hh`/`l`/`ll` lengths; `z` and `t` use portable `isize`/`usize` pairs for `size_t` and `ptrdiff_t`. The `z`/`t` native boundary row records Linux x86-64; a Rust value-derived test checks aliases on other pointer widths, but other native ABI profiles are unproven. C varargs, `va_list`, and `FILE*` are represented by typed argument slices, a `Write` sink, and byte buffers; this intentionally does not claim ABI or stdio buffering equivalence. Positional arguments, pointers, and wide strings remain unsupported.

`Sprintf`/`Vsprintf` growable-`Vec` variants replace the vector with formatted bytes plus NUL. The `_into_slice` variants write into a sufficiently large caller slice, add the terminator, and preserve bytes after it exactly like native `Sprintf`/`Vsprintf`; a too-small slice is rejected before mutation. Native overflow of an undersized C buffer is outside the safe Rust domain, and C `int` count overflow remains partial. `Printf`/`Fprintf` return Rust `io::Result<usize>` and inherit the injected sink's error behavior; they do not reproduce C stdio locking, buffering, or stream error state. Unsupported argument or directive combinations return `InvalidInput` before writing.

Native probe:

- Source: `tools/oracle/tkernel_c_string_format_probe.cxx`
- OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`
- `Standard_CString.cxx` SHA-256: `37462215d76cf69012c70e4a9a958a8610c7373cb6bed85208a03ccf3c3f690f`
- `libTKernel.so` SHA-256: `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b`
- Probe SHA-256: `0ef404d7ec0dc0d29da4a608e53a34d0a6d2747c7a550a703cb9d9c45d22b29d`
- The `long`/`unsigned long` expected row records the Linux LP64 build profile. Rust tests compare those aliases using the target's `c_long`/`c_ulong` widths on every platform; Windows LLP64 native bytes are not covered by this fixture.
- Build/run: see the command below; outputs are in `tkernel_c_string_format.expected`.

```sh
c++ -std=c++17 -O2 \
  -I target/occt-foundation/include/opencascade \
  tools/oracle/tkernel_c_string_format_probe.cxx \
  target/occt-foundation/lin64/gcc/libi/libTKernel.so \
  -Wl,-rpath,target/occt-foundation/lin64/gcc/libi \
  -o /tmp/tkernel-c-string-format-native
/tmp/tkernel-c-string-format-native
```

The separate `tkernel_hex_float.cxx` fixture supplies 1,312 supplemental hexadecimal-double cases, including default/explicit precision, ties, subnormals, signed zeros, special values, casing, prefixes and padding. Source and output hashes are in `porting/audits/standard-hex-float.json`. This matches the Linux glibc C-locale profile at default nearest/even rounding; alternative rounding modes and C libraries remain unproven.
