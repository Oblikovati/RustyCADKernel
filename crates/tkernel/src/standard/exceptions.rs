//! The `Standard_*` exception classes that `DEFINE_STANDARD_EXCEPTION(C1, C2)` declares.
//!
//! C++ catches an exception by any of its bases (`catch (Standard_DomainError&)` also
//! catches `Standard_OutOfRange`). Rust transports every exception as one [`Failure`]
//! value, so the class tree is kept as data: [`exception_parent`] names the direct base
//! and [`Failure::is_kind`] answers the question a C++ `catch` clause asks.
use super::Failure;

/// Declares one constructor per exception class and the `(class, base)` table behind it.
///
/// Each constructor is the macro's `C1(const char* theMessage = "")`; the two-argument
/// `C1(theMessage, theStackTrace)` is `Failure::with_type(name, message, stack)`.
macro_rules! define_standard_exceptions {
    ($table:ident; $($function:ident => $name:literal : $parent:literal;)*) => {
        $(
            #[doc = concat!("Constructs `", $name, "` (a `", $parent, "`); `", stringify!($function), "(\"message\")`.")]
            pub fn $function(message: &str) -> $crate::standard::Failure {
                $crate::standard::Failure::with_message($name, message)
            }
        )*
        /// Every class this module declares, with its direct base class.
        pub(crate) const $table: &[(&str, &str)] = &[$(($name, $parent)),*];
    };
}
pub(crate) use define_standard_exceptions;

/// Constructs the native default-message domain exception; `default_domain_error().message() == ""`.
pub fn default_domain_error() -> Failure {
    domain_error("")
}

/// Constructs the native default-message range exception; `default_range_error().message() == ""`.
pub fn default_range_error() -> Failure {
    range_error("")
}

/// Constructs the native default-message unsupported-operation exception; `default_not_implemented().message() == ""`.
pub fn default_not_implemented() -> Failure {
    not_implemented("")
}

define_standard_exceptions! {
    STANDARD_HIERARCHY;
    abortive_transaction => "Standard_AbortiveTransaction" : "Standard_Failure";
    construction_error => "Standard_ConstructionError" : "Standard_DomainError";
    dimension_error => "Standard_DimensionError" : "Standard_DomainError";
    dimension_mismatch => "Standard_DimensionMismatch" : "Standard_DimensionError";
    divide_by_zero => "Standard_DivideByZero" : "Standard_NumericError";
    domain_error => "Standard_DomainError" : "Standard_Failure";
    immutable_object => "Standard_ImmutableObject" : "Standard_DomainError";
    license_error => "Standard_LicenseError" : "Standard_Failure";
    license_not_found => "Standard_LicenseNotFound" : "Standard_LicenseError";
    multiply_defined => "Standard_MultiplyDefined" : "Standard_DomainError";
    negative_value => "Standard_NegativeValue" : "Standard_RangeError";
    no_more_object => "Standard_NoMoreObject" : "Standard_DomainError";
    no_such_object => "Standard_NoSuchObject" : "Standard_DomainError";
    not_implemented => "Standard_NotImplemented" : "Standard_ProgramError";
    null_object => "Standard_NullObject" : "Standard_DomainError";
    null_value => "Standard_NullValue" : "Standard_RangeError";
    numeric_error => "Standard_NumericError" : "Standard_Failure";
    out_of_range => "Standard_OutOfRange" : "Standard_RangeError";
    overflow => "Standard_Overflow" : "Standard_NumericError";
    program_error => "Standard_ProgramError" : "Standard_Failure";
    range_error => "Standard_RangeError" : "Standard_DomainError";
    type_mismatch => "Standard_TypeMismatch" : "Standard_DomainError";
    underflow => "Standard_Underflow" : "Standard_NumericError";
}

/// `Standard_OutOfMemory` is a hand-written class, not a macro one, but has the same base.
const OUT_OF_MEMORY: (&str, &str) = ("Standard_OutOfMemory", "Standard_ProgramError");
/// Its constructor lives beside the `Failure` fields its own message buffer needs.
pub use super::failure::out_of_memory;

/// Direct C++ base of a TKernel exception class, or `None` for `Standard_Failure` and
/// for names this toolkit does not declare; `exception_parent("Standard_OutOfRange")`.
pub fn exception_parent(name: &str) -> Option<&'static str> {
    let tables: [&[(&str, &str)]; 6] = [
        STANDARD_HIERARCHY,
        crate::std_fail::STD_FAIL_HIERARCHY,
        crate::plugin::PLUGIN_HIERARCHY,
        crate::units::UNITS_HIERARCHY,
        crate::storage::STORAGE_HIERARCHY,
        std::slice::from_ref(&OUT_OF_MEMORY),
    ];
    tables
        .into_iter()
        .flatten()
        .find(|(class, _)| *class == name)
        .map(|(_, parent)| *parent)
}

impl Failure {
    /// True when a C++ `catch (Ancestor&)` would catch this exception; every OCCT
    /// exception is a `Standard_Failure`. `out_of_range("").is_kind("Standard_DomainError")`.
    pub fn is_kind(&self, ancestor: &str) -> bool {
        let mut current: Option<&str> = Some(self.exception_type());
        while let Some(class) = current {
            if class == ancestor {
                return true;
            }
            current = exception_parent(class);
        }
        // Classes declared by other toolkits are not in these tables; they still derive
        // from Standard_Failure, which is the root of every OCCT exception.
        ancestor == "Standard_Failure"
    }
}
