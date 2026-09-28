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

//! Read access to the generated Resource_Unicode tables (see `tables.rs`).
use super::tables;

/// Entry `index` of a sparse table: sorted `(index, value)` u16 pairs, absent means 0.
fn sparse(table: &[u8], index: u16) -> u16 {
    let (mut low, mut high) = (0, table.len() / 4);
    while low < high {
        let middle = (low + high) / 2;
        let key = u16::from_le_bytes([table[4 * middle], table[4 * middle + 1]]);
        match key.cmp(&index) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
            std::cmp::Ordering::Equal => {
                return u16::from_le_bytes([table[4 * middle + 2], table[4 * middle + 3]]);
            }
        }
    }
    0
}
fn dense_u16(table: &[u8], index: usize) -> u16 {
    u16::from_le_bytes([table[2 * index], table[2 * index + 1]])
}
/// Native `unisjis[uni]`: Unicode to Shift-JIS.
pub(super) fn unisjis(index: u16) -> u16 {
    sparse(tables::UNISJIS.0, index)
}
/// Native `sjisuni[sjis]`: Shift-JIS to Unicode.
pub(super) fn sjisuni(index: u16) -> u16 {
    sparse(tables::SJISUNI.0, index)
}
/// Native `unigb[uni]`: Unicode to GB2312 (7-bit row/cell).
pub(super) fn unigb(index: u16) -> u16 {
    sparse(tables::UNIGB.0, index)
}
/// Native `gbuni[gb]`: GB2312 (7-bit row/cell) to Unicode.
pub(super) fn gbuni(index: u16) -> u16 {
    sparse(tables::GBUNI.0, index)
}
/// Native `gbkuni[pointer]`, `pointer < 23940`.
pub(super) fn gbkuni(pointer: usize) -> u16 {
    dense_u16(tables::GBKUNI.0, pointer)
}
/// Native `big5uni[pointer]`, `pointer < 19782`; values above U+FFFF are astral code points.
pub(super) fn big5uni(pointer: usize) -> u32 {
    let table = tables::BIG5UNI.0;
    let bytes = [
        table[4 * pointer],
        table[4 * pointer + 1],
        table[4 * pointer + 2],
        table[4 * pointer + 3],
    ];
    u32::from_le_bytes(bytes)
}
/// Native `THE_CODEPAGES_ANSI[row][column]`: the UTF-16 unit of byte `0x80 | column`.
pub(super) fn code_page(row: usize, column: usize) -> u16 {
    dense_u16(tables::CODE_PAGES.0, row * 128 + column)
}

#[cfg(test)]
mod tests {
    use super::*;
    /// FNV-1a-64, the digest tools/generate_resource_tables.py records for each blob.
    fn fnv1a64(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf2_9ce4_8422_2325, |digest, &byte| {
            (digest ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    }
    #[test]
    fn embedded_tables_match_generated_digests_and_sizes() {
        for (table, bytes) in [
            (tables::UNISJIS, 28172),
            (tables::SJISUNI, 28148),
            (tables::UNIGB, 29776),
            (tables::GBUNI, 29780),
            (tables::GBKUNI, 23940 * 2),
            (tables::BIG5UNI, 19782 * 4),
            (tables::CODE_PAGES, 19 * 128 * 2),
        ] {
            assert_eq!((table.0.len(), fnv1a64(table.0)), (bytes, table.1));
        }
    }
    #[test]
    fn table_entries_read_back_native_literals() {
        // First literals of each header, and the ASCII backslash that unisjis maps to 0x815f.
        assert_eq!(
            (unisjis(0x5c), unisjis(0x41), unisjis(0x7f)),
            (0x815f, 0x41, 0)
        );
        assert_eq!((gbkuni(0), gbkuni(1), big5uni(0)), (0x4E02, 0x4E04, 0));
        assert_eq!(
            (code_page(0, 0), code_page(0, 127), code_page(1, 0)),
            (0x20ac, 0x2d9, 0x402)
        );
    }
}
