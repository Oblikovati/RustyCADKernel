// Created on: 1994-02-24
// Created by: Jean Claude VAUTHIER
// Copyright (c) 1994-1999 Matra Datavision
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

//! Conversions through the current unit of a quantity (a resource entry), and the
//! editing of the current resources. A quantity without a current unit leaves the
//! value unchanged.
use super::{SystemUnits, UnitsApi, current_value, key};
use crate::standard::Failure;
use crate::tcollection::AsciiString;

impl UnitsApi {
    /// The current unit of `quantity` after loading `system`.
    fn current_of(
        &mut self,
        system: SystemUnits,
        quantity: &[u8],
    ) -> Result<Option<AsciiString>, Failure> {
        self.check_loading(system)?;
        Ok(self
            .current_units
            .as_ref()
            .and_then(|catalog| current_value(catalog, quantity)))
    }
    /// Current unit to local system; `api.current_to_ls(1.0, b"LENGTH")?`.
    pub fn current_to_ls(&mut self, data: f64, quantity: &[u8]) -> Result<f64, Failure> {
        let Some(current) = self.current_of(self.local_system, quantity)? else {
            return Ok(data);
        };
        let value = self.units.to_si(data, current.as_bytes())?;
        self.local_system_units
            .convert_si_value_to_user_system(&self.units, quantity, value)
    }
    /// Current unit to SI; `api.current_to_si(1.0, b"LENGTH")?`.
    pub fn current_to_si(&mut self, data: f64, quantity: &[u8]) -> Result<f64, Failure> {
        let Some(current) = self.current_of(SystemUnits::Default, quantity)? else {
            return Ok(data);
        };
        self.units.to_si(data, current.as_bytes())
    }
    /// Local system to current unit; `api.current_from_ls(1.0, b"LENGTH")?`.
    pub fn current_from_ls(&mut self, data: f64, quantity: &[u8]) -> Result<f64, Failure> {
        let Some(current) = self.current_of(self.local_system, quantity)? else {
            return Ok(data);
        };
        let value =
            self.local_system_units
                .convert_user_system_value_to_si(&self.units, quantity, data)?;
        self.units.from_si(value, current.as_bytes())
    }
    /// SI to current unit; `api.current_from_si(1.0, b"LENGTH")?`.
    pub fn current_from_si(&mut self, data: f64, quantity: &[u8]) -> Result<f64, Failure> {
        let Some(current) = self.current_of(SystemUnits::Default, quantity)? else {
            return Ok(data);
        };
        self.units.from_si(data, current.as_bytes())
    }
    /// Current unit to `unit`; `api.current_to_any(1.0, b"LENGTH", b"mm")?`.
    pub fn current_to_any(
        &mut self,
        data: f64,
        quantity: &[u8],
        unit: &[u8],
    ) -> Result<f64, Failure> {
        let Some(current) = self.current_of(SystemUnits::Default, quantity)? else {
            return Ok(data);
        };
        self.any_to_any(data, current.as_bytes(), unit)
    }
    /// `unit` to current unit; `api.current_from_any(1.0, b"LENGTH", b"mm")?`.
    pub fn current_from_any(
        &mut self,
        data: f64,
        quantity: &[u8],
        unit: &[u8],
    ) -> Result<f64, Failure> {
        let Some(current) = self.current_of(SystemUnits::Default, quantity)? else {
            return Ok(data);
        };
        self.any_to_any(data, unit, current.as_bytes())
    }
    /// Local system to SI, only for a quantity having a current unit;
    /// `api.ls_to_si(1.0, b"LENGTH")?`.
    pub fn ls_to_si(&mut self, data: f64, quantity: &[u8]) -> Result<f64, Failure> {
        if self.current_of(self.local_system, quantity)?.is_none() {
            return Ok(data);
        }
        self.local_system_units
            .convert_user_system_value_to_si(&self.units, quantity, data)
    }
    /// SI to local system, only for a quantity having a current unit;
    /// `api.si_to_ls(1.0, b"LENGTH")?`.
    pub fn si_to_ls(&mut self, data: f64, quantity: &[u8]) -> Result<f64, Failure> {
        if self.current_of(self.local_system, quantity)?.is_none() {
            return Ok(data);
        }
        self.local_system_units
            .convert_si_value_to_user_system(&self.units, quantity, data)
    }
    /// Sets the current unit of `quantity` in the loaded resources (an override kept
    /// when switching systems back); `api.set_current_unit(b"LENGTH", b"mm")?`.
    pub fn set_current_unit(&mut self, quantity: &[u8], unit: &[u8]) -> Result<(), Failure> {
        self.check_loading(self.local_system)?;
        if let Some(catalog) = &self.current_units {
            catalog
                .borrow_mut()
                .set_resource(&key(quantity), super::c_name(unit));
        }
        Ok(())
    }
    /// The current unit of `quantity`; `Resource_NoSuchResource` when it has none;
    /// `api.current_unit(b"LENGTH")?`.
    pub fn current_unit(&mut self, quantity: &[u8]) -> Result<AsciiString, Failure> {
        self.check_loading(self.local_system)?;
        let catalog = self
            .current_units
            .as_ref()
            .expect("check_loading loads the current units");
        Ok(catalog.borrow().value(&key(quantity))?.clone())
    }
    /// Saves the current-unit overrides into the user resource directory; the native
    /// call ignores the outcome, which is returned here; `api.save()?`.
    pub fn save(&mut self) -> Result<bool, Failure> {
        self.check_loading(self.local_system)?;
        Ok(self
            .current_units
            .as_ref()
            .is_some_and(|catalog| catalog.borrow().save()))
    }
    /// Always false: the native check body is commented out after loading the
    /// defaults; `api.check(b"LENGTH", b"mm")?`.
    pub fn check(&mut self, quantity: &[u8], _unit: &[u8]) -> Result<bool, Failure> {
        self.current_of(SystemUnits::Default, quantity)?;
        Ok(false)
    }
}
