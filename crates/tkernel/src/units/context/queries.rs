// Created on: 1992-06-22
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

//! Dictionary look-ups of the `Units` package and `Units_Dimensions::Quantity`.
use super::Units;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use crate::tcollection::AsciiString;
use crate::units::dimensions::Dimensions;
use crate::units::quantity::Quantity;
use crate::units::support::{c_str, item, length, object};

type Quantities = Handle<HSequence<Handle<Quantity>>>;

impl Units {
    fn quantities(&self) -> Result<Quantities, Failure> {
        let dictionary = self.dictionary_of_units(false)?;
        Ok(object(&dictionary, "Units::DictionaryOfUnits")?.sequence())
    }
    /// The dictionary quantity named `name`, or a null handle; `units.quantity(b"LENGTH")?`.
    pub fn quantity(&self, name: &[u8]) -> Result<Handle<Quantity>, Failure> {
        let quantities = self.quantities()?;
        for index in 1..=length(&quantities)? {
            let quantity = item(&quantities, index)?;
            if object(&quantity, "Units::Quantity")?.name().as_bytes() == c_str(name) {
                return Ok(quantity);
            }
        }
        Ok(Handle::default())
    }
    /// Name of the first quantity having a unit symbol `unit`. The last answer is
    /// cached by symbol; the cache starts as the empty symbol mapped to an empty name,
    /// so `first_quantity(b"")` answers `Some("")` like the native static strings;
    /// `units.first_quantity(b"mm")?`.
    pub fn first_quantity(&self, unit: &[u8]) -> Result<Option<AsciiString>, Failure> {
        let symbol = AsciiString::from_bytes(unit);
        {
            let caches = self.caches();
            if caches.first_symbol.as_bytes() == symbol.as_bytes() {
                return Ok(Some(caches.first_quantity.clone()));
            }
        }
        let Some(name) = self.search_symbol(symbol.as_bytes())? else {
            return Ok(None);
        };
        let mut caches = self.caches();
        caches.first_symbol = symbol;
        caches.first_quantity = name.clone();
        Ok(Some(name))
    }
    fn search_symbol(&self, symbol: &[u8]) -> Result<Option<AsciiString>, Failure> {
        let quantities = self.quantities()?;
        for index in 1..=length(&quantities)? {
            let quantity = item(&quantities, index)?;
            let quantity = object(&quantity, "Units::FirstQuantity")?;
            let units = quantity.sequence();
            for position in 1..=length(&units)? {
                let symbols =
                    object(&item(&units, position)?, "Units::FirstQuantity")?.symbols_sequence();
                for rank in 1..=length(&symbols)? {
                    if item(&symbols, rank)?.as_bytes() == symbol {
                        return Ok(Some(quantity.name()));
                    }
                }
            }
        }
        Ok(None)
    }
    /// Dimensions of the named dictionary quantity (`Standard_NoSuchObject` when
    /// unknown); `None` (a null C string) gives the shared dimensionless object;
    /// `units.dimensions(Some(b"LENGTH"))?`.
    pub fn dimensions(&self, quantity: Option<&[u8]>) -> Result<Handle<Dimensions>, Failure> {
        let Some(name) = quantity else {
            return Ok(Dimensions::a_less());
        };
        let quantities = self.quantities()?;
        for index in 1..=length(&quantities)? {
            let candidate = item(&quantities, index)?;
            let candidate = object(&candidate, "Units::Dimensions")?;
            if candidate.is_equal(name) {
                return Ok(candidate.dimensions());
            }
        }
        Err(Failure::with_type(
            "Standard_NoSuchObject",
            "Units::Dimensions",
            "",
        ))
    }
    /// The quantity sequence `Units_Dimensions::Quantity` captured on its first call.
    fn dimension_quantities(&self) -> Result<Quantities, Failure> {
        let cached = self.caches().dimension_quantities.clone();
        if cached.is_some() {
            return Ok(cached);
        }
        let quantities = self.quantities()?;
        self.caches().dimension_quantities = quantities.clone();
        Ok(quantities)
    }
}

impl Dimensions {
    /// Name of the first quantity with exactly these dimensions, searched in the
    /// quantity sequence of the context's dictionary as it was on the first call (the
    /// native static is never refreshed); `dims.quantity(&units)?`.
    pub fn quantity(&self, units: &Units) -> Result<Option<AsciiString>, Failure> {
        let quantities = units.dimension_quantities()?;
        for index in 1..=length(&quantities)? {
            let quantity = item(&quantities, index)?;
            let quantity = object(&quantity, "Units_Dimensions::Quantity")?;
            let dimensions = quantity.dimensions();
            if object(&dimensions, "Units_Dimensions::Quantity")?.exponents() == self.exponents() {
                return Ok(Some(quantity.name()));
            }
        }
        Ok(None)
    }
}
