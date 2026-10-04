# `Standard_CString` formatting subset

The fixture compares typed Rust arguments and injected writer/buffer sinks against the pinned OCCT `Standard_CString.cxx` implementation. Its bounded conversion set is `%c`, `%d`, `%i`, `%o`, `%u`, `%x`, `%X`, `%f`, `%F`, `%e`, `%E`, `%g`, `%G`, `%s`, and bare `%%`, with literal or dynamic (`*`) width/precision and the private formatter's supported flags. `%c` accepts width and left alignment; its precision and other flags are rejected. `Int`, `UInt`, `Real`, and byte-string arguments are explicit. C varargs, `va_list`, and `FILE*` are represented by a typed argument slice, a `Write` sink, and a growable byte buffer; this intentionally does not claim ABI or stdio buffering equivalence. Positional arguments, pointers, wide strings, and length forms outside the existing engine are rejected.

`Sprintf`/`Vsprintf` replace the `Vec<u8>` with the formatted bytes and a trailing NUL. They do not model a fixed caller buffer's capacity, untouched tail, or C `int` count overflow. `Printf`/`Fprintf` return Rust `io::Result<usize>` and inherit the injected sink's error behavior; they do not reproduce C stdio locking, buffering, or stream error state. Unsupported argument or directive combinations return `InvalidInput` before writing.

Native probe:

- Source: `tools/oracle/tkernel_c_string_format_probe.cxx`
- OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`
- `Standard_CString.cxx` SHA-256: `37462215d76cf69012c70e4a9a958a8610c7373cb6bed85208a03ccf3c3f690f`
- `libTKernel.so` SHA-256: `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b`
- Probe SHA-256: `98d5d399acb28f526057e3b467f1df788109e56e9338d7113ec8de3b91ab7d12`
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
