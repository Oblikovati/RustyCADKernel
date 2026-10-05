//! Quantity exception constructors and catch-by-base hierarchy.
use rustycad_tkernel::Error;
use rustycad_tkernel::quantity::Date;
use rustycad_tkernel::quantity::Period;
use rustycad_tkernel::quantity::exceptions;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::exception_parent;

fn assert_quantity_domain_error(failure: &Failure, name: &str) {
    assert_eq!(failure.exception_type(), name);
    assert!(failure.is_kind("Standard_DomainError"));
    assert!(failure.is_kind("Standard_Failure"));
    assert!(!failure.is_kind("Standard_RangeError"));
}

#[test]
fn string_constructors_preserve_quantity_type_and_parent() {
    let date = exceptions::date_definition_error("bad date");
    let period = exceptions::period_definition_error("bad period");
    assert_quantity_domain_error(&date, "Quantity_DateDefinitionError");
    assert_quantity_domain_error(&period, "Quantity_PeriodDefinitionError");
    assert_eq!(date.message(), "bad date");
    assert_eq!(period.message(), "bad period");
}

#[test]
fn default_constructors_use_the_native_empty_message() {
    let date = exceptions::default_date_definition_error();
    let period = exceptions::default_period_definition_error();
    assert_quantity_domain_error(&date, "Quantity_DateDefinitionError");
    assert_quantity_domain_error(&period, "Quantity_PeriodDefinitionError");
    assert_eq!(date.message(), "");
    assert_eq!(period.message(), "");
}

#[test]
fn byte_and_stack_constructors_are_generated_for_both_classes() {
    let date_bytes = exceptions::exception_bytes::date_definition_error(b"\xff\0ignored");
    let period_bytes = exceptions::exception_bytes::period_definition_error(b"\xfe\0ignored");
    let date_stack = exceptions::exception_stacks::date_definition_error(b"date", b"\xff");
    let period_stack = exceptions::exception_stacks::period_definition_error(b"period", b"\xfe");
    assert_quantity_domain_error(&date_bytes, "Quantity_DateDefinitionError");
    assert_quantity_domain_error(&period_bytes, "Quantity_PeriodDefinitionError");
    assert_quantity_domain_error(&date_stack, "Quantity_DateDefinitionError");
    assert_quantity_domain_error(&period_stack, "Quantity_PeriodDefinitionError");
    assert_eq!(date_bytes.what(), b"\xff");
    assert_eq!(period_bytes.what(), b"\xfe");
    assert_eq!(date_stack.stack_bytes(), b"\xff");
    assert_eq!(period_stack.stack_bytes(), b"\xfe");
}

#[test]
fn existing_date_and_period_errors_convert_without_changing_their_api() {
    let date_error: Error = Date::new(0, 1, 2000, 0, 0, 0, 0, 0).unwrap_err();
    let period_error: Error = Period::new(-1, 0).unwrap_err();
    let date_failure: Failure = date_error.into();
    let period_failure: Failure = period_error.into();
    assert_quantity_domain_error(&date_failure, "Quantity_DateDefinitionError");
    assert_quantity_domain_error(&period_failure, "Quantity_PeriodDefinitionError");
    assert_eq!(
        date_failure.message(),
        "Quantity_Date::Quantity_Date invalid parameters"
    );
    assert_eq!(
        period_failure.message(),
        "Quantity_Period::SetValues invalid parameters"
    );
    assert_eq!(
        exception_parent(date_failure.exception_type()),
        Some("Standard_DomainError")
    );
}
