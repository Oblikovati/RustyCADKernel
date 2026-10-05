# DataMap Bind allocation ordering

This supplemental probe pins the allocator and subobject order of
`NCollection_DataMap::Bind(const Key&, const Item&)`. `NCollection_BaseMap`
allocates its bucket table with `Standard::Allocate`; the injected allocator's
`A` event therefore marks the actual node placement allocation. On a miss the
trace is `H A V K`: lookup hashes the borrowed key, node storage is allocated,
the `NCollection_TListNode` base copies the item, and the derived node copies
the key. A hit remains `H E V M D` with no allocator event. If key copy throws,
the already-constructed value is destroyed and placement delete frees the node
(`H A V K D F`); the map extent stays zero.

The Rust `bind_ref` miss path now calls `BaseAllocator::allocate_value_with`
before cloning. Its closure clones the value before the key and only then
builds `Entry`. If a clone panics, the uninitialized allocator-owned box is
released and any completed local clone is dropped. The Rust allocator still
owns an `Entry` rather than OCCT's combined base/list node, and bucket arrays
use Rust's global allocator. Existing-slot replacement also cannot reproduce
arbitrary C++ move-assignment and destructor hooks, so the API remains partial.

Reproduce against the pinned OCCT build:

```sh
OCCT_BUILD_ROOT=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
c++ -std=c++17 -O2 -I "$OCCT_BUILD_ROOT/include/opencascade" \
  tools/oracle/ncollection_data_map_bind_ref_allocation.cxx \
  -L "$OCCT_BUILD_ROOT/lin64/gcc/libi" \
  -Wl,-rpath,"$OCCT_BUILD_ROOT/lin64/gcc/libi" -lTKernel \
  -o /tmp/ncollection_data_map_bind_ref_allocation
/tmp/ncollection_data_map_bind_ref_allocation \
  | diff -u tools/oracle/ncollection_data_map_bind_ref_allocation.expected -
cargo test -p rustycad-tkernel --test ncollection_data_map_bind_ref_allocation
```
