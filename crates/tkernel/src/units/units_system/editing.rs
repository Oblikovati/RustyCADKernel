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

//! Specifying, removing and activating units of a `Units_UnitsSystem`.
use super::{UnitsSystem, no_such_type};
use crate::standard::{Failure, Handle};
use crate::units::context::Units;
use crate::units::quantity::Quantity;
use crate::units::support::{
    CHANGE_VALUE, REMOVE_ONE, c_str, edit, item, length, new_sequence, object,
};
use crate::units::token::ShiftedToken;
use crate::units::unit::{ShiftedUnit, Unit};
use crate::units::unit_sentence::UnitSentence;

/// `Units_NoSuchUnit(aunit)`.
fn no_such_unit(unit: &[u8]) -> Failure {
    crate::units::no_such_unit(&String::from_utf8_lossy(c_str(unit)))
}

impl UnitsSystem {
    /// Adds the unit `unit` (named and spelled by the expression itself) to
    /// `quantity`, creating the quantity from the dictionary when needed with no active
    /// unit. Invalid units and unknown quantities print the native warnings on the
    /// context console; `system.specify(&units, b"LENGTH", b"mm")?`.
    pub fn specify(&self, units: &Units, quantity: &[u8], unit: &[u8]) -> Result<(), Failure> {
        let mut sentence = UnitSentence::new(units, unit)?;
        if !sentence.is_done()? {
            units.print(b"Units_UnitsSystem::Specify : incorrect unit\n");
            return Ok(());
        }
        let created = unit_from(&sentence.evaluate()?, unit)?;
        if let Some((_, found)) = self.find(quantity)? {
            object(&created, "Units_UnitsSystem::Specify")?.set_quantity(&found);
            object(&found, "Units_UnitsSystem::Specify")?
                .sequence()
                .get()
                .unwrap()
                .append(created);
            return Ok(());
        }
        let known = units.quantity(quantity)?;
        let Some(known) = known.get() else {
            let warning = [
                b"Warning: in Units_UnitsSystem : Units_NoSuchType '",
                c_str(quantity),
                b"'\n",
            ];
            units.print(&warning.concat());
            return Ok(());
        };
        let copy = Handle::new(Quantity::new(
            known.name().as_bytes(),
            &known.dimensions(),
            &new_sequence(),
        ));
        object(&created, "Units_UnitsSystem::Specify")?.set_quantity(&copy);
        self.quantities.get().unwrap().append(copy.clone());
        self.active.get().unwrap().append(0);
        copy.get()
            .unwrap()
            .sequence()
            .get()
            .unwrap()
            .append(created);
        Ok(())
    }
    /// Removes the unit from the quantity, fixing the active index; removing the last
    /// unit also removes the quantity and then raises `Units_NoSuchUnit` (the native
    /// loop has no return on that path); `system.remove(b"LENGTH", b"mm")?`.
    pub fn remove(&self, quantity: &[u8], unit: &[u8]) -> Result<(), Failure> {
        let Some((index, found)) = self.find(quantity)? else {
            return Err(no_such_type(quantity));
        };
        let units = object(&found, "Units_UnitsSystem::Remove")?.sequence();
        let mut position = 1;
        while position <= length(&units)? {
            if object(&item(&units, position)?, "Units_UnitsSystem::Remove")?.is_equal(unit)? {
                edit(&units, REMOVE_ONE, |items| items.remove(position))?;
                if length(&units)? != 0 {
                    return self.shift_active(index, position);
                }
                edit(&self.quantities, REMOVE_ONE, |items| items.remove(index))?;
                edit(&self.active, REMOVE_ONE, |items| items.remove(index))?;
            }
            position += 1;
        }
        Err(no_such_unit(unit))
    }
    fn shift_active(&self, index: usize, removed: usize) -> Result<(), Failure> {
        let active = item(&self.active, index)?;
        let updated = if active == removed as i32 {
            0
        } else if active > removed as i32 {
            active - 1
        } else {
            return Ok(());
        };
        edit(&self.active, CHANGE_VALUE, |items| {
            items.set_value(index, updated)
        })
    }
    /// Makes the unit active. The native search bound is the number of quantities,
    /// not of units, so it can raise `Standard_OutOfRange` or miss a unit;
    /// `system.activate(b"LENGTH", b"mm")?`.
    pub fn activate(&self, quantity: &[u8], unit: &[u8]) -> Result<(), Failure> {
        let Some((index, found)) = self.find(quantity)? else {
            return Err(no_such_type(quantity));
        };
        let units = object(&found, "Units_UnitsSystem::Activate")?.sequence();
        for position in 1..=length(&self.quantities)? {
            if object(&item(&units, position)?, "Units_UnitsSystem::Activate")?.is_equal(unit)? {
                let position = position as i32;
                return edit(&self.active, CHANGE_VALUE, |items| {
                    items.set_value(index, position)
                });
            }
        }
        Err(no_such_unit(unit))
    }
    /// Activates the first unit of every quantity that has one; `system.activates()?`.
    pub fn activates(&self) -> Result<(), Failure> {
        for index in 1..=length(&self.quantities)? {
            let units = object(
                &item(&self.quantities, index)?,
                "Units_UnitsSystem::Activates",
            )?
            .sequence();
            if length(&units)? > 0 {
                edit(&self.active, CHANGE_VALUE, |items| {
                    items.set_value(index, 1)
                })?;
            }
        }
        Ok(())
    }
}

/// The unit built by `Specify` from the evaluated token: shifted tokens give a shifted
/// unit with the token's value and move.
fn unit_from(
    token: &Handle<crate::units::Token>,
    spelling: &[u8],
) -> Result<Handle<Unit>, Failure> {
    let evaluated = object(token, "Units_UnitsSystem::Specify")?;
    if let Some(shifted) = token.cast::<ShiftedToken>().get() {
        let created = ShiftedUnit::with_symbol(spelling, spelling);
        created.unit().set_value(evaluated.value());
        created.set_movement(shifted.movement());
        return Ok(Handle::new(created).cast());
    }
    let created = Unit::with_symbol(spelling, spelling);
    created.set_value(evaluated.value());
    Ok(Handle::new(created))
}
