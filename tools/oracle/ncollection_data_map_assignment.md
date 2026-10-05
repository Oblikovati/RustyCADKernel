# DataMap assignment oracle

This supplemental probe targets OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It exercises `Assign`, copy
assignment and move assignment using a stateful hasher and tracked key/value
copies. `HVK` records one hash of the original source key, then value copy,
then key copy. The destination keeps its hash policy and allocator on copy.

Copy/move self-assignment returns the same object, keeps its node address and
runs no hash or copy, even when key copying would throw. A key-copy exception
leaves the cleared destination empty. Move assignment exchanges existing nodes,
hash policies and allocator identities. Empty-source assignment returns the
destination and clears bindings.

```sh
c++ -std=c++17 -O2 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/ncollection_data_map_assignment.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -lTKernel -o /tmp/ncollection_data_map_assignment
/tmp/ncollection_data_map_assignment > /tmp/datamap-assignment-native.actual
diff -u tools/oracle/ncollection_data_map_assignment.expected \
  /tmp/datamap-assignment-native.actual
```

This probe and the Rust regressions are supplemental. They do not add original
OCCT test declarations. The generic C++ subobject/address-sensitive copy,
custom destructor/swap hooks and allocator failure semantics remain partial.
