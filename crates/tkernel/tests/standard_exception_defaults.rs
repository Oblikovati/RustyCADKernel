//! Native default arguments preserve the dynamic exception type and empty message.
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::{
    default_domain_error, default_not_implemented, default_range_error,
};

fn assert_default_exception(failure: Failure, class: &str, parent: &str) {
    assert_eq!(failure.exception_type(), class);
    assert_eq!(failure.message(), "");
    assert_eq!(failure.what(), b"");
    assert!(failure.is_kind(parent));
    assert_eq!(failure.to_string(), class);
    let native_row = format!("{}|{}|1|{}", class, failure.message(), failure);
    assert!(
        include_str!("fixtures/standard_exception_defaults.txt")
            .lines()
            .any(|row| row == native_row)
    );
}

#[test]
fn native_domain_error_default_argument() {
    assert_default_exception(
        default_domain_error(),
        "Standard_DomainError",
        "Standard_Failure",
    );
}

#[test]
fn native_range_error_default_argument() {
    assert_default_exception(
        default_range_error(),
        "Standard_RangeError",
        "Standard_DomainError",
    );
}

#[test]
fn native_not_implemented_default_argument() {
    assert_default_exception(
        default_not_implemented(),
        "Standard_NotImplemented",
        "Standard_ProgramError",
    );
}
