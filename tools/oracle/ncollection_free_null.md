# Nullable NCollection allocator free probe

The probe records the defined native result of `Free(nullptr)` for each public allocator kind.
It uses OCCT source revision `3d097a0328e71b826377d4814ab05ec3c3d23871` and the pinned
`target/occt-tkernel-full` build. The linked `libTKernel.so` SHA-256 is
`ddf94242442e02e57820d048b5296ee5e73ec579ae0d0fd8bddf26c7f8e1aefa`.

The native implementations establish the distinction: Common delegates to `Standard::Free`,
Heap and Aligned delegate to their matching free routines, and Inc explicitly ignores the
pointer. Acc calls `findBlock(nullptr)` and throws `Standard_ProgramError` when no live block
owns it. Rust exposes this input through `BaseAllocator::free_raw_nullable`; its ordinary
`free_raw(NonNull<u8>)` remains the safer interface for owned allocations.

Source hashes: `NCollection_BaseAllocator.cxx`
`413466ceac2f98b21660faf91262ee723bc1c8d936c3ad40d643cd9fc6985db7`,
`NCollection_AccAllocator.cxx`
`8d3b3d580ef2cfa4a2a53f11dc5b9b66ae3771edc7cd5f95126be6ab525a9a55`,
`NCollection_HeapAllocator.cxx`
`a0f935473c16daa130571f9d3f4a7c1e2a6d879862f1ca9eeff2db352d1dc796`,
`NCollection_AlignedAllocator.cxx`
`54f4f5104ee34b8cb41d43908d969c3cd91e6ec9aa48bd06beed3682d334767b`, and
`NCollection_IncAllocator.hxx`
`4fa6b2f70b28a1dfc9c4aa8e4b9a0120b60eb453220c36a390692c6432b7ab63`.

Reproduce from the repository root with:

```sh
c++ -std=c++17 -O2 -Itarget/occt-tkernel-full/include/opencascade \
  tools/oracle/ncollection_free_null.cxx \
  -Ltarget/occt-tkernel-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-tkernel-full/lin64/gcc/libi -lTKernel \
  -o /tmp/ncollection_free_null
/tmp/ncollection_free_null
```
