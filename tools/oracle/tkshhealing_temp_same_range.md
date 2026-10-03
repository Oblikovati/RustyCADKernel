# ShapeFix_Edge::TempSameRange oracle

The pinned `TempSameRange` helper is file-local in OCCT, so this probe invokes it through
`ShapeFix_Edge::FixSameParameter` on non-degenerated edges. That public path performs the
same temporary range repair before the remaining same-parameter check. Inputs cover a
sub-`PConfusion` delta, an above-threshold line delta, a periodic p-curve outside the
principal period, a bounded Bezier p-curve, and a closed-surface pair whose first periodic
curve shifts the source range used to reparameterize the second branch.

The fixture records the 3D edge range, p-curve range, dynamic curve types, endpoint values,
seam branch endpoint values, and final SameRange/SameParameter flags.

Build from the repository root against the pinned OCCT modeling build:

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_temp_same_range_probe.cxx \
  -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKOffset -lTKPrim -lTKBO -lTKBRep -lTKTopAlgo \
  -lTKGeomAlgo -lTKGeomBase -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_temp_same_range_probe
/tmp/tkshhealing_temp_same_range_probe
```
