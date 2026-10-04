# UnifySameDomain circular p-curve parameter probe

This probe preserves the circular branch at `ShapeUpgrade_UnifySameDomain.cxx`
around `UnionPCurves` lines 1940–1956. It verifies center/radius matching,
mapping the second arc endpoints through the first circle, sorting the mapped
parameters, and the reversed-curve parameter conversion around lines 2000–2010.

It is a formula-level oracle. The standard OCCT surface parameterizations used
by the current topology fixtures produce line p-curves for circular 3D edges,
so this probe does not claim end-to-end shape-unification coverage of a circular
p-curve pair.

Build it against the pinned OCCT build with:

```sh
c++ -std=c++17 -I"$NATIVE/include/opencascade" \
  tools/oracle/tkshhealing_unify_circle_pcurve.cxx \
  -L"$NATIVE/lin64/gcc/libi" -Wl,-rpath,"$NATIVE/lin64/gcc/libi" \
  -lTKShHealing -lTKPrim -lTKBO -lTKTopAlgo -lTKBRep -lTKGeomBase \
  -lTKG2d -lTKG3d -lTKMath -lTKernel -o /tmp/unify-circle-pcurve-probe
/tmp/unify-circle-pcurve-probe
```

The committed `.expected` file is the deterministic output from this command.
