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

//! OCCT `UnitsAPI`: conversions between any unit, SI, the current units (a resource
//! file per system) and the local system (SI or the millimetre-based MDTV).
//!
//! The native statics (current/SI/MDTV resource managers, local units systems,
//! selected systems) are fields of an injected [`UnitsApi`] context; the resource
//! files are read through the injected `ResourceIo`/`ResourceEnvironment`.
use crate::resource::{ResourceHost, ResourceManager};
use crate::standard::{Failure, Handle};
use crate::tcollection::AsciiString;
use crate::units::{Dimensions, Units, UnitsSystem};
use std::{cell::RefCell, rc::Rc, sync::Arc};

mod conversion;
mod current;
mod mdtv;

/// `UnitsAPI_SystemUnits`: the selectable local systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemUnits {
    /// Keep the current system (SI when none is loaded yet).
    Default,
    /// The international system.
    Si,
    /// The millimetre-based MDTV system.
    Mdtv,
}

type Catalog = Rc<RefCell<ResourceManager>>;

/// An isolated `UnitsAPI` context; `UnitsApi::new(Arc::new(Units::new()), io, environment)`.
pub struct UnitsApi {
    units: Arc<Units>,
    host: ResourceHost,
    current_units: Option<Catalog>,
    si_current_units: Option<Catalog>,
    mdtv_current_units: Option<Catalog>,
    local_system_units: UnitsSystem,
    si_local_system_units: UnitsSystem,
    mdtv_local_system_units: UnitsSystem,
    local_system: SystemUnits,
    current_system: SystemUnits,
}

impl UnitsApi {
    /// A context in its native initial state (local SI, nothing loaded) over the
    /// given units context and resource host;
    /// `UnitsApi::new(Arc::new(Units::new()), ResourceHost::native())`.
    pub fn new(units: Arc<Units>, host: ResourceHost) -> Self {
        Self {
            units,
            host,
            current_units: None,
            si_current_units: None,
            mdtv_current_units: None,
            local_system_units: UnitsSystem::new(),
            si_local_system_units: UnitsSystem::new(),
            mdtv_local_system_units: UnitsSystem::new(),
            local_system: SystemUnits::Si,
            current_system: SystemUnits::Default,
        }
    }
    /// The units context used for every conversion; `api.units()`.
    pub fn units(&self) -> &Arc<Units> {
        &self.units
    }
    fn catalog(&self, name: &str) -> Result<Catalog, Failure> {
        let manager = ResourceManager::new(name, false, self.host.clone())?;
        Ok(Rc::new(RefCell::new(manager)))
    }
    /// `CheckLoading`: switches the current resources and local units system,
    /// loading them on first use (the MDTV system is specified once).
    fn check_loading(&mut self, system: SystemUnits) -> Result<(), Failure> {
        if self.current_system == system && self.current_units.is_some() {
            return Ok(());
        }
        match system {
            SystemUnits::Default if self.current_units.is_some() => Ok(()),
            SystemUnits::Default | SystemUnits::Si => self.load_si(),
            SystemUnits::Mdtv => self.load_mdtv(),
        }
    }
    fn load_si(&mut self) -> Result<(), Failure> {
        self.current_system = SystemUnits::Si;
        if self.si_current_units.is_none() {
            self.si_current_units = Some(self.catalog("CurrentUnits")?);
        }
        self.current_units = self.si_current_units.clone();
        self.local_system_units = self.si_local_system_units.clone();
        Ok(())
    }
    fn load_mdtv(&mut self) -> Result<(), Failure> {
        self.current_system = SystemUnits::Mdtv;
        if self.mdtv_current_units.is_none() {
            self.mdtv_current_units = Some(self.catalog("MDTVCurrentUnits")?);
        }
        self.current_units = self.mdtv_current_units.clone();
        if self.mdtv_local_system_units.is_empty()? {
            for &(quantity, unit) in mdtv::MDTV_UNITS {
                self.mdtv_local_system_units
                    .specify(&self.units, quantity, unit)?;
            }
            self.mdtv_local_system_units.activates()?;
        }
        self.local_system_units = self.mdtv_local_system_units.clone();
        Ok(())
    }
    /// Selects the local system (`Default` keeps the loaded one); `api.set_local_system(SystemUnits::Mdtv)?`.
    pub fn set_local_system(&mut self, system: SystemUnits) -> Result<(), Failure> {
        self.check_loading(system)?;
        self.local_system = self.current_system;
        Ok(())
    }
    /// The selected local system; `api.local_system()`.
    pub fn local_system(&self) -> SystemUnits {
        self.local_system
    }
    /// Forgets the loaded system and reloads the local one; `api.reload()?`.
    pub fn reload(&mut self) -> Result<(), Failure> {
        self.current_system = SystemUnits::Default;
        self.check_loading(self.local_system)
    }
    /// Dimensions of a dictionary quantity (`Units::Dimensions`);
    /// `api.dimensions(Some(b"LENGTH"))?`.
    pub fn dimensions(&self, quantity: Option<&[u8]>) -> Result<Handle<Dimensions>, Failure> {
        self.units.dimensions(quantity)
    }
    /// `DimensionLess`; `UnitsApi::dimension_less()`.
    pub fn dimension_less() -> Handle<Dimensions> {
        Dimensions::a_less()
    }
    /// `DimensionMass`; `UnitsApi::dimension_mass()`.
    pub fn dimension_mass() -> Handle<Dimensions> {
        Dimensions::a_mass()
    }
    /// `DimensionLength`; `UnitsApi::dimension_length()`.
    pub fn dimension_length() -> Handle<Dimensions> {
        Dimensions::a_length()
    }
    /// `DimensionTime`; `UnitsApi::dimension_time()`.
    pub fn dimension_time() -> Handle<Dimensions> {
        Dimensions::a_time()
    }
    /// `DimensionElectricCurrent`; `UnitsApi::dimension_electric_current()`.
    pub fn dimension_electric_current() -> Handle<Dimensions> {
        Dimensions::a_electric_current()
    }
    /// `DimensionThermodynamicTemperature`; `UnitsApi::dimension_thermodynamic_temperature()`.
    pub fn dimension_thermodynamic_temperature() -> Handle<Dimensions> {
        Dimensions::a_thermodynamic_temperature()
    }
    /// `DimensionAmountOfSubstance`; `UnitsApi::dimension_amount_of_substance()`.
    pub fn dimension_amount_of_substance() -> Handle<Dimensions> {
        Dimensions::a_amount_of_substance()
    }
    /// `DimensionLuminousIntensity`; `UnitsApi::dimension_luminous_intensity()`.
    pub fn dimension_luminous_intensity() -> Handle<Dimensions> {
        Dimensions::a_luminous_intensity()
    }
    /// `DimensionPlaneAngle`; `UnitsApi::dimension_plane_angle()`.
    pub fn dimension_plane_angle() -> Handle<Dimensions> {
        Dimensions::a_plane_angle()
    }
    /// `DimensionSolidAngle`; `UnitsApi::dimension_solid_angle()`.
    pub fn dimension_solid_angle() -> Handle<Dimensions> {
        Dimensions::a_solid_angle()
    }
}

/// A quantity name as a resource key (resource keys are ASCII natively).
fn key(quantity: &[u8]) -> String {
    String::from_utf8_lossy(crate::units_api::c_name(quantity)).into_owned()
}

/// The bytes of a C string argument.
fn c_name(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..end]
}

/// The current unit of `quantity` when the current resources define one.
fn current_value(catalog: &Catalog, quantity: &[u8]) -> Option<AsciiString> {
    let catalog = catalog.borrow();
    let name = key(quantity);
    if !catalog.find(&name) {
        return None;
    }
    catalog.value(&name).ok().cloned()
}
