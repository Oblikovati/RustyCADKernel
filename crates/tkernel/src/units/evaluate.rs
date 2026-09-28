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

use super::{Failure, UnitToken, sequence_error};
pub(super) fn calculate(sequence: &mut Vec<UnitToken>) -> Result<Option<UnitToken>, Failure> {
    if sequence.len() == 1 {
        return Ok(Some(sequence[0].clone()));
    }
    if sequence.len() == 2 {
        return unary(sequence);
    }
    let mut result = None;
    while let Some(start) = sequence.iter().position(|token| token.word == b"(") {
        let end = closing(sequence, start)?;
        let token =
            calculate(&mut sequence[start + 1..end].to_vec())?.ok_or_else(sequence_error)?;
        sequence.splice(start..=end, [token.clone()]);
        result = Some(token);
    }
    reduce(sequence, true, &mut result)?;
    reduce(sequence, false, &mut result)?;
    Ok(result)
}
fn unary(sequence: &mut Vec<UnitToken>) -> Result<Option<UnitToken>, Failure> {
    if sequence[0].word == b"+" {
        sequence.remove(0);
    }
    if sequence[0].word == b"-" {
        let mut token = sequence.get(1).ok_or_else(sequence_error)?.clone();
        token.word.insert(0, b'-');
        token.value *= -1.0;
        token.shift = 0.0;
        sequence.remove(0);
        sequence[0] = token;
    }
    Ok(Some(sequence[0].clone()))
}
fn closing(sequence: &[UnitToken], start: usize) -> Result<usize, Failure> {
    let mut depth = 1;
    for (index, token) in sequence.iter().enumerate().skip(start + 1) {
        if token.word == b")" {
            depth -= 1;
        }
        if token.word == b"(" {
            depth += 1;
        }
        if depth == 0 {
            return Ok(index);
        }
    }
    Err(sequence_error())
}
fn reduce(
    sequence: &mut Vec<UnitToken>,
    powers: bool,
    result: &mut Option<UnitToken>,
) -> Result<(), Failure> {
    let mut index = 0;
    while index < sequence.len() {
        let selected = if powers {
            sequence[index].word == b"**"
        } else {
            sequence[index].meaning == "O"
        };
        if !selected {
            index += 1;
            continue;
        }
        let left = sequence
            .get(index.wrapping_sub(1))
            .ok_or_else(sequence_error)?;
        let right = sequence.get(index + 1).ok_or_else(sequence_error)?;
        let operator = if powers {
            b"**".as_slice()
        } else if sequence[index].word == b"/" {
            b"/"
        } else {
            b"*"
        };
        let token = left.multiply(right, operator);
        *result = Some(token.clone());
        sequence.splice(index - 1..=index + 1, [token]);
    }
    Ok(())
}
