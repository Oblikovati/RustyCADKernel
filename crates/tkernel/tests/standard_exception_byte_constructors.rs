//! Byte-message regressions for macro-declared OCCT exceptions.
use rustycad_tkernel as kernel;

#[test]
fn standard_abortivetransaction_retains_native_bytes() {
    let error = kernel::standard::exceptions::exception_bytes::abortive_transaction(
        &[0xff, b'x', 0, b'y'][..],
    );
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_AbortiveTransaction");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(
        kernel::standard::exceptions::abortive_transaction("text").what(),
        b"text"
    );
}

#[test]
fn standard_constructionerror_retains_native_bytes() {
    let error = kernel::standard::exceptions::exception_bytes::construction_error(
        &[0xff, b'x', 0, b'y'][..],
    );
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::construction_error("text").what(),
        b"text"
    );
}

#[test]
fn standard_dimensionerror_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::dimension_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_DimensionError");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::dimension_error("text").what(),
        b"text"
    );
}

#[test]
fn standard_dimensionmismatch_retains_native_bytes() {
    let error = kernel::standard::exceptions::exception_bytes::dimension_mismatch(
        &[0xff, b'x', 0, b'y'][..],
    );
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_DimensionMismatch");
    assert!(error.is_kind("Standard_DimensionError"));
    assert_eq!(
        kernel::standard::exceptions::dimension_mismatch("text").what(),
        b"text"
    );
}

#[test]
fn standard_dividebyzero_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::divide_by_zero(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_DivideByZero");
    assert!(error.is_kind("Standard_NumericError"));
    assert_eq!(
        kernel::standard::exceptions::divide_by_zero("text").what(),
        b"text"
    );
}

#[test]
fn standard_domainerror_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::domain_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(
        kernel::standard::exceptions::domain_error("text").what(),
        b"text"
    );
}

#[test]
fn standard_immutableobject_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::immutable_object(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_ImmutableObject");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::immutable_object("text").what(),
        b"text"
    );
}

#[test]
fn standard_licenseerror_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::license_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_LicenseError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(
        kernel::standard::exceptions::license_error("text").what(),
        b"text"
    );
}

#[test]
fn standard_licensenotfound_retains_native_bytes() {
    let error = kernel::standard::exceptions::exception_bytes::license_not_found(
        &[0xff, b'x', 0, b'y'][..],
    );
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_LicenseNotFound");
    assert!(error.is_kind("Standard_LicenseError"));
    assert_eq!(
        kernel::standard::exceptions::license_not_found("text").what(),
        b"text"
    );
}

#[test]
fn standard_multiplydefined_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::multiply_defined(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_MultiplyDefined");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::multiply_defined("text").what(),
        b"text"
    );
}

#[test]
fn standard_negativevalue_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::negative_value(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_NegativeValue");
    assert!(error.is_kind("Standard_RangeError"));
    assert_eq!(
        kernel::standard::exceptions::negative_value("text").what(),
        b"text"
    );
}

#[test]
fn standard_nomoreobject_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::no_more_object(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_NoMoreObject");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::no_more_object("text").what(),
        b"text"
    );
}

#[test]
fn standard_nosuchobject_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::no_such_object(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::no_such_object("text").what(),
        b"text"
    );
}

#[test]
fn standard_notimplemented_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::not_implemented(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert!(error.is_kind("Standard_ProgramError"));
    assert_eq!(
        kernel::standard::exceptions::not_implemented("text").what(),
        b"text"
    );
}

#[test]
fn standard_nullobject_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::null_object(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_NullObject");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::null_object("text").what(),
        b"text"
    );
}

#[test]
fn standard_nullvalue_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::null_value(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_NullValue");
    assert!(error.is_kind("Standard_RangeError"));
    assert_eq!(
        kernel::standard::exceptions::null_value("text").what(),
        b"text"
    );
}

#[test]
fn standard_numericerror_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::numeric_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_NumericError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(
        kernel::standard::exceptions::numeric_error("text").what(),
        b"text"
    );
}

#[test]
fn standard_outofrange_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::out_of_range(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    assert!(error.is_kind("Standard_RangeError"));
    assert_eq!(
        kernel::standard::exceptions::out_of_range("text").what(),
        b"text"
    );
}

#[test]
fn standard_overflow_retains_native_bytes() {
    let error = kernel::standard::exceptions::exception_bytes::overflow(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_Overflow");
    assert!(error.is_kind("Standard_NumericError"));
    assert_eq!(
        kernel::standard::exceptions::overflow("text").what(),
        b"text"
    );
}

#[test]
fn standard_programerror_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::program_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(
        kernel::standard::exceptions::program_error("text").what(),
        b"text"
    );
}

#[test]
fn standard_rangeerror_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::range_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_RangeError");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::range_error("text").what(),
        b"text"
    );
}

#[test]
fn standard_typemismatch_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::type_mismatch(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_TypeMismatch");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::standard::exceptions::type_mismatch("text").what(),
        b"text"
    );
}

#[test]
fn standard_underflow_retains_native_bytes() {
    let error =
        kernel::standard::exceptions::exception_bytes::underflow(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Standard_Underflow");
    assert!(error.is_kind("Standard_NumericError"));
    assert_eq!(
        kernel::standard::exceptions::underflow("text").what(),
        b"text"
    );
}

#[test]
fn stdfail_infinitesolutions_retains_native_bytes() {
    let error = kernel::std_fail::exception_bytes::infinite_solutions(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "StdFail_InfiniteSolutions");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::std_fail::infinite_solutions("text").what(), b"text");
}

#[test]
fn stdfail_notdone_retains_native_bytes() {
    let error = kernel::std_fail::exception_bytes::not_done(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "StdFail_NotDone");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::std_fail::not_done("text").what(), b"text");
}

#[test]
fn stdfail_undefined_retains_native_bytes() {
    let error = kernel::std_fail::exception_bytes::undefined(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "StdFail_Undefined");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::std_fail::undefined("text").what(), b"text");
}

#[test]
fn stdfail_undefinedderivative_retains_native_bytes() {
    let error = kernel::std_fail::exception_bytes::undefined_derivative(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "StdFail_UndefinedDerivative");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(
        kernel::std_fail::undefined_derivative("text").what(),
        b"text"
    );
}

#[test]
fn stdfail_undefinedvalue_retains_native_bytes() {
    let error = kernel::std_fail::exception_bytes::undefined_value(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "StdFail_UndefinedValue");
    assert!(error.is_kind("Standard_DomainError"));
    assert_eq!(kernel::std_fail::undefined_value("text").what(), b"text");
}

#[test]
fn plugin_failure_retains_native_bytes() {
    let error = kernel::plugin::exception_bytes::failure(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Plugin_Failure");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::plugin::failure("text").what(), b"text");
}

#[test]
fn storage_streamextcharparityerror_retains_native_bytes() {
    let error =
        kernel::storage::exception_bytes::stream_ext_char_parity_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Storage_StreamExtCharParityError");
    assert!(error.is_kind("Storage_StreamReadError"));
    assert_eq!(
        kernel::storage::stream_ext_char_parity_error("text").what(),
        b"text"
    );
}

#[test]
fn storage_streamformaterror_retains_native_bytes() {
    let error = kernel::storage::exception_bytes::stream_format_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Storage_StreamFormatError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::storage::stream_format_error("text").what(), b"text");
}

#[test]
fn storage_streammodeerror_retains_native_bytes() {
    let error = kernel::storage::exception_bytes::stream_mode_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Storage_StreamModeError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::storage::stream_mode_error("text").what(), b"text");
}

#[test]
fn storage_streamreaderror_retains_native_bytes() {
    let error = kernel::storage::exception_bytes::stream_read_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Storage_StreamReadError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::storage::stream_read_error("text").what(), b"text");
}

#[test]
fn storage_streamtypemismatcherror_retains_native_bytes() {
    let error =
        kernel::storage::exception_bytes::stream_type_mismatch_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Storage_StreamTypeMismatchError");
    assert!(error.is_kind("Storage_StreamReadError"));
    assert_eq!(
        kernel::storage::stream_type_mismatch_error("text").what(),
        b"text"
    );
}

#[test]
fn storage_streamunknowntypeerror_retains_native_bytes() {
    let error =
        kernel::storage::exception_bytes::stream_unknown_type_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Storage_StreamUnknownTypeError");
    assert!(error.is_kind("Storage_StreamReadError"));
    assert_eq!(
        kernel::storage::stream_unknown_type_error("text").what(),
        b"text"
    );
}

#[test]
fn storage_streamwriteerror_retains_native_bytes() {
    let error = kernel::storage::exception_bytes::stream_write_error(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Storage_StreamWriteError");
    assert!(error.is_kind("Standard_Failure"));
    assert_eq!(kernel::storage::stream_write_error("text").what(), b"text");
}

#[test]
fn units_nosuchtype_retains_native_bytes() {
    let error = kernel::units::exception_bytes::no_such_type(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Units_NoSuchType");
    assert!(error.is_kind("Standard_NoSuchObject"));
    assert_eq!(kernel::units::no_such_type("text").what(), b"text");
}

#[test]
fn units_nosuchunit_retains_native_bytes() {
    let error = kernel::units::exception_bytes::no_such_unit(&[0xff, b'x', 0, b'y'][..]);
    assert_eq!(error.what(), &[0xff, b'x']);
    assert_eq!(error.exception_type(), "Units_NoSuchUnit");
    assert!(error.is_kind("Standard_NoSuchObject"));
    assert_eq!(kernel::units::no_such_unit("text").what(), b"text");
}
