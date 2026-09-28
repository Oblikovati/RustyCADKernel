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

//! `Units_UnitsSystem`: user-chosen units per quantity, one of them active.
use super::context::Units;
use super::explorer::Explorer;
use super::quantity::Quantity;
use super::support::{c_str, item, length, new_sequence, object, write_bytes};
use super::unit::Unit;
use crate::ncollection::HSequence;
use crate::resource::{ResourceCatalog, ResourceEnvironment, ResourceIo};
use crate::standard::{Failure, Handle, StandardTransient};
use crate::tcollection::AsciiString;
use std::io::Write;
use std::rc::Rc;

mod conversion;
mod editing;

/// A units system; copies share the quantity and active-unit sequences, as a
/// native value copy of the class shares its handles; `UnitsSystem::new()`.
pub struct UnitsSystem {
    base: StandardTransient,
    quantities: Handle<HSequence<Handle<Quantity>>>,
    active: Handle<HSequence<i32>>,
}
crate::impl_transient!(UnitsSystem, "Units_UnitsSystem", StandardTransient, base);

impl Clone for UnitsSystem {
    fn clone(&self) -> Self {
        Self {
            base: StandardTransient::default(),
            quantities: self.quantities.clone(),
            active: self.active.clone(),
        }
    }
}

impl Default for UnitsSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// `Units_NoSuchType(aquantity)`.
pub(crate) fn no_such_type(quantity: &[u8]) -> Failure {
    super::no_such_type(&String::from_utf8_lossy(c_str(quantity)))
}

impl UnitsSystem {
    /// An empty system; `UnitsSystem::new()`.
    pub fn new() -> Self {
        Self {
            base: StandardTransient::default(),
            quantities: new_sequence(),
            active: new_sequence(),
        }
    }
    /// The named constructor: it loads the resource file `name` through the injected
    /// storage and environment and then drops it, as natively (the loaded resources
    /// are never read). `verbose` has no counterpart in the catalog API;
    /// `UnitsSystem::with_resource("Units", false, io, environment)`.
    pub fn with_resource(
        name: &str,
        _verbose: bool,
        io: Rc<dyn ResourceIo>,
        environment: Rc<dyn ResourceEnvironment>,
    ) -> Self {
        drop(ResourceCatalog::new(name, io, environment));
        Self::new()
    }
    /// The shared quantity sequence; `system.quantities_sequence()`.
    pub fn quantities_sequence(&self) -> Handle<HSequence<Handle<Quantity>>> {
        self.quantities.clone()
    }
    /// The shared active-unit indices (0 for none); `system.active_units_sequence()`.
    pub fn active_units_sequence(&self) -> Handle<HSequence<i32>> {
        self.active.clone()
    }
    /// True when no quantity is specified; `system.is_empty()?`.
    pub fn is_empty(&self) -> Result<bool, Failure> {
        Ok(length(&self.quantities)? == 0)
    }
    /// Index and object of the quantity named `quantity`, if specified.
    fn find(&self, quantity: &[u8]) -> Result<Option<(usize, Handle<Quantity>)>, Failure> {
        for index in 1..=length(&self.quantities)? {
            let candidate = item(&self.quantities, index)?;
            if object(&candidate, "Units_UnitsSystem")?.is_equal(quantity) {
                return Ok(Some((index, candidate)));
            }
        }
        Ok(None)
    }
    /// First symbol of the active unit, empty without one; an unspecified quantity
    /// raises `Units_NoSuchType`; `system.active_unit(b"LENGTH")?`.
    pub fn active_unit(&self, quantity: &[u8]) -> Result<AsciiString, Failure> {
        let Some((index, found)) = self.find(quantity)? else {
            return Err(no_such_type(quantity));
        };
        let active = item(&self.active, index)?;
        if active == 0 {
            return Ok(AsciiString::default());
        }
        let units = object(&found, "Units_UnitsSystem::ActiveUnit")?.sequence();
        let unit: Handle<Unit> = item(&units, active as usize)?;
        item(
            &object(&unit, "Units_UnitsSystem::ActiveUnit")?.symbols_sequence(),
            1,
        )
    }
    /// Lists quantities and unit symbols; `system.dump(&mut std::io::stdout())?`.
    pub fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        let mut explorer = Explorer::for_system(self)?;
        write_bytes(out, b" UNITSSYSTEM : \n")?;
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
    /// Looks the quantity up in the dictionary (`Units::Quantity`), raising
    /// `Units_NoSuchType` when it is unknown there too.
    fn known_quantity(units: &Units, quantity: &[u8]) -> Result<(), Failure> {
        if units.quantity(quantity)?.is_null() {
            return Err(no_such_type(quantity));
        }
        Ok(())
    }
}
