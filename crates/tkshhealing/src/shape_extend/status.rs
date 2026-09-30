// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend/ShapeExtend_Status.hxx and ShapeExtend_Parametrisation.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeExtend_Status` and `ShapeExtend_Parametrisation`: the status flags of the Shape
//! Healing tools and the global parametrisation of a composite surface.

/// `ShapeExtend_Status`: one flag of the status a Shape Healing tool records, in the
/// native order, so that `status as i32` is the native enumerator value.
///
/// A tool keeps its status as a bit field (`ShapeExtend::encode_status` gives the bits
/// of one flag); `Done` and `Fail` stand for any of the eight `DONEi` or `FAILi` flags,
/// and `Ok` for no flag at all.
///
/// ```
/// use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
/// assert_eq!(ShapeExtendStatus::Fail as i32, 18);
/// assert_eq!(ShapeExtendStatus::from_index(9), Some(ShapeExtendStatus::Done));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeExtendStatus {
    /// `ShapeExtend_OK`: nothing done, everything OK.
    Ok,
    /// `ShapeExtend_DONE1`: something was done, case 1.
    Done1,
    /// `ShapeExtend_DONE2`: something was done, case 2.
    Done2,
    /// `ShapeExtend_DONE3`: something was done, case 3.
    Done3,
    /// `ShapeExtend_DONE4`: something was done, case 4.
    Done4,
    /// `ShapeExtend_DONE5`: something was done, case 5.
    Done5,
    /// `ShapeExtend_DONE6`: something was done, case 6.
    Done6,
    /// `ShapeExtend_DONE7`: something was done, case 7.
    Done7,
    /// `ShapeExtend_DONE8`: something was done, case 8.
    Done8,
    /// `ShapeExtend_DONE`: something was done (any of the `DONEi`).
    Done,
    /// `ShapeExtend_FAIL1`: the method failed, case 1.
    Fail1,
    /// `ShapeExtend_FAIL2`: the method failed, case 2.
    Fail2,
    /// `ShapeExtend_FAIL3`: the method failed, case 3.
    Fail3,
    /// `ShapeExtend_FAIL4`: the method failed, case 4.
    Fail4,
    /// `ShapeExtend_FAIL5`: the method failed, case 5.
    Fail5,
    /// `ShapeExtend_FAIL6`: the method failed, case 6.
    Fail6,
    /// `ShapeExtend_FAIL7`: the method failed, case 7.
    Fail7,
    /// `ShapeExtend_FAIL8`: the method failed, case 8.
    Fail8,
    /// `ShapeExtend_FAIL`: the method failed (any of the `FAILi`).
    Fail,
}

impl ShapeExtendStatus {
    /// Every enumerator, in native order.
    pub const ALL: [Self; 19] = [
        Self::Ok,
        Self::Done1,
        Self::Done2,
        Self::Done3,
        Self::Done4,
        Self::Done5,
        Self::Done6,
        Self::Done7,
        Self::Done8,
        Self::Done,
        Self::Fail1,
        Self::Fail2,
        Self::Fail3,
        Self::Fail4,
        Self::Fail5,
        Self::Fail6,
        Self::Fail7,
        Self::Fail8,
        Self::Fail,
    ];

    /// The enumerator of native value `index`, `None` outside `[0, 18]`;
    /// `ShapeExtendStatus::from_index(0) == Some(ShapeExtendStatus::Ok)`.
    pub fn from_index(index: i32) -> Option<Self> {
        usize::try_from(index)
            .ok()
            .and_then(|i| Self::ALL.get(i).copied())
    }
}

/// `ShapeExtend_Parametrisation`: how a composite surface builds its global
/// parametrisation from its patches, in the native order.
///
/// ```
/// use rustycad_tkshhealing::shape_extend::ShapeExtendParametrisation;
/// assert_eq!(ShapeExtendParametrisation::Unitary as i32, 2);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapeExtendParametrisation {
    /// `ShapeExtend_Natural`: each patch of the first row and column adds its range,
    /// `U(i+1) = U(i) + URange(i, 1)`.
    Natural,
    /// `ShapeExtend_Uniform`: each patch gives the range 1, `U(i) = i - 1`.
    Uniform,
    /// `ShapeExtend_Unitary`: a uniform parametrisation of the global range `[0, 1]`.
    Unitary,
}
