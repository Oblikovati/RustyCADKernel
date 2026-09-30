//! OCCT's StdFail package: the exceptions algorithms raise when a result is not available.
use crate::standard::exceptions::define_standard_exceptions;

define_standard_exceptions! {
    STD_FAIL_HIERARCHY;
    infinite_solutions => "StdFail_InfiniteSolutions" : "Standard_Failure";
    not_done => "StdFail_NotDone" : "Standard_Failure";
    undefined => "StdFail_Undefined" : "Standard_Failure";
    undefined_derivative => "StdFail_UndefinedDerivative" : "Standard_DomainError";
    undefined_value => "StdFail_UndefinedValue" : "Standard_DomainError";
}
