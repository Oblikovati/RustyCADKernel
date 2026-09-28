// Created on: 1991-01-21
// Created by: Christophe MARION
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of the composition members of FoundationClasses/TKMath/TopLoc/TopLoc_Location.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Composition keeps the native item order and powers, so every cached transformation is
//! built by the same `gp_Trsf::Powered` and `PreMultiply` calls as in OCCT.
use super::{ItemLocation, Location};
use rustycad_tkernel::standard::Failure;
impl Location {
    /// Reverses datum order and negates exponents; a singular datum fails; `location.inverted()?`.
    pub fn inverted(&self) -> Result<Self, Failure> {
        let mut result = Self::default();
        let mut cursor = self.items();
        while let Ok(item) = cursor.value() {
            let inverse = ItemLocation::new(item.datum.clone(), -item.power)?;
            result.items_mut().construct(inverse);
            cursor = cursor.tail();
        }
        Ok(result)
    }
    /// Returns `self * other`, cancelling adjacent powers of one datum; `left.multiplied(&right)?`.
    pub fn multiplied(&self, other: &Self) -> Result<Self, Failure> {
        if self.is_identity() {
            return Ok(other.clone());
        }
        if other.is_identity() {
            return Ok(self.clone());
        }
        // Native recursion prepends the items of `other` from its last to its first; the loop
        // below performs the same prepends in the same order without deep recursion.
        let mut pending = Vec::new();
        let mut cursor = other.items();
        while let Ok(item) = cursor.value() {
            pending.push(item);
            cursor = cursor.tail();
        }
        let mut result = self.clone();
        for item in pending.into_iter().rev() {
            result.prepend_cancelling(item)?;
        }
        Ok(result)
    }
    fn prepend_cancelling(&mut self, item: &ItemLocation) -> Result<(), Failure> {
        let mut power = item.power;
        if let Ok(head) = self.items().value()
            && head.datum == item.datum
        {
            // Signed overflow is undefined natively; the pinned build wraps.
            power = power.wrapping_add(head.power);
            self.items_mut().to_tail();
        }
        if power != 0 {
            let combined = ItemLocation::new(item.datum.clone(), power)?;
            self.items_mut().construct(combined);
        }
        Ok(())
    }
    /// Returns `self * other.inverted()`; `left.divided(&right)?`.
    pub fn divided(&self, other: &Self) -> Result<Self, Failure> {
        self.multiplied(&other.inverted()?)
    }
    /// Returns `other.inverted() * self`, with the identity shortcuts; `left.predivided(&right)?`.
    pub fn predivided(&self, other: &Self) -> Result<Self, Failure> {
        if other.is_identity() {
            return Ok(self.clone());
        }
        if self.is_identity() {
            return other.inverted();
        }
        if self == other {
            return Ok(Self::default());
        }
        other.inverted()?.multiplied(self)
    }
    /// Raises the chain to an integer power; zero is the identity; `location.powered(-2)?`.
    pub fn powered(&self, power: i32) -> Result<Self, Failure> {
        if self.is_identity() || power == 1 {
            return Ok(self.clone());
        }
        if power == 0 {
            return Ok(Self::default());
        }
        let head = self.items().value()?;
        if self.items().tail().is_empty() {
            let mut result = Self::default();
            let item = ItemLocation::new(head.datum.clone(), head.power.wrapping_mul(power))?;
            result.items_mut().construct(item);
            return Ok(result);
        }
        if power > 0 {
            return self.positive_power(power);
        }
        if power == i32::MIN {
            // Natively -INT_MIN overflows and the recursion never ends.
            return Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("TopLoc_Location::Powered: power {power}; expected a negatable exponent"),
                "",
            ));
        }
        self.inverted()?.powered(-power)
    }
    // Powered(n) = Multiplied(Powered(n - 1)), unrolled from Powered(1) = self.
    fn positive_power(&self, power: i32) -> Result<Self, Failure> {
        let mut result = self.clone();
        for _ in 1..power {
            result = self.multiplied(&result)?;
        }
        Ok(result)
    }
}
impl std::ops::Mul for &Location {
    type Output = Result<Location, Failure>;
    fn mul(self, other: Self) -> Self::Output {
        self.multiplied(other)
    }
}
impl std::ops::Div for &Location {
    type Output = Result<Location, Failure>;
    fn div(self, other: Self) -> Self::Output {
        self.divided(other)
    }
}
