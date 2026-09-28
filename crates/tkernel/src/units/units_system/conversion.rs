// Created on: 1993-10-22
// Created by: Gilles DEBARBOUILLE
// Copyright (c) 1993-1999 Matra Datavision
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

//! Value conversions between SI and the active units of a `Units_UnitsSystem`.
use super::UnitsSystem;
use crate::standard::{Failure, Handle};
use crate::units::context::Units;
use crate::units::support::{item, object};
use crate::units::unit::{ShiftedUnit, Unit};
use crate::units::unit_sentence::UnitSentence;

/// The active unit's factor and move (move 0 for a plain unit).
fn factor_and_move(unit: &Handle<Unit>) -> Result<(f64, f64), Failure> {
    if let Some(shifted) = unit.cast::<ShiftedUnit>().get() {
        return Ok((shifted.unit().value(), shifted.movement()));
    }
    Ok((object(unit, "Units_UnitsSystem")?.value(), 0.0))
}

impl UnitsSystem {
    /// Converts `value` expressed in `unit` into this system's unit of `quantity`; an
    /// invalid unit prints the native message and returns 0;
    /// `system.convert_value_to_user_system(&units, b"LENGTH", 1.0, b"m")?`.
    pub fn convert_value_to_user_system(
        &self,
        units: &Units,
        quantity: &[u8],
        value: f64,
        unit: &[u8],
    ) -> Result<f64, Failure> {
        let mut sentence = UnitSentence::new(units, unit)?;
        if !sentence.is_done()? {
            units.print(
                b"Units_UnitsSystem::ConvertValueToUserSystem : incorrect unit => return 0\n",
            );
            return Ok(0.0);
        }
        let token = sentence.evaluate()?;
        let factor = object(&token, "Units_UnitsSystem::ConvertValueToUserSystem")?.value();
        self.convert_si_value_to_user_system(units, quantity, value * factor)
    }
    /// SI to the active unit: `value / factor - move`; unchanged without an active
    /// unit or for a quantity the system lacks but the dictionary knows;
    /// `system.convert_si_value_to_user_system(&units, b"LENGTH", 1.0)?`.
    pub fn convert_si_value_to_user_system(
        &self,
        units: &Units,
        quantity: &[u8],
        value: f64,
    ) -> Result<f64, Failure> {
        let Some(unit) = self.active(units, quantity)? else {
            return Ok(value);
        };
        let (factor, movement) = factor_and_move(&unit)?;
        Ok(value / factor - movement)
    }
    /// Active unit to SI. For a shifted unit the native formula is
    /// `value * (factor + move)`, not `(value + move) * factor`; the quirk is kept;
    /// `system.convert_user_system_value_to_si(&units, b"LENGTH", 1.0)?`.
    pub fn convert_user_system_value_to_si(
        &self,
        units: &Units,
        quantity: &[u8],
        value: f64,
    ) -> Result<f64, Failure> {
        let Some(unit) = self.active(units, quantity)? else {
            return Ok(value);
        };
        if unit.cast::<ShiftedUnit>().is_some() {
            let (factor, movement) = factor_and_move(&unit)?;
            return Ok(value * (factor + movement));
        }
        Ok(value * object(&unit, "Units_UnitsSystem")?.value())
    }
    /// The active unit of a specified quantity (`None` when index 0); an unspecified
    /// quantity must at least exist in the dictionary.
    fn active(&self, units: &Units, quantity: &[u8]) -> Result<Option<Handle<Unit>>, Failure> {
        let Some((index, found)) = self.find(quantity)? else {
            Self::known_quantity(units, quantity)?;
            return Ok(None);
        };
        let active = item(&self.active, index)?;
        if active == 0 {
            return Ok(None);
        }
        let sequence = object(&found, "Units_UnitsSystem")?.sequence();
        Ok(Some(item(&sequence, active as usize)?))
    }
}
