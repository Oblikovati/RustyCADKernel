// SPDX-License-Identifier: GPL-2.0-only
//! Caller IDs occupy typed namespaces, including zero and every possible internal token value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CallerOperationId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CallerBoundaryId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CallerVertexId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CallerDirectedUseId(pub u64);
/// Source-owned identities only; example: `identity.operation = CallerOperationId(42)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SingleBoundaryIdentities {
    pub operation: CallerOperationId,
    pub boundary: CallerBoundaryId,
    pub closure: CallerVertexId,
    pub start: CallerDirectedUseId,
    pub end: CallerDirectedUseId,
}
/// Selection is semantic intent, never an index into returned candidates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SemanticSelectionIntent {
    BoundedRegionContaining { seed: [f64; 2] },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SemanticNameFailure {
    EqualEndpointUses { actual: CallerDirectedUseId },
    NonfiniteSeed { axis: usize, actual: f64 },
    UnknownToken { actual: u64, version: u32 },
}
/// Anchors retain exact caller values without treating them as private symbol numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemanticCallerAnchor {
    Operation(CallerOperationId),
    Boundary(CallerBoundaryId),
    Closure(CallerVertexId),
    Start(CallerDirectedUseId),
    End(CallerDirectedUseId),
}
