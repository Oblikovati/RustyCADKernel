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

The Rust `bind_ref` miss path clones the value before the key and then stores
the entry in the map's safe slot storage (2026-10-07). If a clone panics, any
completed local clone is dropped and nothing is published. Rust allocators are
a closed set (no injected subclass), and map entries take no allocator lease,
so the `A` event is not observable through the Rust API (plan D1 class B6);
the hash, copy and destructor order is. Existing-slot replacement also cannot
reproduce arbitrary C++ move-assignment and destructor hooks, so the API
remains partial.

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
