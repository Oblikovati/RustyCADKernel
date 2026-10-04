# AccAllocator protected alignment helper probe

This probe derives from `NCollection_AccAllocator` and exposes its protected `AlignedSize` and
`AlignedPtr` types. OCCT source revision is `3d097a0328e71b826377d4814ab05ec3c3d23871`;
`NCollection_AccAllocator.hxx` SHA-256 is
`5fe1b00032b8b1493a18d1a8c778410f929cb25a010757c15028d02fa490c585`. The linked
`libTKernel.so` SHA-256 is
`ddf94242442e02e57820d048b5296ee5e73ec579ae0d0fd8bddf26c7f8e1aefa`.

Native `AlignedSize` rounds up to 4 bytes with unsigned `size_t` wrap (`SIZE_MAX` becomes zero).
Native `AlignedPtr` masks its input address down to a 4-byte boundary and supports rounded
addition/subtraction and compound assignments. The pointer arithmetic fixture stays within one
aligned array, so every C++ operation is defined. Rust mirrors the helper value types; pointer
operations use `wrapping_add`/`wrapping_sub` and the differential test only asserts the same
in-allocation cases. The native compound operators return the updated pointer by value; the Rust
`wrapping_*_assign` methods preserve access to that value alongside idiomatic `+=`/`-=` support.
This does not claim Rust allocator inheritance or expose native `Block` storage internals.

Reproduce from the repository root with:

```sh
c++ -std=c++17 -O2 -Itarget/occt-tkernel-full/include/opencascade \
  tools/oracle/ncollection_acc_aligned_helpers.cxx \
  -Ltarget/occt-tkernel-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-tkernel-full/lin64/gcc/libi -lTKernel \
  -o /tmp/ncollection_acc_aligned_helpers
/tmp/ncollection_acc_aligned_helpers
```
