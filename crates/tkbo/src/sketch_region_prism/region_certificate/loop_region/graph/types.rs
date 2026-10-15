// SPDX-License-Identifier: GPL-2.0-only
//! ADR 0084 request and certificate of the plane graph region proof.
use super::super::types::{AuthoredLoopUse, LoopRegionOptions};
use crate::sketch_region::ArrangementSource;
use crate::sketch_region_contact_transport::PairedContactTransaction;
use rustycad_tkmath::predicates::roots::metric_bounds::CertifiedRationalMeasure;

/// Borrowed inputs: the actual contact transaction and the caller vertex ids at native
/// parameters 0 and 1 of every source, in source order.
pub(crate) struct GraphRegionRequest<'a, 'b> {
    pub bridge: &'a PairedContactTransaction,
    pub sources: &'b [ArrangementSource<'a>],
    pub ends: &'b [[u64; 2]],
    pub options: LoopRegionOptions,
}

/// One certified bounded face: a simple counterclockwise orbit (the face on its left) and
/// the components whose outer walks are its holes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphFaceProof {
    /// Directed source uses of the outer boundary, counterclockwise.
    pub uses: Vec<AuthoredLoopUse>,
    /// The connected component of the boundary.
    pub component: usize,
    /// Components nested directly in this face; their outer walks are the holes.
    pub holes: Vec<usize>,
    /// Certified chart area: outer area minus hole areas.
    pub area: CertifiedRationalMeasure,
}

/// One connected component of the plane graph.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphComponentProof {
    /// Its outer walk, clockwise (the unbounded side on its left).
    pub outer: Vec<AuthoredLoopUse>,
    /// The bounded face (index into `faces`) it lies in directly, if any.
    pub container: Option<usize>,
}

/// Complete plane graph certificate; it publishes no body, history or names.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GraphRegionCertificate {
    /// Every bounded face, ordered by its first dart in source order.
    pub faces: Vec<GraphFaceProof>,
    /// Every connected component, ordered by its first source.
    pub components: Vec<GraphComponentProof>,
    /// Certified vertex incidence contacts.
    pub contacts: usize,
    /// Pair domains certified by the ADR 0066 part B common-carrier route.
    pub carrier_pairs: usize,
    /// Pair domains certified in total.
    pub pair_domains: usize,
}
