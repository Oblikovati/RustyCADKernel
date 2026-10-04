# TKGeomBase hyperbola singleton exact-zero profile recheck

This records a reproducible native/Rust result mismatch. It is not a skip, a tolerance change, or a green parity receipt.

## Pinned inputs

- OCCT revision: `3d097a0328e71b826377d4814ab05ec3c3d23871`
- Rust workspace revision for paired rerun: `462fc6bcdac79ffa947025d8baf687db86ba3ddf`
- C++ compiler: `/usr/bin/c++`, GCC 13.3.0
- Native build: `target/occt-modeling-full`, CMake `RelWithDebInfo`; GTest TU flags are `-fexceptions -fPIC -Wall -Wextra -O2 -g -DNDEBUG -std=gnu++17`.
- Test source: `src/ModelingData/TKGeomBase/GTests/ExtremaPC2d_Hyperbola_Test.cxx`, SHA-256 `e890d8c136e29c51611de2ae1dca23e1c6df75ecfae612879b5911a2b576d646`
- Evaluator header: `src/ModelingData/TKGeomBase/ExtremaPC2d/ExtremaPC2d_Hyperbola.hxx`, SHA-256 `751bbd0cdcb6ede73ac37b2874626333ec6730f004f49ed04dceee7d33c40c6c`
- `ElCLib.cxx`: SHA-256 `ffa5c09ed0695c2ad18b28e8c19615bfbfc639a0169bf2bc01dcc72733edcb83`
- `ElCLib.lxx`: SHA-256 `351a4bd7c949da97168fdbe076cd4a2a99c1d09df193b599cc7b5607bbf6e84d`
- Rust test: `crates/tkgeombase/tests/extrema_pc2d_hyperbola.rs`, SHA-256 `21769b749930960e8f638dabe2e246e0384c62f99e6f358f5beb89bae1f6f18c`
- Rust evaluator: `crates/tkgeombase/src/extrema_pc2d/hyperbola.rs`, SHA-256 `9ffca9630779cb1fc5cd2f1fa7fd28ca725605fe71e6339a6b845f267fae9451`
- Rust ElCLib planar evaluators: `crates/tkmath/src/elclib/plane.rs`, SHA-256 `0b0299cea1a90dbff4fe69e201737c752b29a15b2aadae1edc18ba67c996aa46`

## Exact test and results

The native declaration is `ExtremaPC2d_HyperbolaTest.SingletonInvalidAndValueAccessor`, source lines 138–146. It builds `makeHyperbola()` as an origin-centered `gp_Hypr2d` with X direction `(1,0)`, direct orientation, major radius `2`, minor radius `1`; creates a singleton domain `(0.4,0.4)`; and at line 145 asserts:

```cpp
EXPECT_EQ(aSingleton.Value(-0.7).Distance(ElCLib::Value(-0.7, makeHyperbola())), 0.0);
```

The Rust declaration is `ExtremaPC2d_HyperbolaTest__SingletonInvalidAndValueAccessor` in `crates/tkgeombase/tests/extrema_pc2d_hyperbola.rs`; it retains the same exact-zero assertion using `aSingleton.value(-0.7).distance(elclib::value(-0.7, ...))`.

| Executable/profile | Result |
|---|---|
| Existing pinned `OpenCascadeGTest`, standard `-O2` build | Fails: distance `1.1102230246251565e-16`, expected `0.0` |
| Standalone compile of the exact native test TU, GCC 13.3 `-O2 -g -DNDEBUG -std=gnu++17` | Fails with the same `1.1102230246251565e-16` |
| Same TU, `-O0 -g -DNDEBUG` | Passes |
| Same TU, `-O2 -g -DNDEBUG -fno-builtin-sinh -fno-builtin-cosh` | Passes |
| Rust exact test, default Cargo test profile | Passes |
| Rust exact test, Cargo `--release` | Passes |

Reproduction against the built native suite:

```sh
target/occt-modeling-full/lin64/gcc/bini/OpenCascadeGTest \
  --gtest_filter=ExtremaPC2d_HyperbolaTest.SingletonInvalidAndValueAccessor
```

Rust default and optimized test invocations:

```sh
cargo test --locked -p rustycad-tkgeombase --test extrema_pc2d_hyperbola -- \
  --exact ExtremaPC2d_HyperbolaTest__SingletonInvalidAndValueAccessor
cargo test --locked --release -p rustycad-tkgeombase --test extrema_pc2d_hyperbola -- \
  --exact ExtremaPC2d_HyperbolaTest__SingletonInvalidAndValueAccessor
```

Standalone native profile reproduction used the exact OCCT test translation unit and the same pinned built OCCT libraries. The source and library paths are under the native build above; include paths are `target/occt-modeling-full/include/opencascade` and `target/occt-modeling-full/_deps/googletest-src/googletest/{include,.}`; link inputs are `libgtest_main`, `libgtest`, `libTKGeomBase`, `libTKG2d`, `libTKMath`, `libTKernel`, and `pthread`. Flags changed only between the three table rows.

## Interpretation

This is a real observed profile-specific output discrepancy: the standard optimized OCCT test fails its own exact assertion while both Rust profiles return zero. It is not evidence of a missing extrema branch. The native evaluator's `Value()` is inline in `ExtremaPC2d_Hyperbola.hxx` and uses cached center/axis/radii; `ElCLib::Value()` dispatches to `ElCLib::HyperbolaValue()` implemented in the shared library. Both express the same hyperbola formula, but the test compares computations compiled in different translation units. The sensitivity to optimization and `cosh`/`sinh` built-ins indicates a last-bit/compiler-libm effect. The exact failure remains part of pinned OCCT test behavior and remains unresolved for the standard profile; no native source, Rust behavior, assertion, or profile was modified to suppress it.

The broad TKGeomBase runtime receipt must therefore continue to record 838 native passes and this one failure among 839 declarations. This file is evidence only; it does not authorize a skip or claim full toolkit parity.
