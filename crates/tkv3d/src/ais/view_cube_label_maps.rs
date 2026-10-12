//! Native enum hashes preserve DataMap traversal and arbitrary protected label bindings.
use crate::{prs3d::Prs3dDatumParts, v3d::V3dTypeOfOrientation};
use rustycad_tkernel::{
    ncollection::{DataMap, HashPolicy},
    tcollection::AsciiString,
};
/// Pinned native std::hash(enum) identity policy; `AisViewCubeLabelHash`.
#[derive(Clone, Copy, Debug, Default)]
pub struct AisViewCubeLabelHash;
impl HashPolicy<V3dTypeOfOrientation> for AisViewCubeLabelHash {
    fn hash(&self, key: &V3dTypeOfOrientation) -> usize {
        *key as i32 as usize
    }
    fn equal(&self, left: &V3dTypeOfOrientation, right: &V3dTypeOfOrientation) -> bool {
        left == right
    }
}
impl HashPolicy<Prs3dDatumParts> for AisViewCubeLabelHash {
    fn hash(&self, key: &Prs3dDatumParts) -> usize {
        *key as i32 as usize
    }
    fn equal(&self, left: &Prs3dDatumParts, right: &Prs3dDatumParts) -> bool {
        left == right
    }
}
/// Actual protected side-label map; missing and arbitrary orientation keys are retained.
pub type AisViewCubeSideLabels = DataMap<V3dTypeOfOrientation, AsciiString, AisViewCubeLabelHash>;
/// Actual protected datum-label map; non-axis keys remain observable through AxisLabel.
pub type AisViewCubeAxesLabels = DataMap<Prs3dDatumParts, AsciiString, AisViewCubeLabelHash>;
pub(super) fn default_cube_side_labels() -> AisViewCubeSideLabels {
    let mut labels = AisViewCubeSideLabels::default();
    for (orientation, label) in V3dTypeOfOrientation::ALL[..6]
        .iter()
        .zip(["RIGHT", "BACK", "TOP", "LEFT", "FRONT", "BOTTOM"])
    {
        labels.bind(*orientation, AsciiString::from(label));
    }
    labels
}
pub(super) fn default_cube_axes_labels() -> AisViewCubeAxesLabels {
    let mut labels = AisViewCubeAxesLabels::default();
    for (axis, label) in [
        Prs3dDatumParts::XAxis,
        Prs3dDatumParts::YAxis,
        Prs3dDatumParts::ZAxis,
    ]
    .into_iter()
    .zip(["X", "Y", "Z"])
    {
        labels.bind(axis, AsciiString::from(label));
    }
    labels
}
