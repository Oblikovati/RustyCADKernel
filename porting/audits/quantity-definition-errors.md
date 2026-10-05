# Quantity date and period exception classes

OCCT revision `3d097a0328e71b826377d4814ab05ec3c3d23871` declares
`Quantity_DateDefinitionError` and `Quantity_PeriodDefinitionError` with
`DEFINE_STANDARD_EXCEPTION(..., Standard_DomainError)`. Their checked-in Rust
counterparts now use the shared exception macro under `quantity::exceptions`;
the hierarchy table feeds `exception_parent` and `Failure::is_kind`.

Date and Period continue returning the existing `Error` type. Conversion from
`Error` to `Failure` preserves the exception name and C-string message. An
embedded NUL terminates that message as it does in the native constructor.
`Error` does not carry the native throw-site stack, so this conversion cannot
restore it.

## Compiler-bound declarations

The parity ledger includes the six macro-generated members separately from the
class rows: each one-argument constructor (including its Rust default helper),
each explicit message/stack constructor, and each `ExceptionType() const
noexcept` query. `tools/tests/test_quantity_definition_error_bindings.py`
checks that every selector resolves to exactly one captured TKernel declaration
and the expected Rust path. The captured declaration IDs are `0a3c56d6` /
`8ecc720c` for one-argument constructors, `b81d2db7` / `d311f8da` for explicit
stack constructors, and `629b0b41` / `cfd81e71` for exception type queries
(Date / Period respectively).

## Native probe

The probe catches each named exception through `Standard_DomainError` and
prints `ExceptionType()`, `what()`, and explicit stack text. Captured output is:

```text
Quantity_DateDefinitionError:date parameters
Quantity_PeriodDefinitionError:period values
Quantity_DateDefinitionError:
Quantity_PeriodDefinitionError:
Quantity_DateDefinitionError:date failure:date trace
Quantity_PeriodDefinitionError:period failure:period trace
```

Source SHA256 values are `55ec4f77b42d57dc77249510cb2091b64fdb08cf0b67e120e8d0ca0b2836cd3e`
for the date header and `b40be0b818562248126e9bd34c7e5ec6030c8f15cf243ff3b5acd79f725c1f95`
for the period header. The native probe SHA256 is
`15d1af2de86c27fac0ddc15d7461fdd8c596c20ba5761b5949e25a4b4e60930c`, and its
expected output SHA256 is
`3e2dd3ee1c0d748b7952fcfe27fd7a30ca4a2170de946725cce48ef92cdf0b16`.
The probe linked `libTKMath.so.8.1.0` SHA256
`10e1965fdd50486bf85d905493d84234878b828a0e2d06381ce6fedab7b40243` from the
pinned foundation build. `Standard_DefineException.hxx` has SHA256
`718d83d7d37b327bf991526be73f0cf41ba6b745cbfbd0ed7398d114c5cf1d8a`. The
probe was compiled by `g++ (Ubuntu 13.3.0-6ubuntu2~24.04.1) 13.3.0`; the OCCT
working copy is at revision `3d097a0328e71b826377d4814ab05ec3c3d23871`.

```sh
g++ -std=c++17 -O2 -Wall -Wextra -Werror \
  -I/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/include/opencascade \
  tools/oracle/quantity_definition_errors.cxx \
  -L/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi \
  -lTKMath -lTKernel \
  -Wl,-rpath,/home/vmiguel/git/oblikovati-workspace/RustyCADKernel/target/occt-foundation/lin64/gcc/libi \
  -o /tmp/quantity_definition_errors
/tmp/quantity_definition_errors > /tmp/quantity_definition_errors.out
diff -u tools/oracle/quantity_definition_errors.expected /tmp/quantity_definition_errors.out
```

## Rust validation

Pinned Rust `1.98.1`, with incremental compilation and debug symbols disabled:

```sh
$HOME/.cargo/bin/cargo test --locked -p rustycad-tkernel --lib \
  --test quantity_definition_errors \
  --test standard_exceptions_behavior --test boundaries \
  --test quantity_api_behavior --test osd_process_api_behavior
$HOME/.cargo/bin/cargo clippy --locked -p rustycad-tkernel --lib \
  --test quantity_definition_errors \
  --test standard_exceptions_behavior --test boundaries \
  --test quantity_api_behavior --test osd_process_api_behavior -- -D warnings
$HOME/.cargo/bin/cargo test --locked --doc -p rustycad-tkernel
PYTHONPATH=tools python3 -m unittest tools.tests.test_quantity_definition_error_bindings
```

The Rust test command passed 106 library tests and 44 tests across the five
integration targets. The new test target covers string, default, byte,
explicit-stack, and existing Date/Period `Error` conversion behavior; original
test targets and names are unchanged. Strict Clippy passed, the doc-test command
passed 250 tests, and the compiler-binding regression passed. The new exception
module source has SHA256
`8f7fc4bf69d4805287705421388a396770b998a73585faf16955ce6ec68167ac` and its
supplemental test source has SHA256
`2042be20c5e7c32ae7cb18d560d56e38fddf3dac1f88c4b722aab3b744a67d60`;
the selector regression source has SHA256
`81ff9496f66801496a9463333f5fcffb3aa917b1c7d4e6aae4799541f809310d`.
