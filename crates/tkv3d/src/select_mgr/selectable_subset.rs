//! Native selection BVH subsets; the count is a sentinel, never a tree index.
use rustycad_tkernel::standard::Failure;

/// Camera-dependence classes of selectable objects; `SelectMgrBvhSubset::ThreeD`.
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectMgrBvhSubset {
    /// Regular world-space bounds.
    ThreeD,
    /// Camera-dependent persistent world-space bounds.
    ThreeDPersistent,
    /// Projection-dependent screen-space bounds.
    TwoDPersistent,
    /// World-space bounds with orthographic projection.
    OrthoThreeDPersistent,
    /// Screen-space bounds with orthographic projection.
    OrthoTwoDPersistent,
}
impl SelectMgrBvhSubset {
    /// Native `BVHSubsetNb`; `assert_eq!(SelectMgrBvhSubset::COUNT, 5)`.
    pub const COUNT: usize = 5;
    pub(super) const ALL: [Self; Self::COUNT] = [
        Self::ThreeD,
        Self::ThreeDPersistent,
        Self::TwoDPersistent,
        Self::OrthoThreeDPersistent,
        Self::OrthoTwoDPersistent,
    ];
    pub(super) fn is_two_dimensional(self) -> bool {
        matches!(self, Self::TwoDPersistent | Self::OrthoTwoDPersistent)
    }
    pub(super) fn is_orthographic(self) -> bool {
        matches!(
            self,
            Self::OrthoThreeDPersistent | Self::OrthoTwoDPersistent
        )
    }
}
impl TryFrom<i32> for SelectMgrBvhSubset {
    type Error = Failure;
    /// Validate a native subset value; `SelectMgrBvhSubset::try_from(0)?`.
    fn try_from(value: i32) -> Result<Self, Failure> {
        Self::ALL.get(value as usize).copied().ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("BVH subset={value}; expected 0..5"),
            )
        })
    }
}
