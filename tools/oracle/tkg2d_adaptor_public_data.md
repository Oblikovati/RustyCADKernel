# TKG2d adaptor public-data probe

OCCT source is pinned to `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The probe exercises default construction of `OffsetData`, `BezierData`, and
`BSplineData`, then copies `BezierData`, mutates its shared cache through the copy,
and reads the result through the original handle. This checks the native facts that
the records default to null handles/zero offset and that copying a cache handle
preserves identity and shared mutation.

The pinned native build is
`target/occt-modeling-full/lin64/gcc/libi`; `libTKG2d.so` has SHA256
`994da30dc1348ee9427b8bb453417ea4a3c4a6c6856fb2c0ab5c36bf816046ce` and
`libTKMath.so` has SHA256
`7afac5368e0ea361b8a60d2227bc3a1bbe34f9b71ec6c97447bc6957d9f007f8`.
The public adaptor header SHA256 is
`b78eb0d87fadfa6b64e3902e952e6ecdbafb87654c69be1443ee66c10f6925ad`; the cache
header SHA256 is
`a2fa139ecc1871424bf722abf3f8beb041cf4969b2670825c56ab9fe38362ad9`.

Build and run the probe with:

```sh
c++ -std=c++17 -O2 \
  -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkg2d_adaptor_public_data.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -lTKG2d -lTKMath -lTKernel \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -o /tmp/tkg2d-adaptor-public-data
/tmp/tkg2d-adaptor-public-data
```

Native output:

```text
1 0 1 1 1 1 1 1 1
1 0.25 0.25
```

The equivalent Rust assertions are in `crates/tkg2d/tests/geom2d_adaptor_public_data.rs`.
The Rust `rustycad_tkmath::bsplclib::SharedBSplineCache` wrapper preserves nullability,
clone identity, and shared mutation, but it does not claim OCCT `Standard_Transient`
RTTI parity for `BSplCLib_Cache`. It lives in TKMath so both TKG2d and TKG3d can
reuse the cache-handle semantics without depending on adaptor internals.
