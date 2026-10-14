// SPDX-License-Identifier: GPL-2.0-only
//! Owned history operands carry authored keys and complete relation-class multiplicities.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismCandidateCopyLink {
    pub original: PrismHistoryUseKey,
    pub image: PrismHistoricalOccurrence,
    pub kind: u8,
    pub supported: bool,
    pub parents: [Vec<FrozenOriginalKey>; 3],
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismCopiedTreeMember {
    pub occurrence: PrismHistoricalOccurrence,
    pub point_bits: Option<[u64; 3]>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismGeneratedMember {
    pub occurrence: PrismHistoryUseKey,
    pub kind: u8,
    pub point_bits: Option<[u64; 3]>,
    pub descendants: Vec<PrismHistoryUseKey>,
    pub copies: Vec<PrismHistoryMemberKey>,
    pub parents: [Vec<FrozenOriginalKey>; 3],
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PrismGeneratedTrace {
    pub copied: PrismHistoryUseKey,
    pub selected: PrismHistoryUseKey,
    pub complement: PrismHistoryUseKey,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PrismCandidateHistory {
    pub copy_links: Vec<PrismCandidateCopyLink>,
    pub copy_tree: Vec<PrismCopiedTreeMember>,
    pub selected: Vec<PrismGeneratedMember>,
    pub complement: Vec<PrismGeneratedMember>,
    pub directed: Vec<PrismGeneratedTrace>,
    pub omitted_administrative_container: PrismHistoryUseKey,
}
