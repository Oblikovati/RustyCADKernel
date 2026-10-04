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
| protected `Poly_CoherentTriPtr(const Poly_CoherentTriangle*)` | `new_from_nullable_triangle(Option<usize>)`, `new_from_nullable_triangle_in(Option<usize>, BaseAllocator)` | Nullable lifecycle analogue is implemented as a partial mapping. The compiled declaration selector is `0b03943cf032772caf87cbcd6ec6be0322138699d082162b896da66ed68e38d0`; the Rust payload is still an index, not an OCCT pointer. |
| `GetTriangle() -> const Poly_CoherentTriangle&` | `triangle(CoherentTriMember) -> Result<usize, Failure>` | Stable member ownership is represented; a Rust borrow into the separate triangulation array is not. |
| `SetTriangle(const Poly_CoherentTriangle*)` | `set_triangle_nullable(Option<usize>)` | `None` stores the native null state; the triangle value remains an index, so pointer identity and ABI parity are partial. Compiled declaration selector: `8648f15839de4b481b68c5fdd217f00ca1531a1c5ec55628135e1fc123eec931`. |
| `Next()`, `Previous()` return member references | `next_member`, `previous_member` | Stable token links are implemented; positional `next`/`previous` remain compatibility conveniences. |
| `Append(const Poly_CoherentTriangle*, Handle(BaseAllocator))` | `append_member(member, triangle, allocator)` | Each member is a real `AllocatedBox` allocation from the supplied allocator. |
| `Prepend(const Poly_CoherentTriangle*, Handle(BaseAllocator))` | `prepend_member(member, triangle, allocator)` | Each member is a real `AllocatedBox` allocation from the supplied allocator. |
| `Remove(Poly_CoherentTriPtr*, Handle(BaseAllocator))` | `remove_member(member, &allocator)` | Unlinks stable token and frees its allocation; mismatched allocator is rejected safely. |
| `RemoveList(Poly_CoherentTriPtr*, Handle(BaseAllocator))` | `remove_list_in(&allocator)` | Clears all nodes if all allocations match; ordinary `Drop` also frees each node through its recorded allocator. |
| `Iterator::PtrValue() -> const Poly_CoherentTriPtr&` | `CoherentTriPtrIterator::member_token()` | Stable node identity is returned as an opaque token backed by the allocated node's stable address. |

`CoherentTriPtr` now owns actual allocator-backed nodes using `BaseAllocator::allocate_value` and `AllocatedBox`; each node contains stable next/previous member tokens. No unsafe pointers are dereferenced. Legacy positional methods still route through this circular node storage. The Rust poly test target covers stable tokens, previous/set/iterator operations, dead-token errors, mismatched-allocator non-mutation, and independent empty/populated clones. A focused unit test verifies bulk removal drains head-then-next like native. The native allocator probe records frees as `7 1 4`: explicit removal of triangle 7, then `RemoveList` traversing the remaining head and its next member.

Out-of-range positional lookups reject the index before walking the ring, including `usize::MAX`; a supplemental regression covers both `triangle_at` and `iter_from`.

The explicit removal methods currently validate `BaseAllocator` handle identity. That is a conservative ownership rule, not an exact model of every native `Free` combination: different native allocator objects can share a compatible heap backend. The node owner always frees through its creating Rust allocator, so this check prevents releasing memory through an incompatible arena. Exact native backend-compatibility semantics remain open.

The [nullable lifecycle probe](../../tools/oracle/tkmath_coherent_nullable_triptr.md) covers the protected null constructor, linking valid neighbors, resetting the head with `SetTriangle`, and removing a null-valued node without dereferencing it. The Rust nullable inspection, clone, equality, debug, and removal cases are supplemental and do not change the legacy declaration count. Native `Append` and `Prepend` dereference their input pointer while constructing a node, so passing null is undefined and is deliberately excluded from the Rust parity claim. Rust's ordinary value iterator explicitly panics on a null node; native iterator dereference there is undefined.

The subsequent [coherent allocator propagation slice](tkmath-coherent-allocator-propagation.md) adds explicit node allocator operations and retains/forwards the mesh allocator through construction, edits, cloning and destruction. Nullable state can now be represented and manipulated, but triangle pointers are still indices, and backend compatibility across distinct allocator identities remains partial. This is not a parity claim for the complete node or triangulation classes.

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
| Rust `tri_ptr.rs` | `161b506b541b71083e268bf4eba41385f5f8d29884bb08bdce67c592cdda1d8b` |
| Rust `tri_ptr/nullable.rs` | `f0e72dcc87a7b05a51d51a94bb84ce1382a8fccc958eb2107a5bc8513ac55a17` |
| Rust `node.rs` | `d677820448dd1cd95f8da4b6e547de7ba37286fe1db5c64075f1043829429ab2` |
| Rust `allocator.rs` | `80060dbc92dc681ed324edea5739c5067cb18a6a31c3359715fbef8aec7d00c0` |
| Native probe | `18eca46be5cf9b1c0c9dc61846eb407002339d2f272a17c5a1252cd9e31fc5cb` |
| Nullable lifecycle probe | `65f50a3df56a3703a57b5eec341489b4a9102221f9402413fb282df03462c5d9` |
