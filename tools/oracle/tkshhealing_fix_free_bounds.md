# Free-boundary repair oracle

The fixture runs `ShapeFix_FreeBounds` from pinned OCCT
`3d097a0328e71b826377d4814ab05ec3c3d23871`. It records null output state,
open/closed wire counts, open edge counts, and distinct source vertices.
The finite open-face example throws `TopoDS_FrozenShape` during forecast
connection; Rust preserves that exception instead of thawing source edges.

```sh
build="$PWD/target/occt-modeling-full"
lib="$build/lin64/gcc/libi"
g++ -std=c++17 -I"$build/include/opencascade" \
  tools/oracle/tkshhealing_fix_free_bounds.cxx -L"$lib" -Wl,-rpath,"$lib" \
  -lTKShHealing -lTKTopAlgo -lTKBRep -lTKG3d -lTKG2d -lTKMath -lTKernel \
  -o /tmp/tkshhealing_fix_free_bounds
/tmp/tkshhealing_fix_free_bounds > /tmp/tkshhealing_fix_free_bounds.actual
diff -u tools/oracle/tkshhealing_fix_free_bounds.expected /tmp/tkshhealing_fix_free_bounds.actual
cargo test -p rustycad-tkshhealing --test shape_fix_free_bounds
```

These generated geometry cases establish the observed outputs and exception.
They are not exhaustive coverage of the underlying sewing or boundary algorithms.
