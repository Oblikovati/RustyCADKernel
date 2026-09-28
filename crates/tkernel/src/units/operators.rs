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

//! `Units_Operators.hxx`: the free operators on handles, each forwarding to the
//! member it names natively. A null left handle is `Standard_NullObject`.
use super::dimensions::Dimensions;
use super::quantity::Quantity;
use super::support::object;
use super::token::{Token, TokenHandle};
use super::unit::Unit;
use crate::standard::{Failure, Handle};

/// `operator==(handle<Units_Quantity>, const char*)`; `quantity_equals(&q, b"MASS")?`.
pub fn quantity_equals(quantity: &Handle<Quantity>, text: &[u8]) -> Result<bool, Failure> {
    Ok(object(quantity, "operator==")?.is_equal(text))
}
/// `operator==(handle<Units_Token>, const char*)`; `token_equals(&t, b"m")?`.
pub fn token_equals(token: &Handle<Token>, text: &[u8]) -> Result<bool, Failure> {
    Ok(object(token, "operator==")?.is_equal(text))
}
/// `operator==(handle<Units_Unit>, const char*)`: any symbol matches; `unit_equals(&u, b"m")?`.
pub fn unit_equals(unit: &Handle<Unit>, text: &[u8]) -> Result<bool, Failure> {
    object(unit, "operator==")?.is_equal(text)
}
/// `operator*` on dimensions; `dimensions_multiply(&a, &b)?`.
pub fn dimensions_multiply(
    left: &Handle<Dimensions>,
    right: &Handle<Dimensions>,
) -> Result<Handle<Dimensions>, Failure> {
    object(left, "operator*")?.multiply(right)
}
/// `operator/` on dimensions; `dimensions_divide(&a, &b)?`.
pub fn dimensions_divide(
    left: &Handle<Dimensions>,
    right: &Handle<Dimensions>,
) -> Result<Handle<Dimensions>, Failure> {
    object(left, "operator/")?.divide(right)
}
/// `pow(handle<Units_Dimensions>, double)`; `dimensions_pow(&a, 2.0)?`.
pub fn dimensions_pow(
    dimensions: &Handle<Dimensions>,
    exponent: f64,
) -> Result<Handle<Dimensions>, Failure> {
    Ok(object(dimensions, "pow")?.power(exponent))
}
/// `pow(handle<Units_Token>, handle<Units_Token>)`; `token_pow_token(&a, &b)?`.
pub fn token_pow_token(
    left: &Handle<Token>,
    right: &Handle<Token>,
) -> Result<Handle<Token>, Failure> {
    object(left, "pow")?.power_token(right)
}
/// `pow(handle<Units_Token>, double)`; `token_pow(&a, 2.0)?`.
pub fn token_pow(token: &Handle<Token>, exponent: f64) -> Result<Handle<Token>, Failure> {
    object(token, "pow")?.power(exponent)
}
/// `operator+(handle<Units_Token>, int)`: a blank token; `token_add_integer(&a, 1)?`.
pub fn token_add_integer(token: &Handle<Token>, integer: i32) -> Result<Handle<Token>, Failure> {
    Ok(object(token, "operator+")?.add_integer(integer))
}
/// `operator+` on tokens; `token_add(&a, &b)?`.
pub fn token_add(left: &Handle<Token>, right: &Handle<Token>) -> Result<Handle<Token>, Failure> {
    object(left, "operator+")?.add(right)
}
/// `operator-` on tokens; `token_subtract(&a, &b)?`.
pub fn token_subtract(
    left: &Handle<Token>,
    right: &Handle<Token>,
) -> Result<Handle<Token>, Failure> {
    object(left, "operator-")?.subtract(right)
}
/// `operator*` on tokens; `token_multiply(&a, &b)?`.
pub fn token_multiply(
    left: &Handle<Token>,
    right: &Handle<Token>,
) -> Result<Handle<Token>, Failure> {
    object(left, "operator*")?.multiply(right)
}
/// `operator/` on tokens (the left token itself for a near-zero divisor);
/// `token_divide(&a, &b)?`.
pub fn token_divide(left: &Handle<Token>, right: &Handle<Token>) -> Result<Handle<Token>, Failure> {
    left.divide(right)
}
/// `operator!=(handle<Units_Token>, const char*)`; `token_not_equals(&a, b"m")?`.
pub fn token_not_equals(token: &Handle<Token>, text: &[u8]) -> Result<bool, Failure> {
    Ok(object(token, "operator!=")?.is_not_equal(text))
}
/// `operator<=(handle<Units_Token>, const char*)`; `token_less_or_equal(&a, b"mm")?`.
pub fn token_less_or_equal(token: &Handle<Token>, text: &[u8]) -> Result<bool, Failure> {
    Ok(object(token, "operator<=")?.is_less_or_equal(text))
}
/// `operator>(handle<Units_Token>, const char*)`; `token_greater(&a, b"a")?`.
pub fn token_greater(token: &Handle<Token>, text: &[u8]) -> Result<bool, Failure> {
    Ok(object(token, "operator>")?.is_greater(text))
}
/// `operator>` on two tokens; `token_greater_token(&a, &b)?`.
pub fn token_greater_token(left: &Handle<Token>, right: &Handle<Token>) -> Result<bool, Failure> {
    object(left, "operator>")?.is_greater_token(right)
}
/// `operator>=` on two tokens; `token_greater_or_equal(&a, &b)?`.
pub fn token_greater_or_equal(
    left: &Handle<Token>,
    right: &Handle<Token>,
) -> Result<bool, Failure> {
    object(left, "operator>=")?.is_greater_or_equal(right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operators_forward_to_members() {
        let metre = Handle::new(Token::with_dimensions(
            b"m",
            b"U",
            1.0,
            &Dimensions::a_length(),
        ));
        assert!(token_equals(&metre, b"m").unwrap() && !token_not_equals(&metre, b"m").unwrap());
        assert!(
            token_less_or_equal(&metre, b"mm").unwrap() && token_greater(&metre, b"a").unwrap()
        );
        let area = token_pow(&metre, 2.0).unwrap();
        assert!(!token_greater_token(&area, &metre).unwrap());
        assert!(token_greater_or_equal(&metre, &area).unwrap());
        let sum = token_add(&metre, &metre).unwrap();
        assert_eq!(sum.get().unwrap().value(), 2.0);
        assert_eq!(
            token_subtract(&metre, &metre)
                .unwrap()
                .get()
                .unwrap()
                .value(),
            0.0
        );
        assert_eq!(
            token_multiply(&metre, &metre)
                .unwrap()
                .get()
                .unwrap()
                .word(),
            "(m)*(m)"
        );
        assert_eq!(
            token_divide(&metre, &sum).unwrap().get().unwrap().value(),
            0.5
        );
        assert_eq!(
            token_pow_token(&metre, &sum)
                .unwrap()
                .get()
                .unwrap()
                .value(),
            1.0
        );
        assert_eq!(
            token_add_integer(&metre, 4).unwrap().get().unwrap().word(),
            " "
        );
        let length = Dimensions::a_length();
        let time = Dimensions::a_time();
        let speed = dimensions_divide(&length, &time).unwrap();
        let back = dimensions_multiply(&speed, &time).unwrap();
        assert!(back.get().unwrap().is_equal(&length).unwrap());
        assert_eq!(
            dimensions_pow(&length, 3.0)
                .unwrap()
                .get()
                .unwrap()
                .length(),
            3.0
        );
        assert!(token_equals(&Handle::default(), b"m").is_err());
    }
}
