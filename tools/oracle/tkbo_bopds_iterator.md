# `BOPDS_Iterator::Prepare` pair-list oracle

This probe compares every supported typed pair list for two overlapping unit boxes. It runs the pinned OCCT `BOPDS_DS::Init` and `BOPDS_Iterator::Prepare` path and records `Value()` order, including the native vertex/edge/face pair orientation.

```sh
g++ -std=c++17 -O0 -g0 \
  -I"$OCCT_BUILD/inc" -I"$OCCT_BUILD/lin64/gcc/inc" \
  -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkbo_bopds_iterator.cxx \
  -L"$OCCT_BUILD/lin64/gcc/lib" -Wl,--no-as-needed \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo \
  -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_bopds_iterator
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib" \
  /tmp/tkbo_bopds_iterator > /tmp/tkbo_bopds_iterator.expected
diff -u tools/oracle/tkbo_bopds_iterator.expected /tmp/tkbo_bopds_iterator.expected
```

The Rust slice now uses OCCT's linear-BVH BoxSet construction and PairSelector traversal, including mapped-ID sorting, the retained source-range cursor, and pair normalization at the native append point. OBB rejection is covered through the PaveFiller oracle and the reordered-input fixture below. `IntersectExt`, extended lists, and complete API/legacy parity remain outside this slice.

The three-argument range cursor has its own overlapping-box fixture. It catches mistakes where the candidate pair traversal restarts instead of retaining the index between source ranges:

```sh
g++ -std=c++17 -O0 -g0 \
  -I"$OCCT_BUILD/inc" -I"$OCCT_BUILD/lin64/gcc/inc" \
  -I"$OCCT_INSTALL/include/opencascade" \
  tools/oracle/tkbo_bopds_iterator_three_args.cxx \
  -L"$OCCT_BUILD/lin64/gcc/lib" -Wl,--no-as-needed \
  -lTKBO -lTKPrim -lTKTopAlgo -lTKShHealing -lTKGeomAlgo \
  -lTKBRep -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkbo_bopds_iterator_three_args
LD_LIBRARY_PATH="$OCCT_BUILD/lin64/gcc/lib" \
  /tmp/tkbo_bopds_iterator_three_args > /tmp/tkbo_bopds_iterator_three_args.expected
diff -u tools/oracle/tkbo_bopds_iterator_three_args.expected \
  /tmp/tkbo_bopds_iterator_three_args.expected
```

## Linear BVH integration

`tkbo_bopds_iterator_bvh.cxx` records all typed pair lists for two three-box
layouts with reordered spatial inputs and both OBB modes. All 204 output lines
match Rust. `tkbo_linear_box_selector.cxx` independently records 30 ordered
2D/3D selections through the native linear-builder default. The supplied-builder
Rust regression also matches the original BVH binned-builder fixture.

Replay: `cargo test -p rustycad-tkbo --test bop_ds_iterator --test box_selector`.
Parallel scheduling, arbitrary topology and complete legacy/API parity remain
unproven; no whole-module completion claim is made.
