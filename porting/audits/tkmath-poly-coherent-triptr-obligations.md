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
```

These rows mean three allocations and one free after the single-member removal;
the saved member address is still found in the ring after both insertion and
removal and still names triangle node `4`; and `RemoveList` brings the final
allocation/free totals to three each.

## Rust crosswalk and concrete gaps

| Native declaration | Current Rust entry | Difference |
| --- | --- | --- |
| `Poly_CoherentTriPtr(const Poly_CoherentTriangle&)` | `CoherentTriPtr::new(usize)` | Rust stores a triangle index, not a triangle pointer. |
| `GetTriangle() -> const Poly_CoherentTriangle&` | `CoherentTriPtr::get_triangle() -> usize` | No borrowed triangle identity is retained. |
| `SetTriangle(const Poly_CoherentTriangle*)` | `CoherentTriPtr::set_triangle(usize)` | Rust accepts an index and cannot represent the native null pointer. |
| `Next()`, `Previous()` return member references | `next(usize)`, `previous(usize)` return positions | Returned positions are not stable member identities. |
| `Append(const Poly_CoherentTriangle*, Handle(BaseAllocator))` | `append(member_position, triangle_index)` | No allocator argument or allocator-backed member allocation. |
| `Prepend(const Poly_CoherentTriangle*, Handle(BaseAllocator))` | `prepend(member_position, triangle_index)` | Same; insertion shifts later positional tokens. |
| `Remove(Poly_CoherentTriPtr*, Handle(BaseAllocator))` | `remove(member_position)` | No allocator argument; vector removal shifts positions. |
| `RemoveList(Poly_CoherentTriPtr*, Handle(BaseAllocator))` | `remove_list(self)` | No allocator argument; Rust drops the vector. |
| `Iterator::PtrValue() -> const Poly_CoherentTriPtr&` | `ptr_value() -> Option<usize>` | Rust exposes the current numeric position, not a stable node token. |

The production allocator wrappers in TKernel can allocate stable nodes (`BaseAllocator::allocate_value` and `AllocatedBox`), and `NCollection_List` already uses those wrappers. However, directly replacing this ring with a list would still require an explicit circular cursor/member API and allocator propagation from `CoherentNode::AddTriangle`, `RemoveTriangle`, and `Clear`. Adding allocator calls around the current `Vec<usize>` without making each ring member allocator-owned would only imitate counters and would not close pointer-identity semantics, so no such shim was added.

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
| Rust `tri_ptr.rs` | `94b94a26094f1090e91b0edc2585639ed6df84dac37b00233ac1ce0ca5db0e90` |
| Rust `node.rs` | `4c3f41755aa0502775cbbdca2c69144ddae8b494d623c178644ce222dcec061d` |
| Rust `allocator.rs` | `42dc18b0a6ada3338b24bcaeac11008cb0cf326fc7fce9e340661fcf26e7b7c2` |
| Native probe | `483f0ecfde319fbf0318640c5ffd79f5747382317da0b6ba419e6ebcf6547c2e` |
