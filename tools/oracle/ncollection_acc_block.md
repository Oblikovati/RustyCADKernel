# AccAllocator protected Block probe

This derived-class fixture exposes the protected nested `Block` and `AlignedSize` types and
calls `SetFreeSize`, `FreeSize`, `Allocate`, `Free`, and `IsEmpty`. The OCCT source revision is
`3d097a0328e71b826377d4814ab05ec3c3d23871`; `NCollection_AccAllocator.hxx` SHA-256 is
`5fe1b00032b8b1493a18d1a8c778410f929cb25a010757c15028d02fa490c585`. The linked
`libTKernel.so` SHA-256 is `ddf94242442e02e57820d048b5296ee5e73ec579ae0d0fd8bddf26c7f8e1aefa`.

Native initializes `allocStart` at `address + size`, rounded down to four-byte alignment.
`Allocate(AlignedSize(5))` rounds to eight, moves the cursor down, and increments the count;
`Free` decrements it. All pointer arithmetic stays inside one aligned byte array. Rust's
`AccBlock` preserves these value operations using integer address identities and never
 dereferences them. The type exposes the protected descriptor behavior; it does not provide
allocator inheritance, access to OCCT's block map, or actual allocator-owned block pointers.

The captured native output is `ncollection_acc_block.expected`. Reproduce from the repository
root with:

```sh
c++ -std=c++17 -O2 -Itarget/occt-tkernel-full/include/opencascade \
  tools/oracle/ncollection_acc_block.cxx \
  -Ltarget/occt-tkernel-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-tkernel-full/lin64/gcc/libi -lTKernel \
  -o /tmp/ncollection_acc_block
/tmp/ncollection_acc_block
```

The Rust regression is `ncollection_acc_block_behavior.rs`; it checks the defined in-block
cursor and count transitions against the captured native values.
