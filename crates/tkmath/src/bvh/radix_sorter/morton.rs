// Created on: 2016-04-13
// Created by: Denis BOGOLEPOV
// Copyright (c) 2013-2016 OPEN CASCADE SAS
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
// Rust translation of FoundationClasses/TKMath/BVH/BVH_RadixSorter.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BVH::THE_MORTON_LUT` and `BVH::EncodeMortonCode`.

/// Spreads the bits of an 8-bit value to every third position (`THE_MORTON_LUT`).
pub const THE_MORTON_LUT: [u32; 256] = spread_table();

/// Builds the table the upstream header lists literally: bit `i` of the index
/// lands at bit `3 * i` of the entry.
const fn spread_table() -> [u32; 256] {
    let mut table = [0_u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut bit = 0;
        while bit < 8 {
            if index & (1 << bit) != 0 {
                table[index] |= 1 << (3 * bit);
            }
            bit += 1;
        }
        index += 1;
    }
    table
}

/// Spreads a 10-bit voxel coordinate: low byte via the table, upper two bits
/// via the table shifted by 24.
const fn spread_coordinate(voxel: u32) -> u32 {
    THE_MORTON_LUT[(voxel & 0xFF) as usize] | (THE_MORTON_LUT[((voxel >> 8) & 0x03) as usize] << 24)
}

/// Encodes 10-bit voxel coordinates into a 30-bit Morton code (X at bits 3i,
/// Y at 3i+1, Z at 3i+2); bits above the tenth are ignored.
/// Example: `encode_morton_code(1, 1, 1) == 7`.
pub const fn encode_morton_code(voxel_x: u32, voxel_y: u32, voxel_z: u32) -> u32 {
    spread_coordinate(voxel_x)
        | (spread_coordinate(voxel_y) << 1)
        | (spread_coordinate(voxel_z) << 2)
}
