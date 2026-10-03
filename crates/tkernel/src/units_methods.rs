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

//! OCCT `UnitsMethods`: IGES length-unit codes, their millimetre factors and names,
//! and the "cascade length unit" setting.
//!
//! The native setting is a process-wide atomic; here it is a [`UnitsMethods`] value
//! with the same atomic storage, injected where the unit is needed.
//! Rust has no C++-style default arguments, so `*_default` methods preserve calls
//! that omit the native `Millimeter` base without creating extra source declarations.
use crate::tcollection::AsciiString;
use std::sync::atomic::{AtomicU64, Ordering};

/// `UnitsMethods_LengthUnit`: IGES length-unit codes (discriminants are the codes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LengthUnit {
    /// Unknown unit, factor 1.
    Undefined = 0,
    /// 25.4 mm.
    Inch = 1,
    /// 1 mm.
    Millimeter = 2,
    /// 304.8 mm.
    Foot = 4,
    /// 1609344 mm.
    Mile = 5,
    /// 1000 mm.
    Meter = 6,
    /// 1000000 mm.
    Kilometer = 7,
    /// 0.001 inch.
    Mil = 8,
    /// 0.001 mm.
    Micron = 9,
    /// 10 mm.
    Centimeter = 10,
    /// 0.000001 inch.
    Microinch = 11,
}

/// The factor-matching order of `GetLengthUnitByFactorValue`.
const BY_FACTOR: [(f64, LengthUnit); 10] = [
    (1.0, LengthUnit::Millimeter),
    (25.4, LengthUnit::Inch),
    (304.8, LengthUnit::Foot),
    (1609344.0, LengthUnit::Mile),
    (1000.0, LengthUnit::Meter),
    (1000000.0, LengthUnit::Kilometer),
    (0.0254, LengthUnit::Mil),
    (0.001, LengthUnit::Micron),
    (10.0, LengthUnit::Centimeter),
    (0.0000254, LengthUnit::Microinch),
];

/// Unit names in `LengthUnitFromString` order.
const NAMES: [(&str, LengthUnit); 10] = [
    ("mm", LengthUnit::Millimeter),
    ("m", LengthUnit::Meter),
    ("cm", LengthUnit::Centimeter),
    ("km", LengthUnit::Kilometer),
    ("micron", LengthUnit::Micron),
    ("in", LengthUnit::Inch),
    ("min", LengthUnit::Mil),
    ("nin", LengthUnit::Microinch),
    ("ft", LengthUnit::Foot),
    ("stat.mile", LengthUnit::Mile),
];

/// The cascade length unit, in millimetres; `UnitsMethods::default()` starts at 1.
pub struct UnitsMethods {
    cascade_length_unit: AtomicU64,
}

impl Default for UnitsMethods {
    fn default() -> Self {
        Self {
            cascade_length_unit: AtomicU64::new(1.0_f64.to_bits()),
        }
    }
}

impl UnitsMethods {
    /// Millimetres per unit of IGES code `unit` (1 for unknown codes);
    /// `UnitsMethods::get_length_factor_value(1) == 25.4`.
    pub fn get_length_factor_value(unit: i32) -> f64 {
        match unit {
            1 => 25.4,
            2 => 1.0,
            4 => 304.8,
            5 => 1609344.0,
            6 => 1000.0,
            7 => 1000000.0,
            8 => 0.0254,
            9 => 0.001,
            10 => 10.0,
            11 => 0.0000254,
            _ => 1.0,
        }
    }
    /// `GetCasCadeLengthUnit()` with its native default base unit, millimetres;
    /// `methods.get_cascade_length_unit_default()`.
    pub fn get_cascade_length_unit_default(&self) -> f64 {
        self.get_cascade_length_unit(LengthUnit::Millimeter)
    }

    /// `SetCasCadeLengthUnit(value)` with its native default base unit, millimetres;
    /// `methods.set_cascade_length_unit_default(25.4)`.
    pub fn set_cascade_length_unit_default(&self, value: f64) {
        self.set_cascade_length_unit(value, LengthUnit::Millimeter);
    }

    /// The cascade unit expressed in `base`; `methods.get_cascade_length_unit(LengthUnit::Meter)`.
    pub fn get_cascade_length_unit(&self, base: LengthUnit) -> f64 {
        let unit = f64::from_bits(self.cascade_length_unit.load(Ordering::SeqCst));
        unit * Self::get_length_unit_scale(LengthUnit::Millimeter, base)
    }
    /// Sets the cascade unit from a value in `base`;
    /// `methods.set_cascade_length_unit(1.0, LengthUnit::Meter)`.
    pub fn set_cascade_length_unit(&self, value: f64, base: LengthUnit) {
        let unit = value * Self::get_length_unit_scale(base, LengthUnit::Millimeter);
        self.cascade_length_unit
            .store(unit.to_bits(), Ordering::SeqCst);
    }
    /// Sets the cascade unit from an IGES code; `methods.set_cascade_length_unit_code(6)`.
    pub fn set_cascade_length_unit_code(&self, unit: i32) {
        let value = Self::get_length_factor_value(unit);
        self.cascade_length_unit
            .store(value.to_bits(), Ordering::SeqCst);
    }
    /// Factor from `from` to `to`; `UnitsMethods::get_length_unit_scale(LengthUnit::Meter, LengthUnit::Millimeter)`.
    pub fn get_length_unit_scale(from: LengthUnit, to: LengthUnit) -> f64 {
        Self::get_length_factor_value(from as i32) / Self::get_length_factor_value(to as i32)
    }
    /// The unit whose millimetre factor is within 1e-6 of `factor` expressed in
    /// `base`, `Undefined` otherwise; `UnitsMethods::get_length_unit_by_factor_value(1.0, LengthUnit::Meter)`.
    pub fn get_length_unit_by_factor_value(factor: f64, base: LengthUnit) -> LengthUnit {
        let value = factor * Self::get_length_unit_scale(base, LengthUnit::Millimeter);
        BY_FACTOR
            .iter()
            .find(|(reference, _)| (reference - value).abs() < 1.0e-6)
            .map_or(LengthUnit::Undefined, |(_, unit)| *unit)
    }

    /// `GetLengthUnitByFactorValue(factor)` with its native default base unit, millimetres;
    /// `UnitsMethods::get_length_unit_by_factor_value_default(25.4)`.
    pub fn get_length_unit_by_factor_value_default(factor: f64) -> LengthUnit {
        Self::get_length_unit_by_factor_value(factor, LengthUnit::Millimeter)
    }

    /// The short name of the unit matching `factor` in `base`;
    /// `UnitsMethods::dump_length_unit_factor(25.4, LengthUnit::Millimeter) == "in"`.
    pub fn dump_length_unit_factor(factor: f64, base: LengthUnit) -> &'static str {
        Self::dump_length_unit(Self::get_length_unit_by_factor_value(factor, base))
    }

    /// `DumpLengthUnit(factor)` with its native default base unit, millimetres;
    /// `UnitsMethods::dump_length_unit_factor_default(25.4) == "in"`.
    pub fn dump_length_unit_factor_default(factor: f64) -> &'static str {
        Self::dump_length_unit_factor(factor, LengthUnit::Millimeter)
    }
    /// The short name of `unit` (`UNDEFINED` for Undefined; mil is `min`, microinch
    /// `nin`); `UnitsMethods::dump_length_unit(LengthUnit::Foot) == "ft"`.
    pub fn dump_length_unit(unit: LengthUnit) -> &'static str {
        NAMES
            .iter()
            .find(|(_, candidate)| *candidate == unit)
            .map_or("UNDEFINED", |(name, _)| name)
    }
    /// Parses a short name (lower-casing first unless case sensitive);
    /// `UnitsMethods::length_unit_from_string(b"MM", false) == LengthUnit::Millimeter`.
    pub fn length_unit_from_string(text: &[u8], case_sensitive: bool) -> LengthUnit {
        let mut name = AsciiString::from_bytes(text);
        if !case_sensitive {
            name.lower_case();
        }
        NAMES
            .iter()
            .find(|(candidate, _)| name.as_bytes() == candidate.as_bytes())
            .map_or(LengthUnit::Undefined, |(_, unit)| *unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factors_names_and_lookup_tables() {
        assert_eq!(UnitsMethods::get_length_factor_value(3), 1.0);
        assert_eq!(UnitsMethods::get_length_factor_value(11), 0.0000254);
        assert_eq!(UnitsMethods::dump_length_unit(LengthUnit::Mil), "min");
        assert_eq!(
            UnitsMethods::dump_length_unit(LengthUnit::Undefined),
            "UNDEFINED"
        );
        assert_eq!(
            UnitsMethods::length_unit_from_string(b"Stat.Mile", false),
            LengthUnit::Mile
        );
        assert_eq!(
            UnitsMethods::length_unit_from_string(b"MM", true),
            LengthUnit::Undefined
        );
        assert_eq!(
            UnitsMethods::dump_length_unit_factor(0.001, LengthUnit::Meter),
            "mm"
        );
        let unit = UnitsMethods::get_length_unit_by_factor_value(2.54, LengthUnit::Centimeter);
        assert_eq!(unit, LengthUnit::Inch);
    }

    #[test]
    fn cascade_length_unit_is_stored_in_millimetres() {
        let methods = UnitsMethods::default();
        assert_eq!(methods.get_cascade_length_unit(LengthUnit::Millimeter), 1.0);
        methods.set_cascade_length_unit(1.0, LengthUnit::Meter);
        assert_eq!(
            methods.get_cascade_length_unit(LengthUnit::Millimeter),
            1000.0
        );
        assert_eq!(methods.get_cascade_length_unit(LengthUnit::Meter), 1.0);
        methods.set_cascade_length_unit_code(1);
        // 25.4 * (1 / 25.4) is not exactly 1, as natively.
        assert_eq!(
            methods.get_cascade_length_unit(LengthUnit::Inch),
            25.4 * (1.0 / 25.4)
        );
    }

    #[test]
    fn default_argument_calls_use_millimetres() {
        let methods = UnitsMethods::default();
        assert_eq!(
            methods.get_cascade_length_unit_default(),
            methods.get_cascade_length_unit(LengthUnit::Millimeter)
        );
        methods.set_cascade_length_unit_default(25.4);
        assert_eq!(methods.get_cascade_length_unit_default(), 25.4);
        assert_eq!(
            UnitsMethods::get_length_unit_by_factor_value_default(25.4),
            LengthUnit::Inch
        );
        assert_eq!(UnitsMethods::dump_length_unit_factor_default(25.4), "in");
    }
}
