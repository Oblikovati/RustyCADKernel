//! Representation checks beyond the seven OCC24537 declarations.
use rustycad_tkernel::fsd::BinaryFile;

#[test]
fn character_and_uint64_reverse_all_bytes() {
    assert_eq!(BinaryFile::inverse_ext_char(0x1234), 0x3412);
    assert_eq!(
        BinaryFile::inverse_uint64(0x0123_4567_89ab_cdef),
        0xefcd_ab89_6745_2301
    );
    for value in 0..=u16::MAX {
        assert_eq!(
            BinaryFile::inverse_ext_char(BinaryFile::inverse_ext_char(value)),
            value
        );
    }
}

#[test]
fn floating_payloads_survive_round_trip() {
    for bits in [0, 1, u64::MAX, 0x8000_0000_0000_0000, 0x7ff0_0000_0000_0001] {
        let reversed: f64 = BinaryFile::inverse_real(f64::from_bits(bits));
        assert_eq!(
            reversed.to_bits().to_ne_bytes(),
            bits.to_ne_bytes()
                .into_iter()
                .rev()
                .collect::<Vec<u8>>()
                .as_slice()
        );
        assert_eq!(BinaryFile::inverse_real(reversed).to_bits(), bits);
    }
    for bits in [0, 1, u32::MAX, 0x8000_0000, 0x7f80_0001] {
        let reversed: f32 = BinaryFile::inverse_short_real(f32::from_bits(bits));
        assert_eq!(BinaryFile::inverse_short_real(reversed).to_bits(), bits);
    }
}
