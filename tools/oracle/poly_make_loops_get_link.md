# `Poly_MakeLoops::getLink` index behavior

The probe derives from `Poly_MakeLoops2D` to expose protected `getLink`, inserts
one link, and calls it with positive, negative, zero, and missing indices. OCCT
returns the stored link for `1`, toggles `LF_Reversed` for `-1`, and throws
`Standard_OutOfRange` for `0`, `2`, and `-2`. The probe also constructs a null
helper and exercises only `AddLink`/`FindLink`; those methods do not dereference
the helper in this release build. This does not establish null-helper safety for
`Perform`/`chooseLeftWay`: the source dereferences the pointer there. In an
`OCCT_DEBUG` build, `AddLink` also invokes `OnAddLink` through the helper. Both
native 2D and 3D `chooseLeftWay` throw `Standard_NoSuchObject` with message
`NCollection_List::First` when their candidate list is empty; the probe reaches
the fallback in each implementation.

Build and run against OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`:

```sh
g++ -std=c++17 tools/oracle/poly_make_loops_get_link.cxx \
  -I target/occt-modeling-full/include/opencascade \
  -L target/occt-modeling-full/lin64/gcc/libi \
  -Wl,-rpath,$PWD/target/occt-modeling-full/lin64/gcc/libi \
  -lTKMath -lTKernel -o /tmp/poly-make-loops-get-link
/tmp/poly-make-loops-get-link
```

Source SHA-256: `Poly_MakeLoops.hxx`
`788ff54724f54d5b97f86d0a23ec12103271f727cd0284ab457c9ff644d6cc99`,
`Poly_MakeLoops.cxx`
`a5f8f04193600acd91c16b99698d7add57b4070b180f37ea012fcfb5533c04e2`, and
`NCollection_IndexedMap.hxx`
`986806b3bd709e74d0a5b0e9fa7ea1af1bda20faf37f0edb813a66d3c7ca133a`.
The probe SHA-256 is
`f202d1e114fc573185575ea381d527f967c0c1b4dbbb82058c3b95a341b2ec1d`; the linked
`libTKMath.so` SHA-256 is
`7afac5368e0ea361b8a60d2227bc3a1bbe34f9b71ec6c97447bc6957d9f007f8`.

The Rust checked `get_link` helper now delegates missing-index behavior to the
same one-based indexed-map lookup, which reports `Standard_OutOfRange`. The
native `std::abs(INT_MIN)` case is undefined and intentionally remains outside
the parity claim. Rust still requires a non-null helper reference; this safely
excludes native null-helper calls that dereference an invalid pointer.

Compiler-inventory selectors confirmed on the pinned header: protected
`Poly_MakeLoops::getLink(int) const` declaration id
`adc870a79b2da2fde3527c1fe3bf5da91647e64af854b68e7d7b8a9d7820b150`,
`Poly_MakeLoops::chooseLeftWay` id
`f283f473b2f5801506053c24ebe80aa19a83f83365a987639b82f1ef05a8f9b2`, and the
2D/3D overrides ids `2ccb041e0a8c94668371fe011e3da1039c0c1c94d97ad1f2ca8522d993172016`
and `11d0429c11c3102dae015ad38867aad3a59db7f5749a99477ac9b0c12310034d`.
The Rust correspondence is `MakeLoops::get_link` and
`LoopWay::choose_left_way`, implemented by `LeftWay2D`/`LeftWay3D`. No legacy
declaration rows or counts are changed by this slice.
