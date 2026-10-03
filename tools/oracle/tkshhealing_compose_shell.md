# ShapeFix_ComposeShell oracle

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`. The first fixture
records constructor status/result/mode, the default projection transfer strategy, and
replacement of that strategy through the native setter/getter.
The one-patch planar Init fixture also checks that initialization preserves the fresh result
and status while leaving closed mode disabled.
The protected `LoadWires` fixture uses a derived native probe to verify edge count and
traversal orientation for a manifold boundary. It also verifies that an empty
nonmanifold partition does not produce a spurious wire segment.
The `ComputeCode` fixture checks the protected edge-side classifier on a p-curve
subrange against a vertical cutting line.
The patch-limit fixture checks the native undefined-index sentinels and side-bit updates.
The two-patch planar `SplitEdges` row records the replacement type/count for each
boundary edge after a vertical joint splits the top and bottom edges.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_compose_shell.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKGeomBase -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_compose_shell
/tmp/tkshhealing_compose_shell
cargo test -p rustycad-tkshhealing --test shape_fix_compose_shell_behavior
cargo test -p rustycad-tkshhealing --lib shape_fix::compose_shell::load_wires::tests::manifold_boundary_load_matches_native_protected_helper
```
