# MathUtils arithmetic profile

The initial differential profile is the pinned OCCT build on Linux x86-64 with
GCC and the C locale. `MathUtils::DotProduct` uses a compensated C++ `long double`
accumulator. Replacing those intermediate operations with `f64` changes results,
including cancellation and products outside the binary64 exponent range.

The Rust port isolates `rustc_apfloat` behind `math_utils::extended`. It uses
`X87DoubleExtended` for the oracle's 64-bit significand and extended exponent
range, then rounds the final result to binary64. This is pure Rust arithmetic;
production code does not link LLVM or OCCT.

The pinned crate is `rustc_apfloat` 0.2.3+llvm-462a31f5a5ab, checked on
2026-09-28 through Cargo and the [rust-lang upstream repository](https://github.com/rust-lang/rustc_apfloat).
Its [type documentation](https://docs.rs/rustc_apfloat/0.2.3+llvm-462a31f5a5ab/rustc_apfloat/ieee/type.X87DoubleExtended.html)
describes the x87 format. The dependency license is Apache-2.0 WITH LLVM-exception;
`cargo deny check licenses` passes with that explicit license allowed.

The adapter currently preserves this selected oracle profile across Rust targets.
Native platforms whose C++ `long double` differs need their own runtime oracle
profile before platform-specific arithmetic parity can be claimed. Cross-target
compilation checks do not establish that runtime parity.

## Current verification

The numeric corpus passes 16,599 comparisons, including 259 MathUtils vector
arithmetic cases and 128 bracketing cases with callback-order and partial-result
checks, plus 128 line-search and derivative cases. Finite values, infinities and signed zeros match bit for bit; NaNs are
compared by classification with raw differences retained. Supplemental base,
bracketing and polynomial tests pass. All 16 declarations in `MathUtils_Bracket_Test.cxx` are now translated and pass
in Rust. The full toolkit oracle gate also checks their unchanged native counterparts.


Scalar and vector functor differential probes add 256 cases each. Modern scalar
root solvers add 256, modern Powell/BFGS add 128, and scalar minimizers add 256.
The 961-declaration TKMath gate validates these probes alongside the complete
mapped legacy source suites.

The software x87 square-root adapter rounds the full 64-bit significand using
integer arithmetic. Its 2,048 native `sqrtl` fixtures cover normal and subnormal
inputs across the exponent range. The fixture generator is retained in
`tools/oracle/extended_sqrt_fixture.cxx`.

The modern random generator preserves OCCT's xoshiro256** sequence and SplitMix64
seed expansion. A separate 320-row fixture checks both integer outputs and the
bits of alternating real outputs, including zero and maximum 64-bit seeds.
