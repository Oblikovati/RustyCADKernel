// Note: implementation is based on free samples from
// http://www.gamedev.ru/code/articles/?id=4215&page=3
// and maths found in Wikipedia and elsewhere

//! Native enumeration and Shoemake sequence parameters (Graphics Gems IV).
/// Selects an OCCT Euler convention; `EulerSequence::YawPitchRoll`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EulerSequence {
    /// Native gp_EulerAngles convention.
    EulerAngles = 0,
    /// Native gp_YawPitchRoll convention.
    YawPitchRoll = 1,
    /// Native gp_Extrinsic_XYZ convention.
    ExtrinsicXyz = 2,
    /// Native gp_Extrinsic_XZY convention.
    ExtrinsicXzy = 3,
    /// Native gp_Extrinsic_YZX convention.
    ExtrinsicYzx = 4,
    /// Native gp_Extrinsic_YXZ convention.
    ExtrinsicYxz = 5,
    /// Native gp_Extrinsic_ZXY convention.
    ExtrinsicZxy = 6,
    /// Native gp_Extrinsic_ZYX convention.
    ExtrinsicZyx = 7,
    /// Native gp_Intrinsic_XYZ convention.
    IntrinsicXyz = 8,
    /// Native gp_Intrinsic_XZY convention.
    IntrinsicXzy = 9,
    /// Native gp_Intrinsic_YZX convention.
    IntrinsicYzx = 10,
    /// Native gp_Intrinsic_YXZ convention.
    IntrinsicYxz = 11,
    /// Native gp_Intrinsic_ZXY convention.
    IntrinsicZxy = 12,
    /// Native gp_Intrinsic_ZYX convention.
    IntrinsicZyx = 13,
    /// Native gp_Extrinsic_XYX convention.
    ExtrinsicXyx = 14,
    /// Native gp_Extrinsic_XZX convention.
    ExtrinsicXzx = 15,
    /// Native gp_Extrinsic_YZY convention.
    ExtrinsicYzy = 16,
    /// Native gp_Extrinsic_YXY convention.
    ExtrinsicYxy = 17,
    /// Native gp_Extrinsic_ZYZ convention.
    ExtrinsicZyz = 18,
    /// Native gp_Extrinsic_ZXZ convention.
    ExtrinsicZxz = 19,
    /// Native gp_Intrinsic_XYX convention.
    IntrinsicXyx = 20,
    /// Native gp_Intrinsic_XZX convention.
    IntrinsicXzx = 21,
    /// Native gp_Intrinsic_YZY convention.
    IntrinsicYzy = 22,
    /// Native gp_Intrinsic_YXY convention.
    IntrinsicYxy = 23,
    /// Native gp_Intrinsic_ZXZ convention.
    IntrinsicZxz = 24,
    /// Native gp_Intrinsic_ZYZ convention.
    IntrinsicZyz = 25,
}
impl EulerSequence {
    /// All 24 non-alias conventions in native enum order; `EulerSequence::ALL`.
    pub const ALL: [Self; 24] = [
        Self::ExtrinsicXyz,
        Self::ExtrinsicXzy,
        Self::ExtrinsicYzx,
        Self::ExtrinsicYxz,
        Self::ExtrinsicZxy,
        Self::ExtrinsicZyx,
        Self::IntrinsicXyz,
        Self::IntrinsicXzy,
        Self::IntrinsicYzx,
        Self::IntrinsicYxz,
        Self::IntrinsicZxy,
        Self::IntrinsicZyx,
        Self::ExtrinsicXyx,
        Self::ExtrinsicXzx,
        Self::ExtrinsicYzy,
        Self::ExtrinsicYxy,
        Self::ExtrinsicZyz,
        Self::ExtrinsicZxz,
        Self::IntrinsicXyx,
        Self::IntrinsicXzx,
        Self::IntrinsicYzy,
        Self::IntrinsicYxy,
        Self::IntrinsicZxz,
        Self::IntrinsicZyz,
    ];
    pub(super) fn parameters(self) -> Parameters {
        let (i, odd, two, extrinsic) = PARAMETERS[self as usize];
        Parameters {
            i,
            j: (i + 1 + usize::from(odd)) % 3,
            k: (i + 1 + usize::from(!odd)) % 3,
            odd,
            two,
            extrinsic,
        }
    }
}
const PARAMETERS: [(usize, bool, bool, bool); 26] = [
    (2, false, true, false),
    (0, false, false, false),
    (0, false, false, true),
    (0, true, false, true),
    (1, false, false, true),
    (1, true, false, true),
    (2, false, false, true),
    (2, true, false, true),
    (2, true, false, false),
    (1, false, false, false),
    (0, true, false, false),
    (2, false, false, false),
    (1, true, false, false),
    (0, false, false, false),
    (0, false, true, true),
    (0, true, true, true),
    (1, false, true, true),
    (1, true, true, true),
    (2, true, true, true),
    (2, false, true, true),
    (0, false, true, false),
    (0, true, true, false),
    (1, false, true, false),
    (1, true, true, false),
    (2, false, true, false),
    (2, true, true, false),
];
#[derive(Clone, Copy)]
pub(super) struct Parameters {
    pub i: usize,
    pub j: usize,
    pub k: usize,
    pub odd: bool,
    pub two: bool,
    pub extrinsic: bool,
}
