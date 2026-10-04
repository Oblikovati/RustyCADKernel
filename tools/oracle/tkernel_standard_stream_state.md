# TKernel stream-state probes

The probes use the pinned OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871` and `libTKernel.so` from `target/occt-foundation`.

Pinned header hashes: `Standard_ReadBuffer.hxx` is `f343381ffd14894aa2b626e6be7d666510a820f43fc4ce0db5f79993fe946178`; `Standard_ReadLineBuffer.hxx` is `346f21ca3b3e42bff10428aeb4f20c25ec10e5965e90bc532f28d27d1e60c449`.

`tkernel_standard_istream_state.cxx` exercises `Standard_ReadLineBuffer::readStream(std::istream&)` with a pre-set `badbit`, `clear()`, a short EOF read, and a repeated read while EOF/fail remain set. The native contract is that badbit makes the hook fail, EOF/fail without badbit do not, and clearing flags permits a new read from the current source position. Rust's `IstreamStream<R>` carries those flags around an injected `Read` source so repeated hook calls do not accidentally recover from `badbit`.

`tkernel_standard_read_buffer_file_stream.cxx` first sets `ferror(FILE*)` through the line-buffer overload, then reads four later bytes through `Standard_ReadBuffer`. It demonstrates that this overload tests only `fread`'s returned count: a full four-byte transfer succeeds while the prior error indicator remains set. The Rust `read_data_chunk_file` adapter uses that count-only rule; `read_stream_file` retains the line-buffer's distinct `ferror` rule.

Rebuild and run either fixture from the repository root:

```sh
kernel=target/occt-foundation/lin64/gcc/libi/libTKernel.so
include=target/occt-foundation/include/opencascade
c++ -std=c++17 -O2 -I"$include" tools/oracle/tkernel_standard_istream_state.cxx "$kernel" -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-standard-istream-state
/tmp/tkernel-standard-istream-state
c++ -std=c++17 -O2 -I"$include" tools/oracle/tkernel_standard_read_buffer_file_stream.cxx "$kernel" -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-standard-read-buffer-file-stream
/tmp/tkernel-standard-read-buffer-file-stream
```
