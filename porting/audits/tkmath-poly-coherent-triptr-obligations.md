# `Poly_CoherentTriPtr` allocator and identity obligations

This is a bounded gap audit for the coherent triangle-ring API at pinned OCCT
revision `3d097a0328e71b826377d4814ab05ec3c3d23871`. It does not claim parity for
`Poly_CoherentTriPtr`, `Poly_CoherentNode`, or the coherent triangulation package.

## Observable native contract

The native ring is made of individually allocated doubly linked nodes. Each node
stores a triangle pointer plus `Next` and `Previous` node pointers. `Append` and
`Prepend` allocate one node with the supplied `NCollection_BaseAllocator`;
`Remove` unlinks and frees one node through the supplied allocator; `RemoveList`
frees the circular list. `Iterator::PtrValue` identifies a node, so an unrelated
insertion or removal does not change the identity of surviving members.

`tools/oracle/tkmath_coherent_triptr_alloc.cxx` uses a counting subclass of
`NCollection_BaseAllocator`. It allocates the head plus two members, retains the
pointer to the appended member, prepends another node, removes that other node,
and reads the retained member before removing the remaining list. The pinned
optimized OCCT build prints:

```text
3 1 1 1 4
3 3
7 1 4
```

These rows mean three allocations and one free after the single-member removal;
the saved member address is still found in the ring after both insertion and
removal and still names triangle node `4`; and `RemoveList` brings the final
allocation/free totals to three each.

## Rust crosswalk and current coverage

| Native declaration | Rust entry | Status |
| --- | --- | --- |
| `Poly_CoherentTriPtr(const Poly_CoherentTriangle&)` | `CoherentTriPtr::new(usize)`, `new_in(usize, BaseAllocator)` | Still stores triangle indexes because Rust triangulation storage is index-addressed. The custom allocator overload is implemented. |
| `GetTriangle() -> const Poly_CoherentTriangle&` | `triangle(CoherentTriMember) -> Result<usize, Failure>` | Stable member ownership is represented; a Rust borrow into the separate triangulation array is not. |
| `SetTriangle(const Poly_CoherentTriangle*)` | `set_member_triangle(CoherentTriMember, usize)` | Null pointers are intentionally not representable in safe Rust. |
| `Next()`, `Previous()` return member references | `next_member`, `previous_member` | Stable token links are implemented; positional `next`/`previous` remain compatibility conveniences. |
| `Append(const Poly_CoherentTriangle*, Handle(BaseAllocator))` | `append_member(member, triangle, allocator)` | Each member is a real `AllocatedBox` allocation from the supplied allocator. |
| `Prepend(const Poly_CoherentTriangle*, Handle(BaseAllocator))` | `prepend_member(member, triangle, allocator)` | Each member is a real `AllocatedBox` allocation from the supplied allocator. |
| `Remove(Poly_CoherentTriPtr*, Handle(BaseAllocator))` | `remove_member(member, &allocator)` | Unlinks stable token and frees its allocation; mismatched allocator is rejected safely. |
| `RemoveList(Poly_CoherentTriPtr*, Handle(BaseAllocator))` | `remove_list_in(&allocator)` | Clears all nodes if all allocations match; ordinary `Drop` also frees each node through its recorded allocator. |
| `Iterator::PtrValue() -> const Poly_CoherentTriPtr&` | `CoherentTriPtrIterator::member_token()` | Stable node identity is returned as an opaque token backed by the allocated node's stable address. |

`CoherentTriPtr` now owns actual allocator-backed nodes using `BaseAllocator::allocate_value` and `AllocatedBox`; each node contains stable next/previous member tokens. No unsafe pointers are dereferenced. Legacy positional methods still route through this circular node storage. The Rust poly test target covers stable tokens, previous/set/iterator operations, dead-token errors, mismatched-allocator non-mutation, and independent empty/populated clones. A focused unit test verifies bulk removal drains head-then-next like native. The native allocator probe records frees as `7 1 4`: explicit removal of triangle 7, then `RemoveList` traversing the remaining head and its next member.

Out-of-range positional lookups reject the index before walking the ring, including `usize::MAX`; a supplemental regression covers both `triangle_at` and `iter_from`.

The explicit removal methods currently validate `BaseAllocator` handle identity. That is a conservative ownership rule, not an exact model of every native `Free` combination: different native allocator objects can share a compatible heap backend. The node owner always frees through its creating Rust allocator, so this check prevents releasing memory through an incompatible arena. Exact native backend-compatibility semantics remain open.

The broader native node/mesh allocator threading remains incomplete. `Poly_CoherentNode::AddTriangle`, `RemoveTriangle`, and `Clear` receive an allocator at each call, while existing Rust `CoherentNode` methods still use the default common allocator through compatibility methods. `Poly_CoherentTriangulation` also retains its constructor allocator in `myAlloc` and passes it to node operations; Rust does not yet expose this allocator constructor/propagation. Consequently this slice is not a parity claim for the surrounding node or triangulation classes.

## Reproduction

From the RustyCADKernel root, with the pinned OCCT build present:

```sh
occt=target/occt-tkernel-full
c++ -std=c++17 -O2 -I"$occt/include/opencascade" \
  tools/oracle/tkmath_coherent_triptr_alloc.cxx \
  -L"$occt/lin64/gcc/libi" \
  -Wl,-rpath,"$occt/lin64/gcc/libi" -lTKMath -lTKernel \
  -o /tmp/tkmath_coherent_triptr_alloc
/tmp/tkmath_coherent_triptr_alloc
```

The expected output is in `tools/oracle/tkmath_coherent_triptr_alloc.expected`.

## Source hashes

| File | SHA-256 |
| --- | --- |
| OCCT `Poly_CoherentTriPtr.hxx` | `36c26272136fccf3f3e28a66536068d2b4859a855edf7cf01c1e5ffc91708387` |
| OCCT `Poly_CoherentTriPtr.cxx` | `94c8114cd73f61b4df39ab335ac530b0a4f1bd00390d553c0f772894bd4f7eba` |
| OCCT `Poly_CoherentNode.hxx` | `43fb34c629691f23c8345f450a741238aa0a4e0947719e5406b789baf39ce254` |
| OCCT `Poly_CoherentNode.cxx` | `03f46cb19ea098227ed6b4bbc1e0b396004774d1486c689dcf744fa73615fbb1` |
| Rust `tri_ptr.rs` | `bae641cb69a9957b7fe429bca4222641eb286a681a649dd9bd996ac00eaa4a6b` |
| Rust `node.rs` | `4c3f41755aa0502775cbbdca2c69144ddae8b494d623c178644ce222dcec061d` |
| Rust `allocator.rs` | `42dc18b0a6ada3338b24bcaeac11008cb0cf326fc7fce9e340661fcf26e7b7c2` |
| Native probe | `18eca46be5cf9b1c0c9dc61846eb407002339d2f272a17c5a1252cd9e31fc5cb` |
