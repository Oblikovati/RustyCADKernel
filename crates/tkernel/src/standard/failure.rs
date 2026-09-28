// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! Owned exception messages with OCCT's printing and shared-copy semantics.
use std::sync::Arc;

/// A transported OCCT failure; `Failure::out_of_range("test").to_string()`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Failure {
    exception_type: &'static str,
    message: Arc<str>,
    stack: Arc<str>,
}
impl Default for Failure {
    fn default() -> Self {
        Self::new("")
    }
}
impl Failure {
    /// Constructs Standard_Failure; `Failure::new("failure").message() == "failure"`.
    pub fn new(message: &str) -> Self {
        Self::with_type("Standard_Failure", message, "")
    }
    /// Constructs Standard_OutOfRange; `Failure::out_of_range("test")`.
    pub fn out_of_range(message: &str) -> Self {
        Self::with_type("Standard_OutOfRange", message, "")
    }
    /// Preserves an exception category and explicit stack; `Failure::with_type("Standard_Failure","bad", "trace")`.
    pub fn with_type(exception_type: &'static str, message: &str, stack: &str) -> Self {
        Self {
            exception_type,
            message: c_string(message),
            stack: c_string(stack),
        }
    }
    /// Returns the exception name; `Failure::default().exception_type() == "Standard_Failure"`.
    pub const fn exception_type(&self) -> &'static str {
        self.exception_type
    }
    /// Borrows the exception message; `Failure::new("test").message() == "test"`.
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Borrows the explicit stack string; `Failure::default().stack_string().is_empty()`.
    pub fn stack_string(&self) -> &str {
        &self.stack
    }
    /// Replaces the message while preserving copies; `failure.set_message("updated")`.
    pub fn set_message(&mut self, message: &str) {
        self.message = c_string(message);
    }
    /// Replaces the stack while preserving copies; `failure.set_stack_string("trace")`.
    pub fn set_stack_string(&mut self, stack: &str) {
        self.stack = c_string(stack);
    }
}
fn c_string(text: &str) -> Arc<str> {
    Arc::from(text.split('\0').next().unwrap_or_default())
}
impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.exception_type)?;
        if !self.message.is_empty() {
            write!(formatter, ": {}", self.message)?;
        }
        formatter.write_str(&self.stack)
    }
}
impl std::error::Error for Failure {}
