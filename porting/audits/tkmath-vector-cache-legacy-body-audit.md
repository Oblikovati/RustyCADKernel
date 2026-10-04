# TKMath vector and B-spline cache legacy test-body audit

Audited against OCCT `3d097a0328e71b826377d4814ab05ec3c3d23871`.

| Native test source | SHA-256 | Rust suite |
| --- | --- | --- |
| `TKMath/GTests/math_Vector_Test.cxx` | `a8f8acfabf5be11e364a49ad53a7e27f7c3d2e549934b6a94a2e74d5bb1a6bc3` | `crates/tkmath/tests/math_vector.rs` and `math_vector/suites_*.rs` |
| `TKMath/GTests/BSplCLib_Cache_Test.cxx` | `a2602f4ec59ecc37049d51db514c9ff83971c940fd6bd3667390949587265c6f` | `crates/tkmath/tests/bsplclib_cache.rs` and `bsplclib_cache/suites_0.rs` |
| `TKMath/GTests/BSplSLib_Cache_Test.cxx` | `1d69a30c86d0ee7e8e0cafcf8bb56c616b19df571e5f2424cab33a82e7635ccc` | `crates/tkmath/tests/bsplslib_cache.rs` and `bsplslib_cache/suites_0.rs` |

The three Rust suites contain a test for every native declaration: 31 vector tests, 7 curve-cache tests, and 8 surface-cache tests. The vector suite exercises `math_VectorBase` behavior through `math_Vector`; OCCT has no separate `math_VectorBase` GTest body in this source set. The cache loops retain the native parameter ranges and increments, cached-versus-direct evaluation order, and `1e-10` comparison tolerance. The vector fixtures and matrix cases retain their native bounds and expected values; `Precision::Confusion()` expectations use `1e-7`.

One body-level omission was found in `MathVectorTest.InitAndAccess`: the Rust test repeated indexed access for the final assertion instead of testing the native `Value(3)` accessor. It now writes through `change_value(3)` and reads through `value(3)`, preserving the native mutation and read assertions. This covers the public base-vector access path without changing expected behavior.

Validation against the pinned native `OpenCascadeGTest` binary using filter `MathVectorTest.*:BSplCLib_CacheTest.*:BSplSLib_CacheTest.*` passed all 46 tests. The matching Rust command `cargo test -p rustycad-tkmath --test math_vector --test bsplclib_cache --test bsplslib_cache` also passed all 46 tests.
