# Generic NCollection_Vec2 scalar division dispatch

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.

`NCollection_Vec2<T>::operator/(T)` computes both components with `T::operator/`;
`operator/=(T)` updates each component with `T::operator/=`. The custom element
in the probe gives those operations distinct results. Its two rows are retained
in `tkernel_vec2_scalar_division_paths.expected`.

Build and replay against pinned generated headers:

```sh
c++ -std=c++17 -I target/occt-modeling-full/include/opencascade \
  tools/oracle/tkernel_vec2_scalar_division_paths.cxx \
  -o /tmp/vector-vec2-scalar-division
/tmp/vector-vec2-scalar-division > /tmp/vector-vec2-scalar-division.actual
diff -u tools/oracle/tkernel_vec2_scalar_division_paths.expected \
  /tmp/vector-vec2-scalar-division.actual
PATH=/home/vmiguel/.cargo/bin:$PATH cargo test -p rustycad-tkernel \
  --test ncollection_vec2_generic_scalar_division
```

Rust now accepts any component type that implements the corresponding `Div` or
`DivAssign` trait. Value `/` uses `Div`; `/=` uses `DivAssign`. Existing Vec2
float/double/integer behavior and Vec3/Vec4 scalar reciprocal implementations
remain separately covered. This adds supplemental template evidence and does not
add legacy test declarations.
