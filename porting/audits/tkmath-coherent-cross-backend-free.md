# Coherent ring cross-backend release

`Poly_CoherentNode::AddTriangle` allocates through the supplied allocator, while
`RemoveTriangle` and `Clear` call `Free` on whichever allocator the caller supplies.
The pinned OCCT build accepts both Common-to-Heap and Heap-to-Common release for
these ring nodes. Its Common path uses `Standard::Allocate`/`Standard::Free`; the
pinned Linux build uses the same process heap as `NCollection_HeapAllocator`.
Its `Standard.cxx` compile command defines no `OCCT_MMGT_OPT_FLEXIBLE`,
`OCCT_MMGT_OPT_TBB`, or `OCCT_MMGT_OPT_JEMALLOC`, selecting the source's `calloc`
and `free` branch. A native build using one of those custom managers is not covered.

Rust stores enough allocation layout metadata to deallocate Common and Heap blocks
through the same `heap_free` implementation. `BaseAllocator::can_free_from` now
recognizes this cross-policy compatibility. Incremental and accumulating allocators
remain incompatible across identities because they own arena/accounting state.
This establishes only the Common/Heap pairing in the configured native build; it
does not establish compatibility for custom OCCT memory-manager configurations or
other allocator subclasses.

The native probe and `coherent_allocator_behavior` test allocate a node member
through each policy, remove it through the other policy, and verify the ring is
empty. Existing incremental mismatch tests still verify that an incompatible arena
is rejected without mutating the ring.

The affected source declarations are `Poly_CoherentNode::AddTriangle(const
Poly_CoherentTriangle&, const occ::handle<NCollection_BaseAllocator>&)`,
`RemoveTriangle(const Poly_CoherentTriangle&, const
occ::handle<NCollection_BaseAllocator>&)`, and `Clear(const
occ::handle<NCollection_BaseAllocator>&)`. Their Rust counterparts are
`CoherentNode::{add_triangle_in,remove_triangle_in,clear_in}`. The ring release
mechanisms are `Poly_CoherentTriPtr::Remove(Poly_CoherentTriPtr*, const
occ::handle<NCollection_BaseAllocator>&)` and `Poly_CoherentTriPtr::RemoveList(
Poly_CoherentTriPtr*, const occ::handle<NCollection_BaseAllocator>&)`; Rust's
checked counterpart is `CoherentTriPtr::{remove_member,remove_list_in}`. The
affected allocator predicate is `BaseAllocator::can_free_from`.

## Reproduction

```sh
occt=target/occt-tkernel-full
c++ -std=c++17 -O2 -I"$occt/include/opencascade" \
  tools/oracle/tkmath_coherent_cross_backend_free.cxx \
  -L"$occt/lin64/gcc/libi" -Wl,-rpath,"$occt/lin64/gcc/libi" \
  -lTKMath -lTKernel -o /tmp/tkmath_coherent_cross_backend_free
/tmp/tkmath_coherent_cross_backend_free
```

Pinned native output: `tools/oracle/tkmath_coherent_cross_backend_free.expected`.

## Pinned source hashes

| File | SHA-256 |
| --- | --- |
| OCCT `Poly_CoherentNode.hxx` | `43fb34c629691f23c8345f450a741238aa0a4e0947719e5406b789baf39ce254` |
| OCCT `Poly_CoherentNode.cxx` | `03f46cb19ea098227ed6b4bbc1e0b396004774d1486c689dcf744fa73615fbb1` |
| OCCT `Poly_CoherentTriPtr.cxx` | `94c8114cd73f61b4df39ab335ac530b0a4f1bd00390d553c0f772894bd4f7eba` |
| OCCT `Standard.cxx` | `7c50331e36a64ca1aaff7005536bb7b98cb627060edb2049362418d2c7a0dfee` |
| OCCT `NCollection_HeapAllocator.cxx` | `a0f935473c16daa130571f9d3f4a7c1e2a6d879862f1ca9eeff2db352d1dc796` |
| Native probe | `57df3e6e26fa55fa6104637c0c0c4d25b76efc015c1855767262b4cfb58454b2` |
