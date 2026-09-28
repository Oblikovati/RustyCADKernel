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

//! `Units_Explorer`: walks the quantities and unit symbols of a dictionary or of a
//! units system, reporting which unit is active.
use super::dictionary::UnitsDictionary;
use super::quantity::Quantity;
use super::support::{c_str, item, length, new_sequence, object};
use super::unit::Unit;
use super::units_system::UnitsSystem;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use crate::tcollection::AsciiString;

/// A cursor over quantities and units; `Explorer::for_dictionary(&dictionary)?`.
///
/// Positions are one-based; a quantity lookup that fails leaves position 0, from
/// which `quantity()` raises `Standard_OutOfRange` as natively.
pub struct Explorer {
    current_quantity: usize,
    quantities: Handle<HSequence<Handle<Quantity>>>,
    current_unit: usize,
    units: Handle<HSequence<Handle<Unit>>>,
    active: Handle<HSequence<i32>>,
}

impl Default for Explorer {
    /// `Units_Explorer()`: positions 1 and null sequences.
    fn default() -> Self {
        Self {
            current_quantity: 1,
            current_unit: 1,
            ..Self::blank()
        }
    }
}

impl Explorer {
    /// The value-initialized members of every other native constructor.
    fn blank() -> Self {
        Self {
            current_quantity: 0,
            quantities: Handle::default(),
            current_unit: 0,
            units: Handle::default(),
            active: Handle::default(),
        }
    }
    /// Explores a units system; `Explorer::for_system(&system)?`.
    pub fn for_system(system: &UnitsSystem) -> Result<Self, Failure> {
        let mut explorer = Self::blank();
        explorer.init_system(system)?;
        Ok(explorer)
    }
    /// Explores a dictionary, every first unit counting as active;
    /// `Explorer::for_dictionary(&dictionary)?`.
    pub fn for_dictionary(dictionary: &UnitsDictionary) -> Result<Self, Failure> {
        Self::from_sequences(&dictionary.sequence())
    }
    /// Explores one quantity of a system; `Explorer::for_system_quantity(&s, b"LENGTH")?`.
    pub fn for_system_quantity(system: &UnitsSystem, quantity: &[u8]) -> Result<Self, Failure> {
        let mut explorer = Self::blank();
        explorer.init_system_quantity(system, quantity)?;
        Ok(explorer)
    }
    /// Explores one quantity of a dictionary; `Explorer::for_dictionary_quantity(&d, b"MASS")?`.
    pub fn for_dictionary_quantity(
        dictionary: &UnitsDictionary,
        quantity: &[u8],
    ) -> Result<Self, Failure> {
        let mut explorer = Self::blank();
        explorer.init_dictionary_quantity(dictionary, quantity)?;
        Ok(explorer)
    }
    pub(super) fn from_sequences(
        quantities: &Handle<HSequence<Handle<Quantity>>>,
    ) -> Result<Self, Failure> {
        let mut explorer = Self::blank();
        explorer.init_quantities(quantities)?;
        Ok(explorer)
    }
    /// `Init(unitssystem)`; `explorer.init_system(&system)?`.
    pub fn init_system(&mut self, system: &UnitsSystem) -> Result<(), Failure> {
        self.current_quantity = 1;
        self.quantities = system.quantities_sequence();
        self.active = system.active_units_sequence();
        self.load_units()?;
        self.current_unit = 1;
        Ok(())
    }
    /// `Init(unitsdictionary)`; `explorer.init_dictionary(&dictionary)?`.
    pub fn init_dictionary(&mut self, dictionary: &UnitsDictionary) -> Result<(), Failure> {
        self.init_quantities(&dictionary.sequence())
    }
    fn init_quantities(
        &mut self,
        quantities: &Handle<HSequence<Handle<Quantity>>>,
    ) -> Result<(), Failure> {
        self.current_quantity = 1;
        self.quantities = quantities.clone();
        self.active = all_first(quantities)?;
        self.load_units()?;
        self.current_unit = 1;
        Ok(())
    }
    /// `Init(unitssystem, aquantity)`; `explorer.init_system_quantity(&s, b"LENGTH")?`.
    pub fn init_system_quantity(
        &mut self,
        system: &UnitsSystem,
        quantity: &[u8],
    ) -> Result<(), Failure> {
        self.current_quantity = 0;
        self.quantities = system.quantities_sequence();
        self.active = system.active_units_sequence();
        self.seek(quantity)
    }
    /// `Init(unitsdictionary, aquantity)`; `explorer.init_dictionary_quantity(&d, b"MASS")?`.
    pub fn init_dictionary_quantity(
        &mut self,
        dictionary: &UnitsDictionary,
        quantity: &[u8],
    ) -> Result<(), Failure> {
        self.current_quantity = 0;
        self.quantities = dictionary.sequence();
        self.active = all_first(&self.quantities)?;
        self.seek(quantity)
    }
    /// Positions on the named quantity; an unknown name leaves the positions as they
    /// were (quantity 0) without error, as natively.
    fn seek(&mut self, quantity: &[u8]) -> Result<(), Failure> {
        for index in 1..=length(&self.quantities)? {
            let candidate = item(&self.quantities, index)?;
            let candidate = object(&candidate, "Units_Explorer::Init")?;
            if candidate.name().as_bytes() == c_str(quantity) {
                self.current_quantity = index;
                self.current_unit = 1;
                self.units = candidate.sequence();
                return Ok(());
            }
        }
        Ok(())
    }
    fn load_units(&mut self) -> Result<(), Failure> {
        if self.more_quantity()? {
            let quantity = item(&self.quantities, self.current_quantity)?;
            self.units = object(&quantity, "Units_Explorer")?.sequence();
        }
        Ok(())
    }
    /// True while a quantity remains; `explorer.more_quantity()?`.
    pub fn more_quantity(&self) -> Result<bool, Failure> {
        Ok(self.current_quantity <= length(&self.quantities)?)
    }
    /// Moves to the next quantity and its first unit; `explorer.next_quantity()?`.
    pub fn next_quantity(&mut self) -> Result<(), Failure> {
        self.current_quantity += 1;
        self.current_unit = 1;
        self.load_units()
    }
    /// The current quantity name; `explorer.quantity()?`.
    pub fn quantity(&self) -> Result<AsciiString, Failure> {
        let quantity = item(&self.quantities, self.current_quantity)?;
        Ok(object(&quantity, "Units_Explorer::Quantity")?.name())
    }
    /// True while a unit remains in the current quantity; `explorer.more_unit()?`.
    pub fn more_unit(&self) -> Result<bool, Failure> {
        Ok(self.current_unit <= length(&self.units)?)
    }
    /// Moves to the next unit; `explorer.next_unit()`.
    pub fn next_unit(&mut self) {
        self.current_unit += 1;
    }
    /// First symbol of the current unit; `explorer.unit()?`.
    pub fn unit(&self) -> Result<AsciiString, Failure> {
        let unit = item(&self.units, self.current_unit)?;
        item(
            &object(&unit, "Units_Explorer::Unit")?.symbols_sequence(),
            1,
        )
    }
    /// True when the current unit is the quantity's active one; `explorer.is_active()?`.
    pub fn is_active(&self) -> Result<bool, Failure> {
        let active = item(&self.active, self.current_quantity)?;
        Ok(usize::try_from(active).is_ok_and(|active| active == self.current_unit))
    }
}

/// A new active-unit sequence with 1 for every quantity.
fn all_first(
    quantities: &Handle<HSequence<Handle<Quantity>>>,
) -> Result<Handle<HSequence<i32>>, Failure> {
    let active = new_sequence::<i32>();
    for _ in 0..length(quantities)? {
        active.get().unwrap().append(1);
    }
    Ok(active)
}
