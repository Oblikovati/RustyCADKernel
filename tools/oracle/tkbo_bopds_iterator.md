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

The Rust slice reproduces the prepared candidate lists and iteration order, but uses pairwise AABB checks instead of OCCT's BoxTree. OBB rejection, `IntersectExt`, and all geometric intersection stages remain outside this slice.
