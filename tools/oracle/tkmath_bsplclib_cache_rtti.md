# BSplCLib cache transient identity

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
`BSplCLib_Cache.hxx` SHA-256:
`a2fa139ecc1871424bf722abf3f8beb041cf4969b2670825c56ab9fe38362ad9`.
`BSplCLib_Cache.cxx` SHA-256:
`ba74142b88030fc10b7e79a778c1303734e13a1991a9c800a893c377b4fabc59`.

The probe constructs a degree-1 3D curve cache with flat knots `[0, 0, 1, 1]`
and poles `(0,0,0)`, `(1,0,0)`. It stores the cache in
`occ::handle<BSplCLib_Cache>`, upcasts it to `occ::handle<Standard_Transient>`,
then prints the dynamic name, exact-instance and ancestor checks, both reference
counts, and the name through the base handle. OCCT prints
`BSplCLib_Cache 1 1 2 2 BSplCLib_Cache`.

Reproduce with the pinned OCCT headers and libraries:

```sh
g++ -std=c++17 \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/include/opencascade \
  tools/oracle/tkmath_bsplclib_cache_rtti.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-modeling-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/tkmath-bsplclib-cache-rtti
/tmp/tkmath-bsplclib-cache-rtti > /tmp/tkmath-bsplclib-cache-rtti.out
cmp /tmp/tkmath-bsplclib-cache-rtti.out tools/oracle/tkmath_bsplclib_cache_rtti.expected
```

The Rust regression checks the concrete descriptor, its direct
`Standard_Transient` parent, identity and dynamic type after handle projection,
and the incremented reference count. `SharedBSplineCache` remains the mutable
shared-cache wrapper used by adaptors; this adds transient identity to the cache
value without changing that sharing behavior.
