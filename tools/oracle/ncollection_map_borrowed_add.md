# Map borrowed Add and Added oracle

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The supplemental native probe records `H` hash, `E` equality, `A` allocation,
`K` key copy and `F` free. Insertion is `HAK`, while a duplicate is `HE`,
including when key copying would throw. `Added` returns a separately stored
key on miss and the same stored address on hit without an additional lookup.
An injected key-copy exception produces `HAKF` and publishes no key.
The owned overload hashes once and moves the key without a copy (`HA`).

```sh
c++ -std=c++17 -O2 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/ncollection_map_borrowed_add.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -lTKernel -o /tmp/ncollection_map_borrowed_add
/tmp/ncollection_map_borrowed_add > /tmp/map-borrowed-add-native.actual
diff -u tools/oracle/ncollection_map_borrowed_add.expected \
  /tmp/map-borrowed-add-native.actual
```

Rust regressions verify the single lookup, copy suppression on hits, live
node allocation during Clone, stable returned key and panic cleanup. Native
bucket allocation and C++ address-sensitive subobject construction, custom
move/destructor hooks and allocation failure categories remain partial.
These supplemental cases do not add original OCCT test declarations.
