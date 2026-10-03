# TKGeomBase native runtime blocker

Pinned OCCT: `3d097a0328e71b826377d4814ab05ec3c3d23871`.
Paired verifier source revision: `65fed4cd`.

The strict paired run executed all 839 mapped native declarations: 838 passed and
`ExtremaPC2d_HyperbolaTest.SingletonInvalidAndValueAccessor` failed. The assertion
at `src/ModelingData/TKGeomBase/GTests/ExtremaPC2d_Hyperbola_Test.cxx:145`
requires exactly zero distance between the evaluator's `Value(-0.7)` and
`ElCLib::Value(-0.7, makeHyperbola())`; this native build reports
`1.1102230246251565e-16`. An isolated native rerun reproduced the failure.
The corresponding Rust legacy test passes with its original exact-zero assertion.
This is a recorded discrepancy, not an approved skip or a successful paired receipt.

## Reproduction

```sh
../RustyCADKernel/target/occt-modeling-full/lin64/gcc/bini/OpenCascadeGTest \
  --gtest_filter=ExtremaPC2d_HyperbolaTest.SingletonInvalidAndValueAccessor
cargo test -p rustycad-tkgeombase --test extrema_pc2d_hyperbola -- \
  --exact ExtremaPC2d_HyperbolaTest__SingletonInvalidAndValueAccessor
```

Build path refers to the original workspace's pinned modeling build. Full run:
`python3 tools/verify_tkmath_legacy.py --toolkit TKGeomBase --source ../OCCT
--build-dir ../RustyCADKernel/target/occt-modeling-full --jobs 2`.
The verifier rejected the native failure before the Rust pairing stage and did not
publish a success report. Integration-worktree logs are under
`target/oracle-tkgeombase-legacy/native-run.log` and
`target/api-proof/tkgeombase-singleton-{native,rust}-recheck.log`.

Next investigation: compare native evaluator and ElCLib arithmetic, compiler
optimization flags and library/test compilation contexts. Do not weaken the test,
modify pinned OCCT, or silently count the failure as parity. Full toolkit acceptance
remains false even if this numerical issue is resolved.

## Optimization isolation

The same unmodified upstream test translation unit was compiled separately with
GCC 13.3.0, linking the same pinned OCCT and GoogleTest shared libraries:

| Test compilation flags | Isolated legacy test result |
| --- | --- |
| `-O2 -g -DNDEBUG` (native suite configuration) | Fails, distance `1.1102230246251565e-16` |
| `-O0 -g -DNDEBUG` | Passes |
| `-O2 -g -DNDEBUG -fno-builtin-sinh -fno-builtin-cosh` | Passes |

`ExtremaPC2d_Hyperbola.hxx:83` evaluates cosh/sinh in a header; `ElCLib.cxx:559`
evaluates them in the shared library. Disabling the compiler's sinh/cosh built-ins
removes the failure in this experiment. This supports an optimization/libm rounding
explanation; it does not establish every exact compiler transformation or excuse the
optimized suite failure. A simple reduced value-only probe did not reproduce the
failure, so the full test's compilation context matters.

Only the test object compilation changed. Pinned source, expected assertions, and
OCCT shared libraries were not modified. These three isolated experiments are not
a replacement for the complete paired toolkit run. Logs and binaries remain in the
integration worktree's `target/api-proof/hyperbola-legacy-O{0,2}*` paths.
