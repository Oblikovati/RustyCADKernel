# PLib protected D0123 oracle

The probe derives from both PLib basis classes and makes the protected
`D0123` method callable. It initializes each output row to `777`, uses distinct
nonzero lower bounds, and records all four rows as IEEE-754 bit patterns.
This checks derivative-order dispatch, lower-bound offsets, and output mutation.

The pinned OCCT revision is `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The headers and implementations are under
`src/FoundationClasses/TKMath/PLib/PLib_{JacobiPolynomial,HermitJacobi}.{hxx,cxx}`.
The Jacobi probe includes `NDeriv=-1` for both a degree-zero basis and a basis
with degree four, and `NDeriv=4`; the latter source path computes rows 0–2 and
does not compute row 3 by recurrence, but still scales its initialized values
during normalization. For Jacobi `NDeriv=-1`, row 1 follows the native path
from its initialized contents. The Hermit-Jacobi probe covers the supported
range 0–3. Negative or greater-than-three Hermit-Jacobi orders are excluded
because native constructs derivative-sized local arrays from that value.

Reproduce with the repository's configured OCCT build:

```sh
c++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/occt-build/include/opencascade \
  tools/oracle/tkmath_plib_d0123.cxx \
  /home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib/libTKMath.so \
  /home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib/libTKernel.so \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/occt-build/lin64/gcc/lib \
  -o /tmp/tkmath-plib-d0123-native
/tmp/tkmath-plib-d0123-native > /tmp/tkmath-plib-d0123.expected
cmp tools/oracle/tkmath_plib_d0123.expected /tmp/tkmath-plib-d0123.expected
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
objects; callers provide empty slices for unrequested rows. The `BasisRows`
entry point uses disjoint slices, while Jacobi and HermitJacobi also expose
`d0123_overlapping` with one mutable buffer and four offsets to preserve defined
aliasing without unsafe references. Short buffers return `Standard_OutOfRange`.

## Modeling-full replay

The fixture was replayed against the current `target/occt-modeling-full`
libraries without replacing the checked-in expected output. `cmp` succeeded
exactly before fixture bytes were considered for update. Command, run from the
RustyCADKernel repository root:

```sh
c++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade \
  tools/oracle/tkmath_plib_d0123.cxx \
  /home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi/libTKMath.so \
  /home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi/libTKernel.so \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -o target/oracle/tkmath-plib-d0123/native-probe-modeling-full
target/oracle/tkmath-plib-d0123/native-probe-modeling-full \
  > target/oracle/tkmath-plib-d0123/native-probe-modeling-full.expected
cmp tools/oracle/tkmath_plib_d0123.expected \
  target/oracle/tkmath-plib-d0123/native-probe-modeling-full.expected
```

Compiler: `c++ (Ubuntu 13.3.0-6ubuntu2~24.04) 13.3.0`.

- `libTKMath.so`: `7afac5368e0ea361b8a60d2227bc3a1bbe34f9b71ec6c97447bc6957d9f007f8`
- `libTKernel.so`: `6dc26dc2e8ae4fbd4beddd4f1205947a6f1860f8ff202b65b2658970c08e0aee`
- Probe executable: `72743a38b46946840baba624564e1a31ad9a800766136ba82ca7f8de2022961d`
- Replay output: `38f3f7d28a1fe5abf6d50d4813a0166f8ca772c24c50e412519a468986e0caeb`
