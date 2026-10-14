// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularOrientationOptions {
    pub version: u32,
}
impl AnnularOrientationOptions {
    /// Select the annular conditional rule; `AnnularOrientationOptions::v2()` changes no join policy.
    pub(in crate::sketch_region_prism) fn v2() -> Self {
        Self { version: 2 }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sketch_region_prism) struct AnnularBoundaryDirection {
    pub source: RegionSourceId,
    pub traversed_winding: i64,
    pub reversed: bool,
    pub native_winding: i8,
    pub material_side: i8,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularFaceDirection {
    pub(super) named: NamedAnnularFace,
    pub(super) parents: JoinedParentDispositions,
    pub(super) actual: Orientation,
    pub(super) expected: Orientation,
    pub(super) trims: Vec<AnnularTrimDirection>,
}
#[derive(Clone, Debug, PartialEq)]
pub(in crate::sketch_region_prism) struct AnnularTrimDirection {
    pub(super) named: JoinedPrismTrim,
    pub(super) boundary: RegionSourceId,
    pub(super) parents: JoinedParentDispositions,
    pub(super) actual: Orientation,
    pub(super) expected: Orientation,
}
pub(in crate::sketch_region_prism) struct AnnularOrientationCertificate {
    pub(super) requested: AnnularOrientationOptions,
    pub(super) binding: Box<[u8]>,
    pub(super) jacobian_sign: i8,
    pub(super) boundaries: [AnnularBoundaryDirection; 2],
    pub(super) faces: Vec<AnnularFaceDirection>,
}
