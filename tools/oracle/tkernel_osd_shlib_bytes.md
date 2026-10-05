# `OSD_SharedLibrary` native byte-name oracle

This probe creates a temporary symlink with a `0xff` filename byte to a known library,
loads it through `OSD_SharedLibrary`, then asks `dlsym` for a missing symbol containing
`0xff`. It prints the loader error as hex so the non-UTF-8 byte survives the evidence
artifact. The corresponding Rust test exercises the same symlink, preserves the raw name
with `name_bytes`, and compares `dl_error_bytes` with the pinned glibc message.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Source: `FoundationClasses/TKernel/OSD/OSD_SharedLibrary.cxx`.
The source hash is `c1db312f5f4e1214ed25ed2a15d1b81fc9b86ff003bc49b7d8b5436b50cfbcb2`;
the tested `libTKernel.so` hash is `d63309c5c80b99ea9a95baf9ed549fa2dd417f159c5f0cbe598cb4a282bd995b`.
Oracle source hash: `a58c87de708fe170cd126fa33342bc6f65bdba7e851f6f20e2ef2170ce301169`;
expected output hash: `62b3a64f33046cfecc6822236df5125191a56763e0ce82488820b94575313bf0`.

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_osd_shlib_bytes.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-osd-shlib-bytes
/tmp/tkernel-osd-shlib-bytes /lib/x86_64-linux-gnu/libm.so.6 | diff -u \
  tools/oracle/tkernel_osd_shlib_bytes.expected -
```

Rust's `name()` and `dl_error()` remain UTF-8 convenience views; invalid bytes produce
`None` for `name()` and lossy text for `dl_error()`. The byte methods are authoritative for
Unix filenames, symbols, and loader diagnostics. Windows loader behavior and native
post-close dangling-handle behavior remain outside this parity claim.
