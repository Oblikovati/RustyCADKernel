//! Rust side of the exception class tree, type-name and printing cases.
use super::common::{Case, hex, next_bytes};
use rustycad_tkernel::standard::{Failure, exceptions};
use rustycad_tkernel::{plugin, std_fail};
use std::str::SplitWhitespace;

/// Operation name and handler of every case this area answers.
pub const CASES: &[(&str, Case)] = &[("exception_kind", kind_case)];

type Constructor = fn(&str) -> Failure;

/// Column order of the native catch matrix (the X-macro of tkernel_exceptions_probe.cxx).
const CLASSES: &[(&str, Constructor)] = &[
    ("Standard_Failure", Failure::new),
    (
        "Standard_AbortiveTransaction",
        exceptions::abortive_transaction,
    ),
    ("Standard_ConstructionError", exceptions::construction_error),
    ("Standard_DimensionError", exceptions::dimension_error),
    ("Standard_DimensionMismatch", exceptions::dimension_mismatch),
    ("Standard_DivideByZero", exceptions::divide_by_zero),
    ("Standard_DomainError", exceptions::domain_error),
    ("Standard_ImmutableObject", exceptions::immutable_object),
    ("Standard_LicenseError", exceptions::license_error),
    ("Standard_LicenseNotFound", exceptions::license_not_found),
    ("Standard_MultiplyDefined", exceptions::multiply_defined),
    ("Standard_NegativeValue", exceptions::negative_value),
    ("Standard_NoMoreObject", exceptions::no_more_object),
    ("Standard_NoSuchObject", exceptions::no_such_object),
    ("Standard_NotImplemented", exceptions::not_implemented),
    ("Standard_NullObject", exceptions::null_object),
    ("Standard_NullValue", exceptions::null_value),
    ("Standard_NumericError", exceptions::numeric_error),
    ("Standard_OutOfRange", exceptions::out_of_range),
    ("Standard_Overflow", exceptions::overflow),
    ("Standard_ProgramError", exceptions::program_error),
    ("Standard_RangeError", exceptions::range_error),
    ("Standard_TypeMismatch", exceptions::type_mismatch),
    ("Standard_Underflow", exceptions::underflow),
    ("StdFail_InfiniteSolutions", std_fail::infinite_solutions),
    ("StdFail_NotDone", std_fail::not_done),
    ("StdFail_Undefined", std_fail::undefined),
    (
        "StdFail_UndefinedDerivative",
        std_fail::undefined_derivative,
    ),
    ("StdFail_UndefinedValue", std_fail::undefined_value),
    ("Plugin_Failure", plugin::failure),
];

/// "exception_kind <Class> <hex message>": catch matrix, type, message and printed form.
fn kind_case(input: &mut SplitWhitespace<'_>) -> String {
    let name = input.next().expect("exception class");
    let message = next_bytes(input);
    let construct = CLASSES
        .iter()
        .find(|(class, _)| *class == name)
        .unwrap_or_else(|| panic!("exception class {name:?} is not in the probe table"))
        .1;
    // C++ passes the message as a C string, so it ends at the first NUL.
    let text = String::from_utf8_lossy(&message);
    let failure = construct(text.split('\0').next().unwrap_or_default());
    let matrix: String = CLASSES
        .iter()
        .map(|(class, _)| if failure.is_kind(class) { '1' } else { '0' })
        .collect();
    format!(
        "{matrix} {} {} {}",
        failure.exception_type(),
        hex(failure.message().as_bytes()),
        hex(failure.to_string().as_bytes())
    )
}
