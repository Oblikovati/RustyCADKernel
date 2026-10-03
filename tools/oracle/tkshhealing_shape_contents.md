# Shape contents oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
build=/path/to/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_shape_contents.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKPrim -lTKTopAlgo -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/rustycad-shape-contents-oracle
/tmp/rustycad-shape-contents-oracle
cargo test -p rustycad-tkshhealing --test shape_analysis_shape_contents
```

Sixteen rows contain all 33 native counters in header order, then lengths of the
six retained shape sequences (big spline, indirect, offset surface, trimmed 3D,
offset curve, trimmed 2D). The Rust tests observe all counters and sequences.

The fixture includes repeated placements of one box: occurrence counts double,
while native shared counts strip location. It covers seam occurrences, indirect
surfaces, offset/trimmed/B-spline/Bezier surface types, the strict 8192-pole boundary,
3D and 2D curve wrappers, missing p-curves, and free topology. ClearFlags preserves
statistics and recordings; Clear preserves recording flags and empties the existing
sequence handles. The default copy constructor shares those sequence handles while
copying counters and flags; clearing a copy therefore clears the original's saved
shapes without changing its counters.

The native constructor allocates sequences and resets flags, but **does not initialize
its counters**. Reading counters before Clear/Perform is outside defined native
behavior. Rust initializes them to zero; the fixtures deliberately compare counters
only after native Clear or Perform. This documented safety difference is not counted
as behavioral parity. Generated tests supplement legacy obligations and do not prove
all malformed topology, curve, or surface behavior.
