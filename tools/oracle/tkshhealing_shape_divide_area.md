# ShapeUpgrade_ShapeDivideArea oracle

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
build=target/occt-modeling-full
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_shape_divide_area.cxx \
  -L"$lib" -Wl,-rpath,"$lib" -lTKShHealing -lTKTopAlgo -lTKBRep \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/rustycad-shape-divide-area-oracle
/tmp/rustycad-shape-divide-area-oracle
cargo test -p rustycad-tkshhealing --test shape_upgrade_shape_divide_area
```

Each row records wrapper MaxArea/NbParts, the fresh child face tool's values,
and whether two calls returned the same object. The positive-count overflow
case is intentionally excluded: signed integer overflow in native C++ has no
defined parity behavior; Rust rejects it before mutating the valid settings.
