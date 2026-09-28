// Copyright (c) 2025 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! The DEFINE_STANDARD_EXCEPTION classes of Standard, StdFail and Plugin: names, bases,
//! and the catch-by-base question that C++ answers with `catch (Base&)`.
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::exceptions::{self, exception_parent};
use rustycad_tkernel::{plugin, std_fail};

#[test]
fn constructors_carry_the_class_name_and_message() {
    let failure = exceptions::negative_value("length -1");
    assert_eq!(failure.exception_type(), "Standard_NegativeValue");
    assert_eq!(failure.message(), "length -1");
    assert_eq!(failure.to_string(), "Standard_NegativeValue: length -1");
    assert_eq!(std_fail::not_done("").to_string(), "StdFail_NotDone");
    assert_eq!(plugin::failure("x").exception_type(), "Plugin_Failure");
}

#[test]
fn parents_follow_the_declared_class_tree() {
    assert_eq!(
        exception_parent("Standard_OutOfRange"),
        Some("Standard_RangeError")
    );
    assert_eq!(
        exception_parent("Standard_RangeError"),
        Some("Standard_DomainError")
    );
    assert_eq!(
        exception_parent("Standard_DomainError"),
        Some("Standard_Failure")
    );
    assert_eq!(
        exception_parent("StdFail_UndefinedValue"),
        Some("Standard_DomainError")
    );
    assert_eq!(
        exception_parent("Standard_OutOfMemory"),
        Some("Standard_ProgramError")
    );
    assert_eq!(exception_parent("Standard_Failure"), None);
    assert_eq!(exception_parent("Unknown_Error"), None);
}

#[test]
fn is_kind_answers_like_a_catch_by_base() {
    let failure = exceptions::out_of_range("index 4");
    assert!(failure.is_kind("Standard_OutOfRange"));
    assert!(failure.is_kind("Standard_RangeError"));
    assert!(failure.is_kind("Standard_DomainError"));
    assert!(failure.is_kind("Standard_Failure"));
    assert!(!failure.is_kind("Standard_NumericError"));
    assert!(!failure.is_kind("Standard_NegativeValue"));
    assert!(exceptions::license_not_found("").is_kind("Standard_LicenseError"));
    assert!(!exceptions::divide_by_zero("").is_kind("Standard_DomainError"));
}

#[test]
fn classes_of_other_toolkits_are_still_failures() {
    let failure = Failure::with_type("math_NotSquare", "3x2", "");
    assert!(failure.is_kind("math_NotSquare"));
    assert!(failure.is_kind("Standard_Failure"));
    assert!(!failure.is_kind("Standard_DomainError"));
}
