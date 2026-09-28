// =============================================================================
// Morton Code Tests
// =============================================================================

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_Origin() {
    let mut aCode: u32 = encode_morton_code(0, 0, 0);
    assert_eq!(aCode, 0u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_SingleBits() {
    // X=1, Y=0, Z=0 should give 1 (bit 0)
    let mut aCodeX: u32 = encode_morton_code(1, 0, 0);
    assert_eq!(aCodeX, 1u32);

    // X=0, Y=1, Z=0 should give 2 (bit 1)
    let mut aCodeY: u32 = encode_morton_code(0, 1, 0);
    assert_eq!(aCodeY, 2u32);

    // X=0, Y=0, Z=1 should give 4 (bit 2)
    let mut aCodeZ: u32 = encode_morton_code(0, 0, 1);
    assert_eq!(aCodeZ, 4u32);

    // X=1, Y=1, Z=1 should give 7 (bits 0, 1, 2)
    let mut aCodeXYZ: u32 = encode_morton_code(1, 1, 1);
    assert_eq!(aCodeXYZ, 7u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_Interleaving() {
    // X=2 (binary 10), Y=0, Z=0
    // Should produce binary ...001000 = 8
    let mut aCode1: u32 = encode_morton_code(2, 0, 0);
    assert_eq!(aCode1, 8u32);

    // X=0, Y=2, Z=0
    // Should produce binary ...010000 = 16
    let mut aCode2: u32 = encode_morton_code(0, 2, 0);
    assert_eq!(aCode2, 16u32);

    // X=0, Y=0, Z=2
    // Should produce binary ...100000 = 32
    let mut aCode3: u32 = encode_morton_code(0, 0, 2);
    assert_eq!(aCode3, 32u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_MaxValues() {
    // Test with maximum 10-bit values (1023)
    let mut aCode: u32 = encode_morton_code(1023, 1023, 1023);

    // All 30 bits should be set
    assert_eq!(aCode, 0x3FFFFFFFu32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_Symmetry() {
    // Swapping coordinates should produce predictable results
    let mut aCode1: u32 = encode_morton_code(100, 200, 300);
    let mut aCode2: u32 = encode_morton_code(200, 100, 300);
    let mut aCode3: u32 = encode_morton_code(100, 300, 200);

    // Codes should be different
    assert_ne!(aCode1, aCode2);
    assert_ne!(aCode1, aCode3);
    assert_ne!(aCode2, aCode3);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_Ordering() {
    // Points closer in space should have closer Morton codes
    // (0, 0, 0) vs (1, 0, 0) should be closer than (0, 0, 0) vs (0, 0, 1023)
    let mut aCode000: u32 = encode_morton_code(0, 0, 0);
    let mut aCode100: u32 = encode_morton_code(1, 0, 0);
    let mut aCode001023: u32 = encode_morton_code(0, 0, 1023);

    // (0,0,0) to (1,0,0) difference should be small
    let mut aDiff1: u32 = aCode100.wrapping_sub(aCode000);

    // (0,0,0) to (0,0,1023) difference should be large
    let mut aDiff2: u32 = aCode001023.wrapping_sub(aCode000);

    assert!(aDiff1 < aDiff2);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_MidValues() {
    // Test with mid-range values
    let mut aCode: u32 = encode_morton_code(512, 512, 512);

    // Should be non-zero and within 30-bit range
    assert!(aCode > 0u32);
    assert!(aCode <= 0x3FFFFFFFu32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_PowersOfTwo() {
    // Test powers of two for each axis
    let mut aCode4: u32 = encode_morton_code(4, 0, 0); // X=100 binary
    let mut aCode8: u32 = encode_morton_code(8, 0, 0); // X=1000 binary
    let mut aCode16: u32 = encode_morton_code(16, 0, 0); // X=10000 binary

    // Each should be 8x the previous (3 bit positions apart in Morton code)
    assert_eq!(aCode8, aCode4.wrapping_mul(8));
    assert_eq!(aCode16, aCode8.wrapping_mul(8));
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_ConsistentEncoding() {
    // Same inputs should always produce same outputs
    for _i in 0..100 {
        let mut aCode1: u32 = encode_morton_code(100, 200, 300);
        let mut aCode2: u32 = encode_morton_code(100, 200, 300);
        assert_eq!(aCode1, aCode2);
    }
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_BoundaryValues() {
    // Test boundary values near max (1023)
    let mut aCode1022: u32 = encode_morton_code(1022, 1022, 1022);
    let mut aCode1023: u32 = encode_morton_code(1023, 1023, 1023);

    assert!(aCode1022 < aCode1023);
    assert_eq!(aCode1023, 0x3FFFFFFFu32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_SmallValues() {
    // Test small values
    let mut aCode2: u32 = encode_morton_code(2, 2, 2);
    let mut aCode3: u32 = encode_morton_code(3, 3, 3);

    // Both should be small but aCode3 > aCode2
    assert!(aCode2 < 100u32);
    assert!(aCode3 < 100u32);
    assert!(aCode2 < aCode3);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_SingleAxisValues() {
    // Test with value only in one axis
    let mut aCodeX: u32 = encode_morton_code(255, 0, 0);
    let mut aCodeY: u32 = encode_morton_code(0, 255, 0);
    let mut aCodeZ: u32 = encode_morton_code(0, 0, 255);

    // X bits in positions 0, 3, 6, ...
    // Y bits in positions 1, 4, 7, ...
    // Z bits in positions 2, 5, 8, ...
    assert_ne!(aCodeX, aCodeY);
    assert_ne!(aCodeY, aCodeZ);
    assert_ne!(aCodeX, aCodeZ);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_SequentialXValues() {
    // Test that sequential X values produce increasing codes
    let mut aPrev: u32 = encode_morton_code(0, 500, 500);
    for x in 1u32..=10 {
        let mut aCurr: u32 = encode_morton_code(x, 500, 500);
        assert!(aCurr > aPrev);
        aPrev = aCurr;
    }
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_SequentialYValues() {
    // Test that sequential Y values produce increasing codes
    let mut aPrev: u32 = encode_morton_code(500, 0, 500);
    for y in 1u32..=10 {
        let mut aCurr: u32 = encode_morton_code(500, y, 500);
        assert!(aCurr > aPrev);
        aPrev = aCurr;
    }
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_SequentialZValues() {
    // Test that sequential Z values produce increasing codes
    let mut aPrev: u32 = encode_morton_code(500, 500, 0);
    for z in 1u32..=10 {
        let mut aCurr: u32 = encode_morton_code(500, 500, z);
        assert!(aCurr > aPrev);
        aPrev = aCurr;
    }
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_ZeroInDimensions() {
    // Test with zeros in different dimensions
    let mut aCodeXY: u32 = encode_morton_code(100, 100, 0);
    let mut aCodeXZ: u32 = encode_morton_code(100, 0, 100);
    let mut aCodeYZ: u32 = encode_morton_code(0, 100, 100);

    // All should be different
    assert_ne!(aCodeXY, aCodeXZ);
    assert_ne!(aCodeXY, aCodeYZ);
    assert_ne!(aCodeXZ, aCodeYZ);

    // All should be non-zero
    assert!(aCodeXY > 0u32);
    assert!(aCodeXZ > 0u32);
    assert!(aCodeYZ > 0u32);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_UpperByteBits() {
    // Test values that require upper byte (values >= 256)
    let mut aCode256: u32 = encode_morton_code(256, 0, 0);
    let mut aCode512: u32 = encode_morton_code(512, 0, 0);
    let mut aCode768: u32 = encode_morton_code(768, 0, 0);

    assert!(aCode256 > 0u32);
    assert!(aCode512 > aCode256);
    assert!(aCode768 > aCode512);
}

#[test]
fn BVH_RadixSorterTest__EncodeMortonCode_AllOnes() {
    // 255 = 0xFF (all ones in lower byte)
    let mut aCode: u32 = encode_morton_code(255, 255, 255);

    // Should be less than max (1023, 1023, 1023)
    assert!(aCode < 0x3FFFFFFFu32);
    assert!(aCode > 0u32);
}
