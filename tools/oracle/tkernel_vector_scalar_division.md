# Vec3/Vec4 generic scalar division

Pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871` uses element `/=`
for generic Vec3/Vec4 scalar division. Value division copies and calls the same
assignment path. Its float/double specializations compute one reciprocal and
multiply every component by it. The fixture distinguishes these rounding paths
bit-for-bit and uses a custom element with no binary division operator.

`VectorScalarDivision` is the Rust specialization seam. Its default method uses
`DivAssign`; each standard integer type implements that default. The f32/f64
implementations use one reciprocal. A custom `Copy + DivAssign` type opts in with
`impl VectorScalarDivision for MyElement {}`. This explicit trait avoids an
overlapping blanket implementation while retaining the floating specializations.
Vec2 remains on its separate native direct-division path.

```sh
g++ -std=c++17 -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade tools/oracle/tkernel_vector_scalar_division.cxx -o /tmp/tkernel-vector-scalar-division
/tmp/tkernel-vector-scalar-division > /tmp/tkernel-vector-scalar-division.actual
diff -u tools/oracle/tkernel_vector_scalar_division.expected /tmp/tkernel-vector-scalar-division.actual
cargo test -p rustycad-tkernel --test vector_generic_scalar_division
```

The fixtures are supplemental and do not add original OCCT test declarations.
They cover defined arithmetic with Copy elements. C++ integer overflow, integer
division by zero and arbitrary element lifecycle behavior are outside this proof.
