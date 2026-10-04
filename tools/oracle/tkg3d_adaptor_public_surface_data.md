# TKG3d surface-adaptor public-data probe

OCCT source is pinned to `3d097a0328e71b826377d4814ab05ec3c3d23871`.
The native probe default-constructs all five public `GeomAdaptor_Surface` data
records and the `SurfaceDataVariant`, then copies a `BezierData` record, builds
its shared `BSplSLib_Cache` through the copy, and evaluates through the original
record. This covers the public null/default fields and shared cache alias behavior.

The pinned native build is
`target/occt-modeling-full/lin64/gcc/libi`; `libTKG3d.so` has SHA256
`d2a97b6904ce5c80b63da669e68e99b3f240269dec51a262a4746b9b236584f4` and
`libTKMath.so` has SHA256
`7afac5368e0ea361b8a60d2227bc3a1bbe34f9b71ec6c97447bc6957d9f007f8`.
The surface-adaptor header SHA256 is
`eff24d657fc2e7b39ed3887bb000b22278ef685a04add8d5c94bde5d635a6891`; the cache
header SHA256 is
`d6e54cfaa813f8d8c9e6fccd0ae5747e6b65f100c2b1a4cd4d41436c6e28a23e`.

Build and run from the repository root:

```sh
c++ -std=c++17 -O2 \
  -Itarget/occt-modeling-full/include/opencascade \
  tools/oracle/tkg3d_adaptor_public_surface_data.cxx \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -lTKG3d -lTKMath -lTKernel \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -o /tmp/tkg3d-adaptor-public-surface-data
/tmp/tkg3d-adaptor-public-surface-data
```

Pinned native output:

```text
1 0 0 0 1 1 0 0 0 1 1 1 1 1 0 1 1 1 1 1 1 1 0
1 0.25 0.5 0.125
```

Equivalent Rust assertions are in `crates/tkg3d/tests/geom_adaptor_public_surface_data.rs`.
The shared cache wrapper preserves nullability, clone identity, and mutation; it does
not claim OCCT `Standard_Transient` RTTI parity for `BSplSLib_Cache`.
