# ShapeUpgrade_UnifySameDomain controls oracle

The fixture is from OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.
It checks `KeepShape` and `KeepShapes` on a chain junction, `KeepShape` on a
shared face edge, ignoring an unsupported kept face, output endpoint identity
with safe input mode both enabled and disabled, and `Initialize` clearing kept
shapes while preserving safe mode. Native behavior is implemented in
`src/ModelingAlgorithms/TKShHealing/ShapeUpgrade/ShapeUpgrade_UnifySameDomain.cxx`:
`KeepShape` filters to edges and vertices, `CheckSharedVertices` protects kept
vertices, and `UnifyEdges` copies endpoint vertices only in safe mode.

Build and run from the Rust kernel repository after configuring `target/occt-modeling-full`:

```sh
g++ -std=c++17 \
  -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkshhealing_unify_controls.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -lTKShHealing -lTKBO -lTKBool -lTKPrim -lTKTopAlgo -lTKBRep \
  -lTKGeomAlgo -lTKGeomBase -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkshhealing-unify-controls
/tmp/tkshhealing-unify-controls
```
