# ShapeAnalysis package oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
build=/path/to/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_shape_analysis_package.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKTopAlgo -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/rustycad-shape-analysis-package-oracle
/tmp/rustycad-shape-analysis-package-oracle
cargo test -p rustycad-tkshhealing --test shape_analysis_package_bounds
```

Eighteen rows cover four package operations: `GetFaceUVBounds`, `IsOuterBound`,
`ContourArea`, and `FindBounds`. Face rows contain success, U/V bounds and outer-bound
classification. A missing p-curve raises the native void-box failure. Edge-free
faces retain the surface's natural (possibly infinite) bounds. Single-wire outer
classification uses signed area; zero/multiple wires use the infinite-point classifier.

Area rows retain sampled contour closure and ignore the wire's overall orientation.
Endpoint rows contain presence flags then coordinates. Reversing a closed wire may
change its selected coincident endpoint to another vertex, as the native TopExp
algorithm does. Four Rust tests compare the frozen rows.

These generated fixtures are supplemental evidence. They do not replace legacy test
mappings or prove behavior for every curved, nonplanar or malformed topology domain.
