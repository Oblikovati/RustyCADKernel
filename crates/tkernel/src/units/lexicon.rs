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

use super::UnitToken;
use crate::tcollection::AsciiString;
pub(super) fn formula() -> Vec<UnitToken> {
    let mut lexicon = Vec::new();
    for word in [b"(".as_slice(), b")"] {
        lexicon.push(UnitToken::new(word, "S", 0.0));
    }
    for word in [b"+".as_slice(), b"-", b"*", b".", b"/", b"**"] {
        lexicon.push(UnitToken::new(word, "O", 0.0));
    }
    for (word, power) in [
        (b"\xb2".as_slice(), 2.0),
        (b"p2", 2.0),
        (b"sq.", 2.0),
        (b"\xb3", 3.0),
        (b"cu.", 3.0),
    ] {
        lexicon.push(UnitToken::new(word, "P", power));
    }
    for (word, value) in [
        (b"y".as_slice(), 1e-24),
        (b"z", 1e-21),
        (b"a", 1e-18),
        (b"f", 1e-15),
        (b"p", 1e-12),
        (b"n", 1e-9),
        (b"\xb5", 1e-6),
        (b"m", 1e-3),
        (b"c", 1e-2),
        (b"d", 1e-1),
        (b"da", 1e1),
        (b"h", 1e2),
        (b"k", 1e3),
        (b"M", 1e6),
        (b"G", 1e9),
        (b"T", 1e12),
        (b"P", 1e15),
        (b"E", 1e18),
        (b"Z", 1e21),
        (b"Y", 1e24),
    ] {
        lexicon.push(UnitToken::new(word, "M", value));
    }
    for word in [b"\xb6".as_slice(), b"Pi"] {
        lexicon.push(UnitToken::new(word, "", std::f64::consts::PI));
    }
    lexicon
}
pub(super) fn add_unit(lexicon: &mut Vec<UnitToken>, word: &[u8]) {
    if let Some(token) = lexicon.iter_mut().find(|token| token.word == word) {
        token.meaning.push('U');
    } else {
        lexicon.push(UnitToken::new(word, "U", 0.0));
    }
}
pub(super) fn tokenize(lexicon: &[UnitToken], expression: &[u8]) -> Vec<UnitToken> {
    let mut sequence: Vec<UnitToken> = Vec::new();
    let mut rest = expression.split(|b| *b == 0).next().unwrap_or_default();
    while !rest.is_empty() {
        let token = lexicon
            .iter()
            .filter(|token| rest.starts_with(&token.word))
            .max_by_key(|token| token.word.len())
            .cloned()
            .or_else(|| number(rest));
        let Some(token) = token else {
            return Vec::new();
        };
        if sequence
            .last()
            .is_some_and(|previous| invalid_pair(previous, &token))
        {
            return Vec::new();
        }
        rest = &rest[token.word.len()..];
        sequence.push(token);
    }
    sequence
}
fn number(bytes: &[u8]) -> Option<UnitToken> {
    if !bytes[0].is_ascii_digit() {
        return None;
    }
    let mut dot = false;
    let mut length = 1;
    for &byte in &bytes[1..] {
        if byte == b'.' && !dot {
            dot = true;
        } else if !byte.is_ascii_digit() {
            break;
        }
        length += 1;
    }
    Some(UnitToken::new(&bytes[..length], "0", 0.0))
}
fn invalid_pair(previous: &UnitToken, current: &UnitToken) -> bool {
    let before = previous.meaning.as_str();
    let after = current.meaning.as_str();
    (after == "M" && matches!(before, "M" | "MU" | "0"))
        || (after == "U" && matches!(before, "U" | "0"))
        || (after == "O" && matches!(before, "M" | "O"))
        || (matches!(current.word.as_slice(), b"+" | b"-") && previous.word != b"(")
        || (after == "S" && before == "M")
        || (after == "0" && matches!(before, "M" | "U" | "MU"))
}
pub(super) fn constants(sequence: &mut [UnitToken]) {
    for token in sequence.iter_mut().filter(|token| token.value == 0.0) {
        let word = AsciiString::from_bytes(&token.word);
        if word.is_real_value(false) {
            token.meaning = "K".into();
            token.value = word.real_value().unwrap();
        }
    }
}
