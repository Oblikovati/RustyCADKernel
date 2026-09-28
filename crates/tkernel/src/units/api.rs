// Created on: 1992-06-24
// Created by: Gilles DEBARBOUILLE
// Copyright (c) 1992-1999 Matra Datavision
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

use super::{Failure, UnitToken, UnitsDictionary};
use std::{collections::BTreeMap, sync::Arc};
#[path = "mdtv.rs"]
mod mdtv;
/// Native local-system choices; `UnitSystem::Mdtv` uses the dictionary's MDTV overrides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitSystem {
    /// Keep the selected system, initializing SI when needed.
    Default,
    /// International system.
    Si,
    /// Native CAD millimeter-based quantity overrides.
    Mdtv,
}
/// An isolated conversion context; `UnitsApi::new(UnitsDictionary::builtin()?)`.
pub struct UnitsApi {
    dictionary: Arc<UnitsDictionary>,
    local: UnitSystem,
    overrides: BTreeMap<Vec<u8>, UnitToken>,
}
impl UnitsApi {
    /// Injects the immutable dictionary and starts in SI; `UnitsApi::new(dictionary)`.
    pub fn new(dictionary: Arc<UnitsDictionary>) -> Self {
        Self {
            dictionary,
            local: UnitSystem::Si,
            overrides: BTreeMap::new(),
        }
    }
    /// Borrows the dictionary used by this context; `api.dictionary()`.
    pub fn dictionary(&self) -> &UnitsDictionary {
        &self.dictionary
    }
    /// Selects native local quantity units; `api.set_local_system(UnitSystem::Mdtv)?`.
    pub fn set_local_system(&mut self, system: UnitSystem) -> Result<(), Failure> {
        if system == UnitSystem::Default {
            return Ok(());
        }
        let mut overrides = BTreeMap::new();
        if system == UnitSystem::Mdtv {
            for &(quantity, expression) in mdtv::MDTV {
                if let Some(token) = self.dictionary.evaluate(expression)? {
                    overrides.insert(quantity.to_vec(), token);
                }
            }
        }
        self.local = system;
        self.overrides = overrides;
        Ok(())
    }
    /// Returns the active local system; `api.local_system()==UnitSystem::Si`.
    pub fn local_system(&self) -> UnitSystem {
        self.local
    }
    /// Converts to SI, returning zero for lexically unknown units; `api.any_to_si(1.0,"min")?`.
    pub fn any_to_si(&self, value: f64, unit: impl AsRef<[u8]>) -> Result<f64, Failure> {
        Ok(self
            .dictionary
            .evaluate(unit.as_ref())?
            .map_or(0.0, |token| token.to_si(value)))
    }
    /// Converts from SI including shifted temperatures; `api.any_from_si(273.15,"deg.C")?`.
    pub fn any_from_si(&self, value: f64, unit: impl AsRef<[u8]>) -> Result<f64, Failure> {
        Ok(self
            .dictionary
            .evaluate(unit.as_ref())?
            .map_or(0.0, |token| token.convert_from_si(value)))
    }
    /// Preserves the input for unknown or incompatible dimensions (OCC33176); `api.any_to_any(1.0,"m","mm")?`.
    pub fn any_to_any(
        &self,
        value: f64,
        from: impl AsRef<[u8]>,
        to: impl AsRef<[u8]>,
    ) -> Result<f64, Failure> {
        let Some(source) = self.dictionary.evaluate(from.as_ref())? else {
            return Ok(value);
        };
        let Some(target) = self.dictionary.evaluate(to.as_ref())? else {
            return Ok(value);
        };
        if source.multiply(&target, b"/").dimensions != [0.0; 9] {
            return Ok(value);
        }
        Ok(target.convert_from_si(source.to_si(value)))
    }
    /// Converts to the active local quantity unit; `api.any_to_ls(3.0,"mm")?`.
    pub fn any_to_ls(&self, value: f64, unit: impl AsRef<[u8]>) -> Result<f64, Failure> {
        let Some(token) = self.dictionary.evaluate(unit.as_ref())? else {
            return Ok(0.0);
        };
        let si = token.to_si(value);
        let local = self
            .dictionary
            .quantity(&token.dimensions)
            .and_then(|name| self.overrides.get(name));
        Ok(local.map_or(si, |unit| unit.convert_from_si(si)))
    }
}
