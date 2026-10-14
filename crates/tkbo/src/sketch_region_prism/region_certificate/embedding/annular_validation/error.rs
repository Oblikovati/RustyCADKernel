// SPDX-License-Identifier: GPL-2.0-only
use super::*;
#[derive(Debug)]
pub(in crate::sketch_region_prism) enum AnnularTableFailure {
    Cancelled {
        completed: usize,
    },
    Reconstruction(Box<RegionHypothesisCause>),
    DomainMultiplicity {
        key: Box<AnnularChartIdentity>,
        actual: usize,
        expected: usize,
    },
    DomainCorrespondence {
        actual: Box<AnnularFaceDomain>,
        expected: Box<AnnularFaceDomain>,
    },
    Coverage(AnnularPairCoverageFailure<AnnularChartIdentity>),
    Correspondence {
        actual: Box<AnnularFaceIncidence>,
        expected: Box<AnnularFaceIncidence>,
    },
}
