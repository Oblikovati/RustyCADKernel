# `Standard_MMgrOpt::AllocMemory` and `FreeMemory` protected-hook oracle

The derived-class fixture confirms the in/out size and clear behavior on the pinned Linux
build. With mmap enabled, a `page_size + 1` request returns a two-page size; with mmap
disabled, a 37-byte request keeps size 37. `clear=true` zeroes both results. Freeing only
two pages of a three-page region leaves its final page mapped (including PAGE_ALIGN on
the requested free size). Linux `mincore` confirms the released prefix is unmapped while
the suffix remains mapped. Size zero leaves a mapping unchanged, and heap mode ignores
the size argument.

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256: `9ca7d6b876c018ff937746df766eac2568ab1dbafd8a27b2ba745adfea032a22`.
Source SHA-256: `5bc434cc85e3143f605d0ff852058eab8f168cea1bca8657bfdfc42819359424`.
Expected output is in `tkernel_mmgr_alloc_memory.expected`.

```sh
occt_build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation
kernel="$occt_build/lin64/gcc/libi/libTKernel.so"
c++ -std=c++17 -O2 -I"$occt_build/include/opencascade" \
  tools/oracle/tkernel_mmgr_alloc_memory.cxx "$kernel" \
  -Wl,-rpath,"$(dirname "$kernel")" -o /tmp/tkernel-mmgr-alloc-memory
/tmp/tkernel-mmgr-alloc-memory
```

Rust exposes these operations as an opaque region token over `AddressSpace`. Heap mode
ignores the size argument; Unix mapped mode page-aligns it, calls `munmap` for the released
prefix, rekeys the surviving suffix, and marks the token released when the whole tracked
region is unmapped. A mapped request that would extend beyond the tracked region is rejected
rather than changing adjacent regions. Heap contents when `clear=false`, OS allocation
failure details outside the tested error path, and non-Unix physical mapping backends remain
separate parity obligations. `Initialize` is
represented by `MMgrOpt::new`, which performs the same constructor initialization; calling
the native protected method again leaks/replaces internal resources and is not exposed as a
safe reinitializer.
