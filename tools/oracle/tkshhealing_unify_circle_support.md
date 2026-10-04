# Circle support selection in `ShapeUpgrade_UnifySameDomain`

The probe joins two adjacent arcs with a shared center and radii `2.0` and
`2.0 + 5e-8`. The wire builder accepts the shared endpoint within confusion
tolerance. OCCT merges the chain into one arc using the first edge's radius,
with parameter range `[0, π]`.

This captures the source behavior in `ShapeUpgrade_UnifySameDomain.cxx`: circle
chain compatibility checks the centers and join tangent, while the union path
constructs the result from the first circle. It does not reject this radius
difference. The Rust regression compares that native choice and result.

Rebuild the pinned native probe with:

```sh
g++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkshhealing_unify_circle_support.cxx \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,target/occt-modeling-full/lin64/gcc/libi \
  -lTKShHealing -lTKBO -lTKBool -lTKPrim -lTKTopAlgo -lTKBRep \
  -lTKGeomAlgo -lTKGeomBase -lTKG2d -lTKG3d -lTKMath -lTKernel \
  -o /tmp/tkshhealing-unify-circle-support
/tmp/tkshhealing-unify-circle-support
```
