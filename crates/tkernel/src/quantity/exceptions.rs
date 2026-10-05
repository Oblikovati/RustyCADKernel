//! Named errors declared by OCCT's Quantity package.
use crate::standard::exceptions::define_standard_exceptions;

define_standard_exceptions! {
    QUANTITY_HIERARCHY;
    date_definition_error => "Quantity_DateDefinitionError" : "Standard_DomainError";
    period_definition_error => "Quantity_PeriodDefinitionError" : "Standard_DomainError";
}

/// Constructs `Quantity_DateDefinitionError` with its default empty message.
/// `let failure = default_date_definition_error();`.
pub fn default_date_definition_error() -> crate::standard::Failure {
    date_definition_error("")
}

/// Constructs `Quantity_PeriodDefinitionError` with its default empty message.
/// `let failure = default_period_definition_error();`.
pub fn default_period_definition_error() -> crate::standard::Failure {
    period_definition_error("")
}
