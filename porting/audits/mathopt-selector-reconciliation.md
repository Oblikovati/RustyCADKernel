# MathOpt solver selector reconciliation

The `MathOpt` crosswalk had 25 implemented solver/config rows with no compiler
match. The Rust implementations already exist; their selectors used shortened
namespace names (`Config`, `FRPRConfig`, `PSOConfig`) that did not match the
captured declaration types. These rows now use the exact written parameter
types from the compiler declaration capture. The 25 selectors resolve uniquely
to the IDs in `tools/tests/test_mathopt_solver_bindings.py`; no Rust solver
behavior or declaration denominator changed.

## Original test reconciliation

OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871` has 156 MathOpt GTests
across eight source files. Rust has 156 `MathOpt_*` test functions, each named
`<native suite>__<native test>`; set comparison found zero missing and zero
extra tests. The sorted case-name inventory SHA256 is
`b9b02d359b92ef082391e9606fb281ae3a2516b9d90bfcb44b69c348f383b238`.

| Native source | Tests | SHA256 |
| --- | ---: | --- |
| `MathOpt_1D_Comparison_Test.cxx` | 13 | `3366a8a56295ebea0bbebfc11b29c472ae1782dd275291d76a1e22eb36f9886c` |
| `MathOpt_1D_Test.cxx` | 27 | `999262c7659de1c47255557d881fd58ee6297941a92783bf989f04e1dd6f1020` |
| `MathOpt_FRPR_Test.cxx` | 12 | `874f8a55935957d9118f30795c3cf800c157cf4c8b9651d26fc1d720198c2060` |
| `MathOpt_GlobOpt_Test.cxx` | 24 | `f22d48b37abba9f00e101d0335d2540b7ee37734b18cc77f02365549f589ffde` |
| `MathOpt_NDim_Comparison_Test.cxx` | 12 | `54d5c053c082950c1dcb1c18a800bd683795b6fe1a7a09bd3fcc557df709b3c7` |
| `MathOpt_NDim_Test.cxx` | 25 | `32c88ac4f228a8c9e9de484713febc312c582dbafe4cb78390d16e6546f3c31f` |
| `MathOpt_Newton_Test.cxx` | 15 | `5842c4e18eafc8a76d92d602da610d92353841d391f3f00fa192a710128bf2e5` |
| `MathOpt_PSO_Test.cxx` | 28 | `40fc755a70d12c8fbadd119236e609bd18723619bfb4fa0a583709cbf0d7652f` |

## Compiler selector evidence

The binding test loads the captured declarations and header manifest, asserts
all 25 corrected selectors resolve uniquely, pins each declaration ID, and
checks the existing Rust target/status. Input SHA256 values are
`4bc75a502ac655a45bacbc426fb5159028d3fb88deb4829135aca2466cd89095` for
`occt-declarations.jsonl.gz` and
`240a3fce1762bd3ea634922b7f6ee192c1d0c53abf83627aefd74b6b59adbc43` for
`api-header-manifest.json`. The regression source SHA256 is
`92b17aee1b46271c018ce5afec3eddca2800e38209f6e13f234686b3e0e9c71b`.

```sh
PYTHONPATH=tools python3 -m unittest tools.tests.test_mathopt_solver_bindings
```
