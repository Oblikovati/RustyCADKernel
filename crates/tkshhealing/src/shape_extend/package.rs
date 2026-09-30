// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend/ShapeExtend.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeExtend`: the package functions (the status bit codes).
use super::status::ShapeExtendStatus;

/// `ShapeExtend`: the tools common to the Shape Healing packages.
///
/// ```
/// use rustycad_tkshhealing::shape_extend::{ShapeExtend, ShapeExtendStatus};
/// let flag = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
/// assert_eq!(flag, 0x0004);
/// assert!(ShapeExtend::decode_status(flag, ShapeExtendStatus::Done));
/// assert!(!ShapeExtend::decode_status(flag, ShapeExtendStatus::Fail));
/// ```
pub struct ShapeExtend;

impl ShapeExtend {
    /// `EncodeStatus`: the bit flag of `status`: `DONEi` is bit `i - 1`, `FAILi` is bit
    /// `i + 7`, `DONE` and `FAIL` are all eight bits of their byte, `OK` is 0;
    /// `ShapeExtend::encode_status(ShapeExtendStatus::Fail1) == 0x0100`.
    pub fn encode_status(status: ShapeExtendStatus) -> i32 {
        match status {
            ShapeExtendStatus::Ok => 0x0000,
            ShapeExtendStatus::Done => 0x00ff,
            ShapeExtendStatus::Fail => 0xff00,
            // DONE1..DONE8 are the enumerators 1..8 and FAIL1..FAIL8 the enumerators
            // 10..17: one bit each, in order, as the native switch spells them.
            done if (done as i32) < (ShapeExtendStatus::Done as i32) => 1 << (done as i32 - 1),
            fail => 1 << (fail as i32 - 2),
        }
    }

    /// `DecodeStatus`: whether `flag` holds the bit of `status`; for `OK`, whether `flag`
    /// holds no bit at all; `ShapeExtend::decode_status(0, ShapeExtendStatus::Ok)`.
    pub fn decode_status(flag: i32, status: ShapeExtendStatus) -> bool {
        if status == ShapeExtendStatus::Ok {
            return flag == 0;
        }
        (flag & Self::encode_status(status)) != 0
    }
}
