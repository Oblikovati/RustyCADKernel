# Jacobi `D0123` overlapping-output probe

This supplemental probe exercises `PLib_JacobiPolynomial::D0123` when its four
mutable `NCollection_Array1` arguments overlap a shared caller buffer. The
native implementation writes seed rows first, then each recurrence index in
ascending derivative order, then normalizes by index and row. Same-array aliasing
for orders -1, 0, 1, 2, 3, and 4, selected-row aliasing, offset overlap, and
disjoint-buffer cases preserve those write effects as IEEE-754 bit patterns.

The native source is pinned OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`;
the relevant implementation is `PLib_JacobiPolynomial.cxx` lines 379–519.
The Rust API `JacobiPolynomial::d0123_overlapping` takes one mutable slice and
four element offsets, so safe Rust never creates overlapping `&mut` references.
The existing `BasisRows` entry point uses the same indexed recurrence against
four disjoint slices. Short spans return `Standard_OutOfRange` before mutating
the buffer; that is the defined Rust boundary for inputs that would otherwise
index outside caller storage.

Replay against the configured OCCT build:

```sh
c++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-build/include/opencascade \
  tools/oracle/tkmath_plib_jacobi_overlap.cxx \
  /home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib/libTKMath.so \
  /home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib/libTKernel.so \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib \
  -o /tmp/tkmath-plib-jacobi-overlap
/tmp/tkmath-plib-jacobi-overlap > /tmp/tkmath-plib-jacobi-overlap.expected
cmp tools/oracle/tkmath_plib_jacobi_overlap.expected \
  /tmp/tkmath-plib-jacobi-overlap.expected
```

Evidence hashes:

- Native implementation: `b71c7a244a2ecb1c4c5436db8c35c49a67ec1823fde3585d05b8d95e0388a4e8`
- Native probe: `3755fedac6b73c16c5cca1322f211dd18ba5e0f4332a1310807bb566ee9d7e0b`
- Native output fixture: `ca0421e011ff45988353d1cc27b3cae7ed084525e4e44fbc580a970d1469780e`
- Native executable: `715e6c1151db6698d668fafd157cdc159211b9c8f2d92fb02091b788ef4798c2`
- `libTKMath.so`: `42cf96a613fc5d7ff0c840206a4d06e9777e63aa22117dd4c55701ae7b3d192f`
- `libTKernel.so`: `78221dc495f8316c2a28e4fd9202920efdd17e056549006646ffd81bd0ddcc87`
