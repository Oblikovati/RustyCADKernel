# Public 3D curve adaptor records

Pinned OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Header SHA-256: `3c97006c38c06a9e2158df0c5f8b7ba22d72aea8c5807555a59e9ccc707253dd`.

`GeomAdaptor_Curve.hxx` declares public `OffsetData`, `BezierData`, `BSplineData`
and `CurveDataVariant`. Rust previously kept only private evaluation-state forms.
The new public records expose those independently constructible aggregates and
reuse `TKMath::bsplclib::SharedBSplineCache` for nullable, shared mutable caches.
The offset direction defaults to positive X, matching `gp_Dir`.

The additive public aggregate types leave private adaptor evaluation unchanged.
All nine native variant alternatives are constructible. Record/variant copies
retain curve and cache identity; cache rebuilding through one copy is observable
through the other. This is not a claim of `BSplCLib_Cache` inherited RTTI parity;
that separate base obligation remains open.

Compile the supplemental native probe from the repository root:

```sh
g++ -std=c++17 -O0 -g0 tools/oracle/tkg3d_adaptor_public_curve_data.cxx \
  -Itarget/occt-modeling-full/include/opencascade \
  -Ltarget/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,"$PWD/target/occt-modeling-full/lin64/gcc/libi" \
  -lTKG3d -lTKG2d -lTKMath -lTKernel -o /tmp/tkg3d_public_curve_data
/tmp/tkg3d_public_curve_data
```

Output matches `tkg3d_adaptor_public_curve_data.expected`. The corresponding
Rust assertions are in `geom_adaptor_public_curve_data.rs` (3 tests, all passing).
`cargo clippy -p rustycad-tkg3d --all-targets -- -D warnings` also passes.
These supplemental checks add no original legacy declarations or skips.
