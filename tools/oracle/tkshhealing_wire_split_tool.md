# ShapeFix split-tool range-cut branches

Pinned OCCT source: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

Two standalone native probes exercise `ShapeFix_SplitTool::CutEdge`:

- The tiny trimmed-line cut reaches the native early return after it updates the
  face range. It leaves `cutLine` false and preserves the edge's resulting
  stored range.
- The bounded same-parameter curve cut asks `ShapeAnalysis_Curve::ValidateRange`
  to clamp the cut to the trimmed curve domain. Native range becomes `[0, 1]`
  and SameRange/SameParameter remain true.

The Rust regressions live with the adjacent wire-intersection tests because the
helpers are private implementation dependencies of `ShapeFix_Wire`. Each native
probe can be built with the OCCT install using the common link set:

```sh
for fixture in tkshhealing_wire_split_tool_early_range tkshhealing_wire_split_tool_validated_range; do
  g++ -std=c++17 -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
    "tools/oracle/${fixture}.cxx" \
    -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
    -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
    -lTKShHealing -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKBRep -lTKMath -lTKernel \
    -o "/tmp/${fixture}"
  LD_LIBRARY_PATH=/home/vmiguel/git/oblikovati-workspace/occt-install/lib "/tmp/${fixture}"
done
```

These probes cover two `CutEdge` branches and do not establish full
`ShapeFix_Wire` or toolkit parity.
