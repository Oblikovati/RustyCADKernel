# `NCollection_IndexedDataMap::Add(const Key&, const Item&)` probe

The probe uses a custom `Hasher` and copy-observing key/item types. It distinguishes
the original-key lookup from copy construction and checks the existing-key no-op.

Build against the pinned OCCT installation:

```sh
c++ -std=c++17 -O2 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/ncollection_indexed_data_map_borrowed_add.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKernel -o /tmp/ncollection_indexed_data_map_borrowed_add
/tmp/ncollection_indexed_data_map_borrowed_add
```

Pinned source: `NCollection_IndexedDataMap.hxx`, `addImpl` and its node constructor.
On insertion, OCCT hashes the borrowed key, copies the item before the key, and
returns the new one-based index. A hit hashes and compares the borrowed key, then
returns its index without copying either input or changing the stored item.
The output is checked in `ncollection_indexed_data_map_borrowed_add.expected`.

This probe does not model arbitrary C++ copy/destructor exceptions or allocator
failure side effects; it records value-copy order for the tested types.
