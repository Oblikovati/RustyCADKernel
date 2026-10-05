//! Byte-message regressions for macro-declared OCCT exceptions.
use kernel::standard::Failure;
use rustycad_tkernel as kernel;

fn assert_byte_constructor(
    construct: fn(&[u8]) -> Failure,
    text_construct: fn(&str) -> Failure,
    name: &str,
    parent: &str,
) {
    let error = construct(&[0xff, b'x', 0, b'y']);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), name);
    assert!(error.is_kind(parent));
    assert_eq!(text_construct("text").what(), b"text");
}

#[test]
fn standard_abortivetransaction_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::abortive_transaction,
        kernel::standard::exceptions::abortive_transaction,
        "Standard_AbortiveTransaction",
        "Standard_Failure",
    );
}

#[test]
fn standard_constructionerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::construction_error,
        kernel::standard::exceptions::construction_error,
        "Standard_ConstructionError",
        "Standard_DomainError",
    );
}

#[test]
fn standard_dimensionerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::dimension_error,
        kernel::standard::exceptions::dimension_error,
        "Standard_DimensionError",
        "Standard_DomainError",
    );
}

#[test]
fn standard_dimensionmismatch_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::dimension_mismatch,
        kernel::standard::exceptions::dimension_mismatch,
        "Standard_DimensionMismatch",
        "Standard_DimensionError",
    );
}

#[test]
fn standard_dividebyzero_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::divide_by_zero,
        kernel::standard::exceptions::divide_by_zero,
        "Standard_DivideByZero",
        "Standard_NumericError",
    );
}

#[test]
fn standard_domainerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::domain_error,
        kernel::standard::exceptions::domain_error,
        "Standard_DomainError",
        "Standard_Failure",
    );
}

#[test]
fn standard_immutableobject_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::immutable_object,
        kernel::standard::exceptions::immutable_object,
        "Standard_ImmutableObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_licenseerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::license_error,
        kernel::standard::exceptions::license_error,
        "Standard_LicenseError",
        "Standard_Failure",
    );
}

#[test]
fn standard_licensenotfound_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::license_not_found,
        kernel::standard::exceptions::license_not_found,
        "Standard_LicenseNotFound",
        "Standard_LicenseError",
    );
}

#[test]
fn standard_multiplydefined_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::multiply_defined,
        kernel::standard::exceptions::multiply_defined,
        "Standard_MultiplyDefined",
        "Standard_DomainError",
    );
}

#[test]
fn standard_negativevalue_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::negative_value,
        kernel::standard::exceptions::negative_value,
        "Standard_NegativeValue",
        "Standard_RangeError",
    );
}

#[test]
fn standard_nomoreobject_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::no_more_object,
        kernel::standard::exceptions::no_more_object,
        "Standard_NoMoreObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_nosuchobject_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::no_such_object,
        kernel::standard::exceptions::no_such_object,
        "Standard_NoSuchObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_notimplemented_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::not_implemented,
        kernel::standard::exceptions::not_implemented,
        "Standard_NotImplemented",
        "Standard_ProgramError",
    );
}

#[test]
fn standard_nullobject_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::null_object,
        kernel::standard::exceptions::null_object,
        "Standard_NullObject",
        "Standard_DomainError",
    );
}

#[test]
fn standard_nullvalue_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::null_value,
        kernel::standard::exceptions::null_value,
        "Standard_NullValue",
        "Standard_RangeError",
    );
}

#[test]
fn standard_numericerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::numeric_error,
        kernel::standard::exceptions::numeric_error,
        "Standard_NumericError",
        "Standard_Failure",
    );
}

#[test]
fn standard_outofrange_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::out_of_range,
        kernel::standard::exceptions::out_of_range,
        "Standard_OutOfRange",
        "Standard_RangeError",
    );
}

#[test]
fn standard_overflow_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::overflow,
        kernel::standard::exceptions::overflow,
        "Standard_Overflow",
        "Standard_NumericError",
    );
}

#[test]
fn standard_programerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::program_error,
        kernel::standard::exceptions::program_error,
        "Standard_ProgramError",
        "Standard_Failure",
    );
}

#[test]
fn standard_rangeerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::range_error,
        kernel::standard::exceptions::range_error,
        "Standard_RangeError",
        "Standard_DomainError",
    );
}

#[test]
fn standard_typemismatch_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::type_mismatch,
        kernel::standard::exceptions::type_mismatch,
        "Standard_TypeMismatch",
        "Standard_DomainError",
    );
}

#[test]
fn standard_underflow_retains_native_bytes() {
    assert_byte_constructor(
        kernel::standard::exceptions::exception_bytes::underflow,
        kernel::standard::exceptions::underflow,
        "Standard_Underflow",
        "Standard_NumericError",
    );
}

#[test]
fn stdfail_infinitesolutions_retains_native_bytes() {
    assert_byte_constructor(
        kernel::std_fail::exception_bytes::infinite_solutions,
        kernel::std_fail::infinite_solutions,
        "StdFail_InfiniteSolutions",
        "Standard_Failure",
    );
}

#[test]
fn stdfail_notdone_retains_native_bytes() {
    assert_byte_constructor(
        kernel::std_fail::exception_bytes::not_done,
        kernel::std_fail::not_done,
        "StdFail_NotDone",
        "Standard_Failure",
    );
}

#[test]
fn stdfail_undefined_retains_native_bytes() {
    assert_byte_constructor(
        kernel::std_fail::exception_bytes::undefined,
        kernel::std_fail::undefined,
        "StdFail_Undefined",
        "Standard_Failure",
    );
}

#[test]
fn stdfail_undefinedderivative_retains_native_bytes() {
    assert_byte_constructor(
        kernel::std_fail::exception_bytes::undefined_derivative,
        kernel::std_fail::undefined_derivative,
        "StdFail_UndefinedDerivative",
        "Standard_DomainError",
    );
}

#[test]
fn stdfail_undefinedvalue_retains_native_bytes() {
    assert_byte_constructor(
        kernel::std_fail::exception_bytes::undefined_value,
        kernel::std_fail::undefined_value,
        "StdFail_UndefinedValue",
        "Standard_DomainError",
    );
}

#[test]
fn plugin_failure_retains_native_bytes() {
    assert_byte_constructor(
        kernel::plugin::exception_bytes::failure,
        kernel::plugin::failure,
        "Plugin_Failure",
        "Standard_Failure",
    );
}

#[test]
fn storage_streamextcharparityerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::storage::exception_bytes::stream_ext_char_parity_error,
        kernel::storage::stream_ext_char_parity_error,
        "Storage_StreamExtCharParityError",
        "Storage_StreamReadError",
    );
}

#[test]
fn storage_streamformaterror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::storage::exception_bytes::stream_format_error,
        kernel::storage::stream_format_error,
        "Storage_StreamFormatError",
        "Standard_Failure",
    );
}

#[test]
fn storage_streammodeerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::storage::exception_bytes::stream_mode_error,
        kernel::storage::stream_mode_error,
        "Storage_StreamModeError",
        "Standard_Failure",
    );
}

#[test]
fn storage_streamreaderror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::storage::exception_bytes::stream_read_error,
        kernel::storage::stream_read_error,
        "Storage_StreamReadError",
        "Standard_Failure",
    );
}

#[test]
fn storage_streamtypemismatcherror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::storage::exception_bytes::stream_type_mismatch_error,
        kernel::storage::stream_type_mismatch_error,
        "Storage_StreamTypeMismatchError",
        "Storage_StreamReadError",
    );
}

#[test]
fn storage_streamunknowntypeerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::storage::exception_bytes::stream_unknown_type_error,
        kernel::storage::stream_unknown_type_error,
        "Storage_StreamUnknownTypeError",
        "Storage_StreamReadError",
    );
}

#[test]
fn storage_streamwriteerror_retains_native_bytes() {
    assert_byte_constructor(
        kernel::storage::exception_bytes::stream_write_error,
        kernel::storage::stream_write_error,
        "Storage_StreamWriteError",
        "Standard_Failure",
    );
}

#[test]
fn units_nosuchtype_retains_native_bytes() {
    assert_byte_constructor(
        kernel::units::exception_bytes::no_such_type,
        kernel::units::no_such_type,
        "Units_NoSuchType",
        "Standard_NoSuchObject",
    );
}

#[test]
fn units_nosuchunit_retains_native_bytes() {
    assert_byte_constructor(
        kernel::units::exception_bytes::no_such_unit,
        kernel::units::no_such_unit,
        "Units_NoSuchUnit",
        "Standard_NoSuchObject",
    );
}
