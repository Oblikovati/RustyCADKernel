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

### Code-generation diagnosis

The standard flags are faithful to the pinned build, not a custom oracle compile: `CMAKE_BUILD_TYPE=RelWithDebInfo`, GCC 13.3.0, and `-fexceptions -fPIC -Wall -Wextra -O2 -g -DNDEBUG -std=gnu++17` for both the GTest TU and TKMath. The library object for `ElCLib::HyperbolaValue(double, gp_Ax22d const&, double, double)` disassembles to calls to `cosh@plt` and `sinh@plt`. The test's inline `ExtremaPC2d_Hyperbola::Value(-0.7)` has a compile-time literal argument; under `-O2`, GCC constant-folds the hyperbolic functions into constants in the caller TU.

The call-site/library disassembly check is reproducible with:

```sh
objdump -d -C /tmp/tkgeombase_hyperbola_value_profile | rg 'sinh@plt|cosh@plt|ElCLib::HyperbolaValue'
objdump -d -C --disassemble='ElCLib::HyperbolaValue(double, gp_Ax22d const&, double, double)' \
  target/occt-modeling-full/lin64/gcc/libi/libTKMath.so.8.1.0 | rg 'sinh@plt|cosh@plt'
```

The caller command shows its ElCLib call but no caller-side `sinh`/`cosh` calls; the second shows both PLT calls in TKMath.

`tools/oracle/tkgeombase_hyperbola_value_profile.cxx` reproduces the exact geometry and prints component bits. With the standard flags, `cosh` and the X coordinate match, while Y is `-0.7585837018395335` (`bfe8465153d5bdbd`) in the inline caller and `-0.75858370183953339` (`bfe8465153d5bdbc`) from the shared-library ElCLib call. The one-ULP difference gives distance `1.1102230246251565e-16`. Adding `-fno-builtin-sinh -fno-builtin-cosh` to the test TU makes both paths call libm and produces exact equality; it also changes the pinned `RelWithDebInfo` test profile, so it is diagnostic evidence, not an accepted green workaround.

Build and replay the bit-level probe with the native flags:

```sh
occt=target/occt-modeling-full
c++ -fexceptions -fPIC -Wall -Wextra -O2 -g -DNDEBUG -std=gnu++17 \
  -I"$occt/include/opencascade" tools/oracle/tkgeombase_hyperbola_value_profile.cxx \
  -L"$occt/lin64/gcc/libi" -Wl,-rpath,"$PWD/$occt/lin64/gcc/libi" \
  -lTKGeomBase -lTKMath -lTKernel -o /tmp/tkgeombase_hyperbola_value_profile
/tmp/tkgeombase_hyperbola_value_profile
```

Probe source SHA-256: `bb49ce9d7ea936657972a3ba2fbb982505797f7bf0931e3fb9b93abd6b729a45`.
Standard-profile expected output SHA-256: `a448f719bf7e8b5ec8691643069e543b3ad24b62fd38c0e08937036f484cbbf6`.
The control output with `-fno-builtin-sinh -fno-builtin-cosh` has SHA-256
`9acb3fd1874594824b024f131b6dbf2f62c707884fe1592eb2aff6b261c5f985`.
The control compile is otherwise identical, with those two flags appended. The exact native
GTest rerun still fails under the CMake-generated standard flags. Rust default and release
reruns on the current main-based isolated worktree both pass the original exact assertion.

The standard OCCT exact test therefore remains failing and the TKGeomBase module remains un-green. Passing it would require changing the original exact assertion or compiling that test with nonstandard math flags; neither preserves the currently pinned native test behavior.

The broad TKGeomBase runtime receipt must therefore continue to record 838 native passes and this one failure among 839 declarations. This file is evidence only; it does not authorize a skip or claim full toolkit parity.

## Additional full Debug-profile join (2026-10-05)

A fresh CMake `Debug` build was configured from the same pinned OCCT revision and compiled its native test and toolkit libraries with GCC 13.3.0. The test TU flags were `-fexceptions -fPIC -Wall -Wextra -g -std=gnu++17` (no `-O2`, no `-DNDEBUG`). With that separate profile, the focused `SingletonInvalidAndValueAccessor` test passes unchanged. The TKGeomBase verifier also joined all 839 mapped cases: 839 native tests, 839 Rust translations, and 839 Rust runtime cases passed. The verifier still reports `toolkit_complete: false`; this is the mapped subset, not full toolkit acceptance.

This does not replace or rewrite the standard `RelWithDebInfo` receipt above: its 838/839 result and one-ULP exact-zero failure remain intact. The full Debug profile uses Debug-built libraries as well as Debug-built tests, while the focused standalone `-O0` probe against the existing optimized libraries isolates the caller-TU optimization effect. See `porting/runtime-evidence/tkgeombase-debug-profile-0fcfd33.json` for CMake options, source/build fingerprints, exact command, and joined-case counts.
