//! Supplemental native byte message and explicit stack constructor checks.
use kernel::standard::Failure;
use rustycad_tkernel as kernel;

fn assert_stack_constructor(construct: fn(&[u8], &[u8]) -> Failure, name: &str, parent: &str) {
    let error = construct(&[0xff, b'm', 0, b'x'], &[0xfe, b's', 0, b'x']);
    assert_eq!(error.what(), &[0xff, b'm']);
    assert_eq!(error.stack_bytes(), &[0xfe, b's']);
    assert_eq!(error.exception_type(), name);
    assert!(error.is_kind(parent));
    let empty = construct(&[], &[]);
    assert_eq!(empty.what(), b"");
    assert_eq!(empty.stack_bytes(), b"");
}

#[test]
fn standard_abortivetransaction_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::abortive_transaction,
        "Standard_AbortiveTransaction",
        "Standard_Failure",
    );
}

#[test]
fn standard_constructionerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::construction_error,
        "Standard_ConstructionError",
        "Standard_DomainError",
    );
}

#[test]
fn standard_dimensionerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::dimension_error,
        "Standard_DimensionError",
        "Standard_DomainError",
    );
}

#[test]
fn standard_dimensionmismatch_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::dimension_mismatch,
        "Standard_DimensionMismatch",
        "Standard_DimensionError",
    );
}

#[test]
fn standard_dividebyzero_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::divide_by_zero,
        "Standard_DivideByZero",
        "Standard_NumericError",
    );
}

#[test]
fn standard_domainerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::domain_error,
        "Standard_DomainError",
        "Standard_Failure",
    );
}

#[test]
fn standard_immutableobject_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::immutable_object,
        "Standard_ImmutableObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_licenseerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::license_error,
        "Standard_LicenseError",
        "Standard_Failure",
    );
}

#[test]
fn standard_licensenotfound_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::license_not_found,
        "Standard_LicenseNotFound",
        "Standard_LicenseError",
    );
}

#[test]
fn standard_multiplydefined_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::multiply_defined,
        "Standard_MultiplyDefined",
        "Standard_DomainError",
    );
}

#[test]
fn standard_negativevalue_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::negative_value,
        "Standard_NegativeValue",
        "Standard_RangeError",
    );
}

#[test]
fn standard_nomoreobject_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::no_more_object,
        "Standard_NoMoreObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_nosuchobject_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::no_such_object,
        "Standard_NoSuchObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_notimplemented_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::not_implemented,
        "Standard_NotImplemented",
        "Standard_ProgramError",
    );
}

#[test]
fn standard_nullobject_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::null_object,
        "Standard_NullObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_nullvalue_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::null_value,
        "Standard_NullValue",
        "Standard_RangeError",
    );
}

#[test]
fn standard_numericerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::numeric_error,
        "Standard_NumericError",
        "Standard_Failure",
    );
}

#[test]
fn standard_outofrange_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::out_of_range,
        "Standard_OutOfRange",
        "Standard_RangeError",
    );
}

#[test]
fn standard_overflow_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::overflow,
        "Standard_Overflow",
        "Standard_NumericError",
    );
}

#[test]
fn standard_programerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::program_error,
        "Standard_ProgramError",
        "Standard_Failure",
    );
}

#[test]
fn standard_rangeerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::range_error,
        "Standard_RangeError",
        "Standard_DomainError",
    );
}

#[test]
fn standard_typemismatch_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::type_mismatch,
        "Standard_TypeMismatch",
        "Standard_DomainError",
    );
}

#[test]
fn standard_underflow_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::standard::exceptions::exception_stacks::underflow,
        "Standard_Underflow",
        "Standard_NumericError",
    );
}

#[test]
fn stdfail_infinitesolutions_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::std_fail::exception_stacks::infinite_solutions,
        "StdFail_InfiniteSolutions",
        "Standard_Failure",
    );
}

#[test]
fn stdfail_notdone_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::std_fail::exception_stacks::not_done,
        "StdFail_NotDone",
        "Standard_Failure",
    );
}

#[test]
fn stdfail_undefined_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::std_fail::exception_stacks::undefined,
        "StdFail_Undefined",
        "Standard_Failure",
    );
}

#[test]
fn stdfail_undefinedderivative_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::std_fail::exception_stacks::undefined_derivative,
        "StdFail_UndefinedDerivative",
        "Standard_DomainError",
    );
}

#[test]
fn stdfail_undefinedvalue_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::std_fail::exception_stacks::undefined_value,
        "StdFail_UndefinedValue",
        "Standard_DomainError",
    );
}

#[test]
fn plugin_failure_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::plugin::exception_stacks::failure,
        "Plugin_Failure",
        "Standard_Failure",
    );
}

#[test]
fn storage_streamextcharparityerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::storage::exception_stacks::stream_ext_char_parity_error,
        "Storage_StreamExtCharParityError",
        "Storage_StreamReadError",
    );
}

#[test]
fn storage_streamformaterror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::storage::exception_stacks::stream_format_error,
        "Storage_StreamFormatError",
        "Standard_Failure",
    );
}

#[test]
fn storage_streammodeerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::storage::exception_stacks::stream_mode_error,
        "Storage_StreamModeError",
        "Standard_Failure",
    );
}

#[test]
fn storage_streamreaderror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::storage::exception_stacks::stream_read_error,
        "Storage_StreamReadError",
        "Standard_Failure",
    );
}

#[test]
fn storage_streamtypemismatcherror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::storage::exception_stacks::stream_type_mismatch_error,
        "Storage_StreamTypeMismatchError",
        "Storage_StreamReadError",
    );
}

#[test]
fn storage_streamunknowntypeerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::storage::exception_stacks::stream_unknown_type_error,
        "Storage_StreamUnknownTypeError",
        "Storage_StreamReadError",
    );
}

#[test]
fn storage_streamwriteerror_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::storage::exception_stacks::stream_write_error,
        "Storage_StreamWriteError",
        "Standard_Failure",
    );
}

#[test]
fn units_nosuchtype_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::units::exception_stacks::no_such_type,
        "Units_NoSuchType",
        "Standard_NoSuchObject",
    );
}

#[test]
fn units_nosuchunit_preserves_explicit_stack_bytes() {
    assert_stack_constructor(
        kernel::units::exception_stacks::no_such_unit,
        "Units_NoSuchUnit",
        "Standard_NoSuchObject",
    );
}
