# BSplSLib cache transient identity

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`BSplSLib_Cache.hxx` SHA-256:
`d6e54cfaa813f8d8c9e6fccd0ae5747e6b65f100c2b1a4cd4d41436c6e28a23e`.
`BSplSLib_Cache.cxx` SHA-256:
`00643cffe1811a98c2da02eb6eed6a3242eae7e5ca9f62e91c37acea60b913a3`.

The probe constructs a degree-1 cache with flat knots `[0, 0, 1, 1]`, stores it
in `occ::handle<BSplSLib_Cache>`, then projects that handle to
`occ::handle<Standard_Transient>`. It prints the dynamic name, exact-instance
and ancestor checks, both shared reference counts, and the dynamic name through
the base handle. OCCT prints `BSplSLib_Cache 1 1 2 2 BSplSLib_Cache`.

Reproduce with the pinned local OCCT headers and libraries:

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade \
  tools/oracle/tkmath_bsplslib_cache_rtti.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/tkmath-bsplslib-cache-rtti
/tmp/tkmath-bsplslib-cache-rtti > /tmp/tkmath-bsplslib-cache-rtti.out
cmp /tmp/tkmath-bsplslib-cache-rtti.out tools/oracle/tkmath_bsplslib_cache_rtti.expected
```

The Rust regression checks the matching concrete descriptor, its
`Standard_Transient` parent, dynamic identity after upcast, object identity,
and reference-count increments. The existing `SharedBSplineSurfaceCache`
remains the separate mutable shared-cache wrapper used by adaptors; this change
adds native runtime identity to the cache value without replacing that
mutation path.
