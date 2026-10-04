# Stream-buffer integer putback hook

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` compares the full
`std::streambuf::int_type` request to a previous `char` promoted to `int`.
On Linux x86_64, a previous byte 128 accepts request -128 and rejects request 128.
Request -1 is EOF and always backs up when a previous byte exists. The returned
character is unsigned (128), even when the successful request was negative.

The existing Rust `Option<u8>` wrapper preserved the positive-byte cases, but could
not express negative requests other than EOF. `put_back_integer(i32)` now exposes
the protected hook's request domain, and the wrapper delegates to it. Rust uses
the target's `c_char` signedness for the native promotion. The Linux x86_64 fixture
test compares all 13 native rows, including failure preserving position and a
second putback at the beginning. Other targets have portable logical tests but
do not acquire native profile evidence from this Linux receipt.

Reproduce using GCC 13.3.0:

```sh
c++ -std=c++17 -O2 -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/array_stream_integer_putback.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi -lTKernel \
  -o /tmp/array_stream_integer_putback
/tmp/array_stream_integer_putback | diff - tools/oracle/array_stream_integer_putback.expected
cargo test -p rustycad-tkernel --test array_stream_integer_putback
```

This is supplemental API coverage; original legacy declaration counts are unchanged.
