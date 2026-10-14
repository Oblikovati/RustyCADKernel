// SPDX-License-Identifier: GPL-2.0-only
//! Borrowed views retain the whole region proof; no public constructor grants binding authority.
use super::*;
impl AnnularPrismBinding {
    /// Borrow all classed original copy/image parents and measured mapping work; `binding.parents()`.
    pub(in crate::sketch_region_prism) fn parents(&self) -> &AnnularParentCapture {
        &self.operands.parents
    }
    /// Borrow the exact retained region proof; `binding.region()`.
    pub(in crate::sketch_region_prism) fn region(&self) -> &TwoLoopRegionCertificate {
        &self.region
    }
    /// Read full source-keyed copy correspondence; `binding.boundaries()`.
    pub(in crate::sketch_region_prism) fn boundaries(&self) -> &[AnnularBoundaryBinding] {
        &self.operands.boundaries
    }
    /// Read actual four chart operands and all trim contexts; `binding.charts()`.
    pub(in crate::sketch_region_prism) fn charts(&self) -> &[AnnularBoundChart] {
        &self.operands.charts
    }
    /// Retain requested/effective limits without widening; `binding.limits()`.
    pub(in crate::sketch_region_prism) fn limits(
        &self,
    ) -> &super::super::super::frozen::FrozenPrismLimits {
        &self.operands.limits
    }
    /// Read the exact signed affine sweep determinant; `binding.signed_jacobian()`.
    pub(in crate::sketch_region_prism) fn signed_jacobian(
        &self,
    ) -> &rustycad_tkmath::predicates::roots::DyadicBound {
        &self.operands.signed_jacobian
    }
    /// Borrow the full current copied source coefficients; `binding.copied_sources()`.
    pub(in crate::sketch_region_prism) fn copied_sources(
        &self,
    ) -> &super::super::super::surface_charts::CapturedPrismSources {
        &self.operands.copy.copied
    }
    /// Borrow complete identity-free numeric replay; `binding.replay_bytes()`.
    pub(in crate::sketch_region_prism) fn replay_bytes(&self) -> &[u8] {
        &self.replay
    }
}
