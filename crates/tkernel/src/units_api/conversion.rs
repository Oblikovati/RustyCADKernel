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

//! Conversions of a value expressed in any unit to SI or to the local system.
use super::{SystemUnits, UnitsApi};
use crate::standard::{Failure, Handle};
use crate::units::Dimensions;

/// Dimensions the native code dereferences unchecked (a null handle crashes there).
fn required(dimensions: &Handle<Dimensions>) -> Result<&Dimensions, Failure> {
    dimensions.get().ok_or_else(|| {
        crate::standard::exceptions::null_object(
            "UnitsAPI: null dimensions of an invalid unit; expected an evaluable unit",
        )
    })
}

impl UnitsApi {
    /// `unit` to the local system: SI, then the local unit of the quantity the
    /// dimensions name; an invalid unit gives 0; `api.any_to_ls(3.0, b"mm")?`.
    pub fn any_to_ls(&mut self, data: f64, unit: &[u8]) -> Result<f64, Failure> {
        self.check_loading(self.local_system)?;
        let mut dimensions = Handle::default();
        let value = self.units.to_si_dimensions(data, unit, &mut dimensions)?;
        let Some(dims) = dimensions.get() else {
            return Ok(value);
        };
        self.to_local(dims, value)
    }
    /// `any_to_ls` also returning the dimensions. The native overload asks the
    /// (possibly still null) dimensions for their quantity before its null check, so a
    /// null result is a `Standard_NullObject` error here; `api.any_to_ls_dimensions(1.0, b"m", &mut d)?`.
    pub fn any_to_ls_dimensions(
        &mut self,
        data: f64,
        unit: &[u8],
        dimensions: &mut Handle<Dimensions>,
    ) -> Result<f64, Failure> {
        self.check_loading(self.local_system)?;
        let value = self.units.to_si_dimensions(data, unit, dimensions)?;
        let dims = required(dimensions)?;
        self.to_local(dims, value)
    }
    fn to_local(&self, dimensions: &Dimensions, value: f64) -> Result<f64, Failure> {
        let Some(quantity) = dimensions.quantity(&self.units)? else {
            return Ok(value);
        };
        self.local_system_units.convert_si_value_to_user_system(
            &self.units,
            quantity.as_bytes(),
            value,
        )
    }
    /// `unit` to SI (0 for an invalid unit); `api.any_to_si(1.0, b"min")?`.
    pub fn any_to_si(&mut self, data: f64, unit: &[u8]) -> Result<f64, Failure> {
        self.check_loading(SystemUnits::Default)?;
        self.units.to_si(data, unit)
    }
    /// `any_to_si` also returning the dimensions; `api.any_to_si_dimensions(1.0, b"m", &mut d)?`.
    pub fn any_to_si_dimensions(
        &mut self,
        data: f64,
        unit: &[u8],
        dimensions: &mut Handle<Dimensions>,
    ) -> Result<f64, Failure> {
        self.check_loading(SystemUnits::Default)?;
        self.units.to_si_dimensions(data, unit, dimensions)
    }
    /// Local system to `unit`. The native code converts SI -> `unit` first and then
    /// applies the local-to-SI conversion of the quantity to that result; an invalid
    /// unit has null dimensions, dereferenced natively, so it is a `Standard_NullObject`
    /// error; `api.any_from_ls(1.0, b"m")?`.
    pub fn any_from_ls(&mut self, data: f64, unit: &[u8]) -> Result<f64, Failure> {
        self.check_loading(self.local_system)?;
        let mut dimensions = Handle::default();
        let value = self.units.from_si_dimensions(data, unit, &mut dimensions)?;
        let Some(quantity) = required(&dimensions)?.quantity(&self.units)? else {
            return Ok(value);
        };
        self.local_system_units.convert_user_system_value_to_si(
            &self.units,
            quantity.as_bytes(),
            value,
        )
    }
    /// SI to `unit` (0 for an invalid unit); `api.any_from_si(273.15, b"deg.C")?`.
    pub fn any_from_si(&mut self, data: f64, unit: &[u8]) -> Result<f64, Failure> {
        self.check_loading(SystemUnits::Default)?;
        self.units.from_si(data, unit)
    }
    /// `first` to `second` (value kept for unknown units or other dimensions, OCC33176);
    /// `api.any_to_any(1.0, b"m", b"mm")?`.
    pub fn any_to_any(&mut self, data: f64, first: &[u8], second: &[u8]) -> Result<f64, Failure> {
        self.check_loading(SystemUnits::Default)?;
        self.units.convert(data, first, second)
    }
}
