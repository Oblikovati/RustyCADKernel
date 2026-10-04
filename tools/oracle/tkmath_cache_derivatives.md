# BSplCLib cache raw derivative buffer

Pinned source: OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871`;
`BSplCLib_Cache.cxx` SHA-256 `ba74142b88030fc10b7e79a778c1303734e13a1991a9c800a893c377b4fabc59`;
`BSplCLib_Cache.hxx` SHA-256 `a2fa139ecc1871424bf722abf3f8beb041cf4969b2670825c56ab9fe38362ad9`;
`PLib.cxx` SHA-256 `c408b3a963e5e2161934162ec47868779303f6f7033033beee929bc43acb9fe6`.

The protected native helper is called through a derived `ExposedCache` in
`tkmath_cache_derivatives.cxx`. It builds a degree-1 2D cache, evaluates local
parameter `0.25` through derivative order 1 into six caller values initialized
to `-77`, then requests order 3 from a degree-1 curve and prints all ten slots.
It also evaluates a rational 3D curve on a non-unit `[2, 6]` span through both
the global-parameter and local-parameter protected helpers. It also requests
order 5 from both protected helpers for a non-rational quartic Bezier on
`[2, 6]`; the local probe supplies normalized parameter `0.5` directly. A rational 2D quartic on `[2, 6]` additionally evaluates order four at global `3.0` and local `0.25`: five homogeneous rows require 15 temporary values, within the native 16-value array. Its two extra output slots remain `-77`.

Compiler used for the checked-in output: `g++ (Ubuntu 13.3.0-6ubuntu2~24.04.1)
13.3.0`. The exact command below produced `/tmp/tkmath-cache-derivatives`;
the checked-in expected file was copied from its stdout and verified with `cmp`.
Binary SHA-256: `25c372428c47b3715bd8e04223c81c8e1997983feacd5b9741f172169b176944`;
captured stdout SHA-256: `e843e0cf5da64b499f81b1f572e2d8c7767f47875344587e5f280f618583ce5f`.

Reproduce against the pinned OCCT headers and libraries:

```sh
g++ -std=c++17 -O2 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade \
  tools/oracle/tkmath_cache_derivatives.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/tkmath-cache-derivatives
/tmp/tkmath-cache-derivatives
```

To regenerate and check the expected output byte-for-byte, use:

```sh
/tmp/tkmath-cache-derivatives > /tmp/tkmath-cache-derivatives.out
cmp /tmp/tkmath-cache-derivatives.out tools/oracle/tkmath_cache_derivatives.expected
```

Observed OCCT output begins with `1 2 4 8 -77 -77`, then `1 2 4 8 0 0 0 0
-77 -77`. Both rational global and local helpers produce point `(1.6, 3.2, 4.8)`
and derivative `(1.28, 2.56, 3.84)`. Both helpers scale derivative rows by the
inverse span length. The local helper accepts a normalized span parameter but
returns derivatives in the original curve-parameter units.
Both leave spare capacity alone. Rust exposes a bounded slice operation for
this protected subclass contract; too-short slices return
`Standard_DimensionMismatch` before changing any element. Rational requests support 0..=4 in 2D and 0..=3 in 3D, matching the capacity of the native fixed 16-value temporary. Both reject higher orders before writing; native requests beyond this capacity would write out of bounds. For
non-rational global and local requests, the implementation uses caller storage
and `PLib::EvalPolynomial`'s general-order path. Although the header documents
local orders 1..=3, its implementation supports these higher orders: order-5
quartic rows confirm a nonzero fourth derivative, a cleared fifth row, and
untouched spare slots; the direct Local and global rows match. Rust's rational
result comparison uses absolute tolerance `1e-14`; it does not assert bit-for-bit
identity.

The fourth-order 2D rows are exactly representable in binary and compare exactly in Rust: fourth derivative `(-0.9375, 0.46875)`. The prior rational 3D comparison retains its existing absolute tolerance; no tolerance was widened. These additions are supplemental cases, not new legacy declarations.
