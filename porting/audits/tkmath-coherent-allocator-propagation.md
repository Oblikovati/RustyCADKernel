# Coherent triangulation allocator propagation

This slice follows the native `Poly_CoherentTriangulation` allocator from constructors and `Clone(theAlloc)` into every `Poly_CoherentNode::AddTriangle`, `RemoveTriangle`, and destructor `Clear` call. Native `Poly_CoherentNode` itself does not retain the allocator: each of those operations receives the allocator explicitly. Rust exposes `CoherentNode::{add_triangle_in,remove_triangle_in,clear_in}` for that contract, keeps the existing convenience methods, and stores the allocator on `CoherentTriangulation` so its constructor copy, edits, clone, and `Drop` pass the same allocator.

`CoherentTriangulation::new_in` and `from_triangulation_in` are allocator-taking constructor forms. `allocator()` corresponds to native `Allocator()`. Existing constructors continue to use `BaseAllocator::common()`. New node-ring members therefore receive the mesh's allocator, and mesh removal/destruction supplies that same allocator for release. Tests also verify a different allocator is rejected before a node ring is changed.

The focused Rust tests use `NCollection_IncAllocator` ownership accounting: `reset(false)` fails while mesh or clone ring nodes are live and succeeds after removal or destruction. This checks allocator ownership rather than only checking that the constructor preserved a handle.

The native probe uses four nodes and one source triangle. It records allocator counts after source copy construction, cloning into a second allocator, clone destruction, adding a second triangle, removing it, clearing a standalone node ring, and destroying the source mesh. The expected observations are:

```text
3 0
3 0
3 3
6 0
6 3
7 3
7 4
7 7
```

Rows one, two, and three report source allocator construction, clone allocator construction, and clone destruction respectively. The remaining rows report the source allocator through mutation, standalone-node clear, and mesh destruction.

The standalone node is cleared exactly once because native `Poly_CoherentNode::Clear` calls `RemoveList` but does not null its raw ring pointer; invoking it on a mesh-owned node and later destroying the mesh would repeat that removal. The probe avoids that invalid lifecycle. Rust safely clears the `Option` after a successful allocator-matched removal. A supplied allocator that does not match the Rust node owner returns a `Standard_ProgramError` rather than freeing through an incompatible arena.

This remains partial. `BaseAllocator` currently enforces allocator-handle identity, while native allocators with compatible allocation/free backends may be interchangeable. Null native triangle pointers are also outside the safe Rust API. No ledger row is promoted by this audit.

## Reproduction

With the pinned OCCT build at `target/occt-tkernel-full`:

```sh
occt=target/occt-tkernel-full
c++ -std=c++17 -O2 -I"$occt/include/opencascade" \
  tools/oracle/tkmath_coherent_allocator_propagation.cxx \
  -L"$occt/lin64/gcc/libi" -Wl,-rpath,"$occt/lin64/gcc/libi" \
  -lTKMath -lTKernel -o /tmp/tkmath_coherent_allocator_propagation
/tmp/tkmath_coherent_allocator_propagation
```

The checked output is `tools/oracle/tkmath_coherent_allocator_propagation.expected`.

## Pinned source hashes

| File | SHA-256 |
| --- | --- |
| OCCT `Poly_CoherentNode.hxx` | `43fb34c629691f23c8345f450a741238aa0a4e0947719e5406b789baf39ce254` |
| OCCT `Poly_CoherentNode.cxx` | `03f46cb19ea098227ed6b4bbc1e0b396004774d1486c689dcf744fa73615fbb1` |
| OCCT `Poly_CoherentTriangulation.hxx` | `6c2a5d9d94542542372ddc9346544f15cebb743df0194f4877e4040e82ebf1a7` |
| OCCT `Poly_CoherentTriangulation.cxx` | `de8bddeed00a2572d13818116e58ab97b633fe633f2c4c2a4d7d2ec21f34b70e` |
| Native probe | `96f5ba6ea7a9707b72b702cfe53e85a5397bd46aba3c75ea48cd475c834f6ee8` |
