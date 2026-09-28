// Copyright (c) 2026 OPEN CASCADE SAS
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

use super::{AsciiString, Failure, ResourceCatalog};
impl ResourceCatalog {
    /// Creates or replaces an explicit override; `catalog.set_resource("key", AsciiString::new("value"))`.
    pub fn set_resource(&mut self, name: &str, value: AsciiString) {
        self.overrides.bind(AsciiString::new(name), value);
    }
    /// Uses the native integer-prefix validation; `catalog.integer("count")?`.
    pub fn integer(&self, name: &str) -> Result<i32, Failure> {
        let value = self.value(name)?;
        if !value.is_integer_value() {
            return Err(type_mismatch(name, "an integer"));
        }
        value.integer_value()
    }
    /// Uses the native real-prefix validation; `catalog.real("tolerance")?`.
    pub fn real(&self, name: &str) -> Result<f64, Failure> {
        let value = self.value(name)?;
        if !value.is_real_value(false) {
            return Err(type_mismatch(name, "a real"));
        }
        value.real_value()
    }
}
fn type_mismatch(name: &str, expected: &str) -> Failure {
    Failure::with_type(
        "Standard_TypeMismatch",
        &format!("Value of resource `{name}` is not {expected}"),
        "",
    )
}
