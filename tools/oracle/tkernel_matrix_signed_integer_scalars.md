# Signed-integer Mat3/Mat4 adjoint, inversion, and translation

The pinned native classes are unconstrained templates; Rust previously gated
these methods behind `MatrixScalar`, implemented only for `f32`/`f64`. The
trait also requires the arithmetic operations used by the formulas, all of
which `i32`/`i64` supply. This slice adds those two signed implementations.

The probe compares defined inputs only: all intermediate signed operations
remain in range, and no division by zero occurs. C++ signed overflow is
undefined; this evidence does not claim parity there. It covers determinant
`+1`, `-1`, `2`, and `0` inversion paths. Mat3 divides each adjugate component
by its determinant, while Mat4 computes an integer reciprocal first. For a
determinant of `2`, that produces Mat3's component-wise truncation and Mat4's
all-zero result. On a zero determinant, Mat3 preserves the output matrix;
Mat4 publishes its unscaled cofactors before returning `false`.

The pinned revision is `3d097a0328e71b826377d4814ab05ec3c3d23871`. Relevant
header implementation locations are Mat3 `Adjoint` line 453 and `Inverted`
lines 466–495; Mat4 `Translate` line 498, `Inverted` lines 524–609, and
`Adjoint` line 626. Header SHA-256 values:

| Header | SHA-256 |
| --- | --- |
| `NCollection_Mat3.hxx` | `a6e8f11773efab41789dedd45e55d05f556f420c7073f3fd01c96d0e079afa51` |
| `NCollection_Mat4.hxx` | `1b2e16e62d4c520c3939ee6916a9607bd8d128d83850fbca2b52f0b8498e2df0` |

The oracle command uses GCC 13.3 and the pinned OCCT install:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-install/include/opencascade \
  tools/oracle/tkernel_matrix_signed_integer_scalars.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-install/lib \
  -lTKernel -o /tmp/tkernel-matrix-signed-integer-scalars-native
/tmp/tkernel-matrix-signed-integer-scalars-native
```

Its complete output is preserved in the adjacent `.expected` file. Rust tests
exercise the same cases for both `i32` and `i64`, including output mutation,
boolean and throwing wrappers, adjugates, and translation of a scaled matrix.
