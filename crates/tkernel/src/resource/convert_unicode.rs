// Copyright (c) 1998-1999 Matra Datavision
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

//! Resource_ConvertUnicode.c: byte-pair conversions between Unicode, Shift-JIS, EUC and
//! GB2312. Each function takes the native `(*ph, *pl)` pair and returns the updated pair;
//! unsigned C arithmetic is kept with wrapping operations.
use super::table_lookup;

/// A native `(high, low)` byte pair held in `unsigned int`s.
pub(super) type BytePair = (u32, u32);

pub(super) fn is_euc(c: u32) -> bool {
    (0xa1..=0xfe).contains(&c)
}
pub(super) fn is_sjis1(c: u32) -> bool {
    (0x81..=0x9f).contains(&c) || (0xe0..=0xef).contains(&c)
}
fn is_sjis2(c: u32) -> bool {
    (0x40..=0xfc).contains(&c) && c != 0x7f
}
/// The C macro `isshift`: 0x80..=0xff.
pub(super) fn is_shift(c: u32) -> bool {
    (0x80..=0xff).contains(&c)
}
fn above_byte((ph, pl): BytePair) -> bool {
    (ph & 0xFFFF_FF00) != 0 || (pl & 0xFFFF_FF00) != 0
}
fn split(value: u16) -> BytePair {
    (u32::from(value >> 8), u32::from(value & 0xFF))
}
fn sjis_to_jis((ph, pl): BytePair) -> BytePair {
    if !is_sjis1(ph) || !is_sjis2(pl) {
        return (ph, pl);
    }
    let high = match (ph <= 0x9f, pl < 0x9f) {
        (true, true) => (ph << 1).wrapping_sub(0xe1),
        (true, false) => (ph << 1).wrapping_sub(0xe0),
        (false, true) => (ph << 1).wrapping_sub(0x161),
        (false, false) => (ph << 1).wrapping_sub(0x160),
    };
    let low = if pl < 0x7f {
        pl - 0x1f
    } else if pl < 0x9f {
        pl - 0x20
    } else {
        pl - 0x7e
    };
    (high, low)
}
fn jis_to_sjis((ph, pl): BytePair) -> BytePair {
    let low = match (ph & 1 != 0, pl < 0x60) {
        (true, true) => pl.wrapping_add(0x1f),
        (true, false) => pl.wrapping_add(0x20),
        (false, _) => pl.wrapping_add(0x7e),
    };
    let high = if ph < 0x5f {
        (ph + 0xe1) >> 1
    } else {
        (ph + 0x161) >> 1
    };
    (high, low)
}
fn euc_to_sjis(pair: BytePair) -> BytePair {
    if above_byte(pair) {
        return (0, 0);
    }
    if !is_euc(pair.0) || !is_euc(pair.1) {
        return pair;
    }
    jis_to_sjis((pair.0 & 0x7F, pair.1 & 0x7F))
}
fn sjis_to_euc(pair: BytePair) -> BytePair {
    if above_byte(pair) {
        return (0, 0);
    }
    if !is_sjis1(pair.0) || !is_sjis2(pair.1) || pair == (0, 0) {
        return pair;
    }
    let (ph, pl) = sjis_to_jis(pair);
    (ph | 0x80, pl | 0x80)
}
/// Native `Resource_sjis_to_unicode`.
pub(super) fn sjis_to_unicode(pair: BytePair) -> BytePair {
    if above_byte(pair) {
        return (0, 0);
    }
    if !is_sjis1(pair.0) || !is_sjis2(pair.1) {
        return pair;
    }
    split(table_lookup::sjisuni(((pair.0 << 8) | pair.1) as u16))
}
/// Native `Resource_unicode_to_sjis`.
pub(super) fn unicode_to_sjis(pair: BytePair) -> BytePair {
    if above_byte(pair) {
        return (0, 0);
    }
    if pair == (0, 0) {
        return pair;
    }
    split(table_lookup::unisjis(((pair.0 << 8) | pair.1) as u16))
}
/// Native `Resource_unicode_to_euc`: through Shift-JIS, keeping non-SJIS results as ANSI.
pub(super) fn unicode_to_euc(pair: BytePair) -> BytePair {
    if pair == (0, 0) {
        return pair;
    }
    let sjis = unicode_to_sjis(pair);
    if is_sjis1(sjis.0) {
        sjis_to_euc(sjis)
    } else {
        sjis
    }
}
/// Native `Resource_euc_to_unicode`: through Shift-JIS.
pub(super) fn euc_to_unicode(pair: BytePair) -> BytePair {
    if !is_euc(pair.0) || !is_euc(pair.1) || pair == (0, 0) {
        return pair;
    }
    sjis_to_unicode(euc_to_sjis(pair))
}
/// Native `Resource_gb_to_unicode`: the 7-bit row/cell indexes `gbuni`.
pub(super) fn gb_to_unicode(pair: BytePair) -> BytePair {
    if above_byte(pair) {
        return (0, 0);
    }
    if !is_shift(pair.0) || !is_shift(pair.1) {
        return pair;
    }
    split(table_lookup::gbuni(
        (((pair.0 & 0x7f) << 8) | (pair.1 & 0x7f)) as u16,
    ))
}
/// Native `Resource_unicode_to_gb`: unmapped characters become `(0, 0)`.
pub(super) fn unicode_to_gb(pair: BytePair) -> BytePair {
    if above_byte(pair) {
        return (0, 0);
    }
    if pair == (0, 0) {
        return pair;
    }
    let gb = table_lookup::unigb(((pair.0 << 8) | pair.1) as u16);
    if gb == 0 {
        return (0, 0);
    }
    (u32::from(gb >> 8) | 0x80, u32::from(gb & 0xFF) | 0x80)
}
