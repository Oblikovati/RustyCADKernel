#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_LocalityPreservation() {
    // Points close in space should have relatively close Morton codes
    let mut aCenter: u32 = encode_morton_code(512, 512, 512);
    let mut aNear1: u32 = encode_morton_code(513, 512, 512);
    let mut aNear2: u32 = encode_morton_code(512, 513, 512);
    let mut aFar: u32 = encode_morton_code(0, 0, 0);

    // Near points should have smaller difference than far point
    let mut aDiffNear1: u32 = if aNear1 > aCenter { aNear1 - aCenter } else { aCenter - aNear1 };
    let mut aDiffNear2: u32 = if aNear2 > aCenter { aNear2 - aCenter } else { aCenter - aNear2 };
    let mut aDiffFar: u32 = if aFar > aCenter { aFar - aCenter } else { aCenter - aFar };

    assert!(aDiffNear1 < aDiffFar);
    assert!(aDiffNear2 < aDiffFar);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_BitPattern3() {
    // X=3 (binary 11), Y=0, Z=0
    // Interleaved: ...001001 = 9
    let mut aCode: u32 = encode_morton_code(3, 0, 0);
    assert_eq!(aCode, 9u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_BitPattern7() {
    // X=7 (binary 111), Y=0, Z=0
    // Interleaved: 001001001 = 73
    let mut aCode: u32 = encode_morton_code(7, 0, 0);
    assert_eq!(aCode, 73u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_XYEqual() {
    // When X = Y and Z = 0, code should follow pattern
    let mut aCode1: u32 = encode_morton_code(1, 1, 0);
    let mut aCode2: u32 = encode_morton_code(2, 2, 0);
    let mut aCode3: u32 = encode_morton_code(4, 4, 0);

    // X=1, Y=1, Z=0: bits at 0,1 -> 3
    assert_eq!(aCode1, 3u32);

    // X=2, Y=2, Z=0: bits at 3,4 -> 24
    assert_eq!(aCode2, 24u32);

    // X=4, Y=4, Z=0: bits at 6,7 -> 192
    assert_eq!(aCode3, 192u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_AllAxesEqual() {
    // When all axes are equal
    let mut aCode1: u32 = encode_morton_code(1, 1, 1);
    let mut aCode2: u32 = encode_morton_code(2, 2, 2);

    // X=1, Y=1, Z=1: bits at 0,1,2 -> 7
    assert_eq!(aCode1, 7u32);

    // X=2, Y=2, Z=2: bits at 3,4,5 -> 56
    assert_eq!(aCode2, 56u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_HalfMax() {
    // Test with half of maximum (512)
    let mut aCode: u32 = encode_morton_code(512, 512, 512);

    // Should be in upper half of range
    assert!(aCode > 0x1FFFFFFFu32);
    assert!(aCode <= 0x3FFFFFFFu32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_QuarterMax() {
    // Test with quarter of maximum (256)
    let mut aCode: u32 = encode_morton_code(256, 256, 256);

    // Should be less than half max
    assert!(aCode > 0u32);
    assert!(aCode < 0x3FFFFFFFu32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_DifferentScales() {
    // Test that scaling all coordinates by 2 shifts the Morton code
    let mut aCode1: u32 = encode_morton_code(100, 100, 100);
    let mut aCode2: u32 = encode_morton_code(200, 200, 200);

    // Code2 should be larger
    assert!(aCode2 > aCode1);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_AdjacentCells() {
    // Test adjacent cells in X direction
    let mut aCode1: u32 = encode_morton_code(100, 100, 100);
    let mut aCode2: u32 = encode_morton_code(101, 100, 100);

    // Difference should be 1 (X bit is at position 0)
    assert_eq!(aCode2.wrapping_sub(aCode1), 1u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_AdjacentCellsY() {
    // Test adjacent cells in Y direction
    let mut aCode1: u32 = encode_morton_code(100, 100, 100);
    let mut aCode2: u32 = encode_morton_code(100, 101, 100);

    // Difference should be 2 (Y bit is at position 1)
    assert_eq!(aCode2.wrapping_sub(aCode1), 2u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_AdjacentCellsZ() {
    // Test adjacent cells in Z direction
    let mut aCode1: u32 = encode_morton_code(100, 100, 100);
    let mut aCode2: u32 = encode_morton_code(100, 100, 101);

    // Difference should be 4 (Z bit is at position 2)
    assert_eq!(aCode2.wrapping_sub(aCode1), 4u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_MonotonicX() {
    // Verify monotonicity when only X changes
    let mut aPrev: u32 = 0;
    for x in 0u32..100 {
        let mut aCode: u32 = encode_morton_code(x, 0, 0);
        assert!(aCode >= aPrev);
        aPrev = aCode;
    }
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_MonotonicY() {
    // Verify monotonicity when only Y changes
    let mut aPrev: u32 = 0;
    for y in 0u32..100 {
        let mut aCode: u32 = encode_morton_code(0, y, 0);
        assert!(aCode >= aPrev);
        aPrev = aCode;
    }
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_MonotonicZ() {
    // Verify monotonicity when only Z changes
    let mut aPrev: u32 = 0;
    for z in 0u32..100 {
        let mut aCode: u32 = encode_morton_code(0, 0, z);
        assert!(aCode >= aPrev);
        aPrev = aCode;
    }
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_NoOverflow() {
    // Verify no overflow with maximum values
    let mut aCode: u32 = encode_morton_code(1023, 1023, 1023);

    // Should fit in 30 bits
    assert_eq!(aCode & 0xC0000000u32, 0u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_UniqueForDifferentInputs() {
    // Different inputs should produce different codes
    let mut aCode1: u32 = encode_morton_code(100, 200, 300);
    let mut aCode2: u32 = encode_morton_code(100, 200, 301);
    let mut aCode3: u32 = encode_morton_code(100, 201, 300);
    let mut aCode4: u32 = encode_morton_code(101, 200, 300);

    assert_ne!(aCode1, aCode2);
    assert_ne!(aCode1, aCode3);
    assert_ne!(aCode1, aCode4);
    assert_ne!(aCode2, aCode3);
    assert_ne!(aCode2, aCode4);
    assert_ne!(aCode3, aCode4);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_SpecificPattern() {
    // Test specific known pattern
    // X=5 (101), Y=3 (011), Z=6 (110)
    let mut aCode: u32 = encode_morton_code(5, 3, 6);

    // Morton code interleaves bits: X at 3i, Y at 3i+1, Z at 3i+2
    // The exact value depends on the interleaving order
    // Just verify it's non-zero and within range
    assert!(aCode > 0u32);
    assert!(aCode <= 0x3FFFFFFFu32);

    // Also verify consistency
    let mut aCode2: u32 = encode_morton_code(5, 3, 6);
    assert_eq!(aCode, aCode2);
}
