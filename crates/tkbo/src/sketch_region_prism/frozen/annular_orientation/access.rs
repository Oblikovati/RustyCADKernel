// SPDX-License-Identifier: GPL-2.0-only
use super::*;
impl AnnularOrientationCertificate {
    /// Compare the complete immutable input; `proof.binds(join)` never compares only IDs or counts.
    pub(in crate::sketch_region_prism) fn binds(&self, join: &AnnularFrozenDomain) -> bool {
        &*self.binding == join.replay_bytes()
    }
    /// Borrow complete source bytes; `proof.source_binding()` includes every joined policy and operand.
    pub(in crate::sketch_region_prism) fn source_binding(&self) -> &[u8] {
        &self.binding
    }
    /// Read unchanged policy; `proof.requested()` retains the behavior version.
    pub(in crate::sketch_region_prism) fn requested(&self) -> AnnularOrientationOptions {
        self.requested
    }
    /// Read exact determinant sign; `proof.jacobian_sign()` is never inferred from measured volume.
    pub(in crate::sketch_region_prism) fn jacobian_sign(&self) -> i8 {
        self.jacobian_sign
    }
    /// Borrow source-specific winding provenance; `proof.boundaries()` keeps traversal separate.
    pub(in crate::sketch_region_prism) fn boundaries(&self) -> &[AnnularBoundaryDirection; 2] {
        &self.boundaries
    }
    /// Borrow all named directional conclusions; `proof.faces()` includes both distinct Generated contexts.
    pub(in crate::sketch_region_prism) fn faces(&self) -> &[AnnularFaceDirection] {
        &self.faces
    }
}
impl AnnularFaceDirection {
    /// Borrow full authored face and boundary paths; `row.named()` retains cap-hole roles.
    pub(in crate::sketch_region_prism) fn named(&self) -> &NamedAnnularFace {
        &self.named
    }
    /// Borrow every original parent class; `row.parents()` preserves multiplicity.
    pub(in crate::sketch_region_prism) fn parents(&self) -> &JoinedParentDispositions {
        &self.parents
    }
    /// Read observed and required directions; `row.orientations()` contains exact discrete orientations.
    pub(in crate::sketch_region_prism) fn orientations(&self) -> (Orientation, Orientation) {
        (self.actual, self.expected)
    }
    /// Borrow all directed trims; `row.trims()` keeps repeated rim and seam uses distinct.
    pub(in crate::sketch_region_prism) fn trims(&self) -> &[AnnularTrimDirection] {
        &self.trims
    }
}
impl AnnularTrimDirection {
    /// Borrow the actual named trim; `row.named()` retains its source, native interval and parents.
    pub(in crate::sketch_region_prism) fn named(&self) -> &JoinedPrismTrim {
        &self.named
    }
    /// Read its certified source boundary; `row.boundary()` comes from exact wire ancestry.
    pub(in crate::sketch_region_prism) fn boundary(&self) -> RegionSourceId {
        self.boundary
    }
    /// Borrow complete parent dispositions; `row.parents()` keeps duplicate original relations.
    pub(in crate::sketch_region_prism) fn parents(&self) -> &JoinedParentDispositions {
        &self.parents
    }
    /// Read observed and induced directions; `row.orientations()` never samples a normal.
    pub(in crate::sketch_region_prism) fn orientations(&self) -> (Orientation, Orientation) {
        (self.actual, self.expected)
    }
}
