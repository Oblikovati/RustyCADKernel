# ShapeUpgrade_FaceDivide oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Nine native cases compare return value, result nullness/identity/face count, context
creation, all DONE1–3/FAIL1–3 flags, and recorded original vertex occurrences.
Cases cover base plane behavior, disabled surface/wire/both tools, segmentation
mode, a cylinder split by the angular criterion, null input, a derived surface
policy reporting DONE3, and an input face removed by context.

The derived `MarkModifiedSurface` policy is implemented in both languages. It
exercises the native protected-state extension contract and forces copying original
vertices before composition. The cylinder fixture also verifies dynamic dispatch
through the base surface-tool handle. The existing two translated legacy safety
regressions (including concurrent calls) still pass. These nine oracle cases do
not add legacy declaration mappings.

Face subdivision now performs surface splitting before boundary splitting, with
native intermediate result/status updates and tool selection order. It inherits
remaining limitations of shell composition and configured splitting policies;
this fixture is not proof of complete toolkit or whole-kernel parity.

```sh
build=/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -O2 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_face_divide.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_face_divide
/tmp/tkshhealing_face_divide
cargo test -p rustycad-tkshhealing --test shape_upgrade_face_divide
cargo test -p rustycad-tkshhealing --test shape_upgrade_face_divide_behavior
```
