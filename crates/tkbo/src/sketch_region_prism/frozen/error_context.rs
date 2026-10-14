// SPDX-License-Identifier: GPL-2.0-only
//! Known authored entities accompany their unchanged typed cause; unbound failures stay unbound.
use super::{bindings::BoundProjection, types::*};
impl PrismFreezeCause {
    pub(super) fn at_occurrence(self, bound: &BoundProjection<'_>, index: usize) -> Self {
        Self::Entities {
            primary: bound.names[index],
            related: Vec::new(),
            occurrence: Some(bound.use_names[index].clone()),
            cause: Box::new(self),
        }
    }
    pub(super) fn at_endpoint(self, edge: FrozenPrismKey, vertex: FrozenPrismKey) -> Self {
        Self::Entities {
            primary: edge,
            related: vec![vertex],
            occurrence: None,
            cause: Box::new(self),
        }
    }
    pub(super) fn primary_entity(&self) -> Option<FrozenPrismKey> {
        match self {
            Self::Entities { primary, .. } => Some(*primary),
            _ => None,
        }
    }
    pub(super) fn native_failure(&self) -> bool {
        match self {
            Self::Native(_) => true,
            Self::Entities { cause, .. } => cause.native_failure(),
            _ => false,
        }
    }
}
