# PLib protected D0123 oracle

The probe derives from both PLib basis classes and makes the protected
`D0123` method callable. It initializes each output row to `777`, uses distinct
nonzero lower bounds, and records all four rows as IEEE-754 bit patterns.
This checks derivative-order dispatch, lower-bound offsets, and untouched rows.

The pinned OCCT revision is `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The headers and implementations are under
`src/FoundationClasses/TKMath/PLib/PLib_{JacobiPolynomial,HermitJacobi}.{hxx,cxx}`.
The Jacobi probe includes `NDeriv=-1` for both a degree-zero basis and a basis
with degree four, and `NDeriv=4`; the latter source path computes rows 0–2 and
leaves row 3 unchanged. The Hermit-Jacobi probe covers the supported public
range 0–3. Negative Hermit-Jacobi order is excluded because native constructs
derivative-sized local arrays from that value.

Reproduce with the repository's configured OCCT build:

```sh
c++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-build/include/opencascade \
  tools/oracle/tkmath_plib_d0123.cxx \
  /home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib/libTKMath.so \
  /home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib/libTKernel.so \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib \
  -o /tmp/tkmath-plib-d0123-native
/tmp/tkmath-plib-d0123-native > tools/oracle/tkmath_plib_d0123.expected
```

The expected file stores 12 deterministic output rows. SHA-256 values at
capture time:

- `PLib_JacobiPolynomial.hxx`: `bafa7209bf5ae06dfcc6dc0d9f6939803cbe570daa2498939f1c069a51d95bb7`
- `PLib_HermitJacobi.hxx`: `1c2bf48ff8950940fba027679ec01fe98d139e7e1da4f3333363d08ce28b6a16`
- `PLib_JacobiPolynomial.cxx`: `b71c7a244a2ecb1c4c5436db8c35c49a67ec1823fde3585d05b8d95e0388a4e8`
- `PLib_HermitJacobi.cxx`: `935cf733c4b2bdecdf722d4c9bf1ee58491f08279e494e075ab4a09d94aca93a`
- Probe source: `3c169d1316faaef89d7b39fdf80dd6f0e08d8283243f9d39af98d1a062fb2673`
- Expected output: `38f3f7d28a1fe5abf6d50d4813a0166f8ca772c24c50e412519a468986e0caeb`

The Rust wrappers expose borrowed slices instead of four `NCollection_Array1`
objects; callers provide empty slices for unrequested rows. This cannot alias
the same array into unused native parameters, but it preserves all defined
output rows without unsafe aliasing. Short buffers return `Standard_OutOfRange`.
