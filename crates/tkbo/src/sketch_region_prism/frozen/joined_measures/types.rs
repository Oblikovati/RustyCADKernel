// SPDX-License-Identifier: GPL-2.0-only
use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct JoinedPrismMeasureOptions {
    pub version: u32,
    pub scalars: CompletePrismMeasureOptions,
}
impl JoinedPrismMeasureOptions {
    /// Select owned measurement behavior 1; `JoinedPrismMeasureOptions::v1(scalars)` keeps every supplied limit.
    pub(crate) fn v1(scalars: CompletePrismMeasureOptions) -> Self {
        Self {
            version: 1,
            scalars,
        }
    }
}
pub(crate) struct JoinedPrismMeasures {
    pub(super) requested: JoinedPrismMeasureOptions,
    pub(super) names: [JoinedPrismFace; 3],
    pub(super) measures: CompletePrismMeasures,
    pub(super) binding: Box<[u8]>,
}
/// A borrowed report keeps the source-bound face, all trims and parent multiplicities.
/// Example: `let named = measures.face_area(&authored_use);`.
pub(crate) struct NamedPrismArea<'a> {
    pub face: &'a JoinedPrismFace,
    pub scalar: PrismAreaReport<'a>,
}
pub(crate) enum PrismAreaReport<'a> {
    Plane(&'a CertifiedPolynomialPlaneMeasure),
    Side(&'a CertifiedPolynomialSideIntegral),
}
