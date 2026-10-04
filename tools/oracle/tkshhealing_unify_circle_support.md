# Circle support selection in `ShapeUpgrade_UnifySameDomain`

The first case joins adjacent arcs with a shared center and radii `2.0` and
`2.0 + 5e-8`. The wire builder accepts the shared endpoint within confusion
tolerance. OCCT merges the chain into one arc using the first edge's radius,
with parameter range `[0, π]`. The beyond-tolerance radius case is disconnected
and remains two edges. Two more cases give the arcs a common endpoint but
nonparallel axes or nonconcentric centers; OCCT retains both edges.

This captures the source behavior in `ShapeUpgrade_UnifySameDomain.cxx`: circle
chain compatibility checks connected endpoints, coincident centers and join
tangent, while the union path constructs the result from the first circle. It
does not reject the radius difference within the endpoint tolerance. The Rust
regressions compare that native choice and its rejection boundaries.

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
