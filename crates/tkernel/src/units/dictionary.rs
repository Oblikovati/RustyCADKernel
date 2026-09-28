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

//! `Units_UnitsDictionary`: every quantity and unit of the compiled `Units.dat`.
use super::dimensions::Dimensions;
use super::explorer::Explorer;
use super::quantity::Quantity;
use super::support::{c_str, item, length, object, write_bytes};
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle, StandardTransient};
use crate::tcollection::AsciiString;
use std::io::Write;
use std::sync::{PoisonError, RwLock};

mod creates;

/// The dictionary of units; `units.dictionary_of_units(false)?` returns the shared one.
pub struct UnitsDictionary {
    base: StandardTransient,
    quantities: RwLock<Handle<HSequence<Handle<Quantity>>>>,
}
crate::impl_transient!(
    UnitsDictionary,
    "Units_UnitsDictionary",
    StandardTransient,
    base
);

impl UnitsDictionary {
    /// An uncreated dictionary whose sequence is still null; `UnitsDictionary::new()`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: StandardTransient::default(),
            quantities: RwLock::new(Handle::default()),
        }
    }
    /// The shared quantity sequence (null before `creates`); `dictionary.sequence()`.
    pub fn sequence(&self) -> Handle<HSequence<Handle<Quantity>>> {
        self.quantities
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    fn set_sequence(&self, quantities: Handle<HSequence<Handle<Quantity>>>) {
        *self
            .quantities
            .write()
            .unwrap_or_else(PoisonError::into_inner) = quantities;
    }
    /// First symbol of the first unit of the named quantity, empty when it has no
    /// unit; an unknown quantity prints the native French warning and returns empty;
    /// `dictionary.active_unit(b"LENGTH", &mut std::io::stdout())?`.
    pub fn active_unit(
        &self,
        quantity: &[u8],
        out: &mut dyn Write,
    ) -> Result<AsciiString, Failure> {
        let quantities = self.sequence();
        for index in 1..=length(&quantities)? {
            let current = item(&quantities, index)?;
            let current = object(&current, "Units_UnitsDictionary::ActiveUnit")?;
            if !current.is_equal(quantity) {
                continue;
            }
            let units = current.sequence();
            if length(&units)? == 0 {
                return Ok(AsciiString::default());
            }
            let first = item(&units, 1)?;
            return item(
                &object(&first, "Units_UnitsDictionary::ActiveUnit")?.symbols_sequence(),
                1,
            );
        }
        let warning = [
            b" La grandeur physique ",
            c_str(quantity),
            b" n'existe pas.\n",
        ]
        .concat();
        write_bytes(out, &warning)?;
        Ok(AsciiString::default())
    }
    /// Level 2 dumps every quantity with its units; any other level lists quantities
    /// and unit symbols through an explorer; `dictionary.dump(1, &mut out)?`.
    pub fn dump(&self, level: i32, out: &mut dyn Write) -> Result<(), Failure> {
        let quantities = self.sequence();
        if level != 2 {
            return dump_explored(Explorer::from_sequences(&quantities)?, out);
        }
        if quantities.is_null() {
            return Ok(());
        }
        for index in 1..=length(&quantities)? {
            object(&item(&quantities, index)?, "Units_UnitsDictionary::Dump")?.dump(0, 1, out)?;
        }
        Ok(())
    }
    /// Dumps (level 1) every quantity having exactly these dimensions;
    /// `dictionary.dump_dimensions(&Dimensions::a_length(), &mut out)?`.
    pub fn dump_dimensions(
        &self,
        dimensions: &Handle<Dimensions>,
        out: &mut dyn Write,
    ) -> Result<(), Failure> {
        let quantities = self.sequence();
        for index in 1..=length(&quantities)? {
            let quantity = item(&quantities, index)?;
            let quantity = object(&quantity, "Units_UnitsDictionary::Dump")?;
            if object(&quantity.dimensions(), "Units_UnitsDictionary::Dump")?
                .is_equal(dimensions)?
            {
                quantity.dump(0, 1, out)?;
            }
        }
        Ok(())
    }
}

/// The `Units_Explorer` listing of `Dump(1)`: quantity names, then indented symbols.
fn dump_explored(mut explorer: Explorer, out: &mut dyn Write) -> Result<(), Failure> {
    write_bytes(out, b" UNITS DICTIONARY : \n")?;
    while explorer.more_quantity()? {
        write_bytes(out, &[explorer.quantity()?.as_bytes(), b"\n"].concat())?;
        while explorer.more_unit()? {
            write_bytes(out, &[b"  ", explorer.unit()?.as_bytes(), b"\n"].concat())?;
            explorer.next_unit();
        }
        explorer.next_quantity()?;
    }
    Ok(())
}
