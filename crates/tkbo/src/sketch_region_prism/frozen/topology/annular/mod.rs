// SPDX-License-Identifier: GPL-2.0-only
//! ADR0052: parameter-domain Euler policy sealed by the actual named annular embedding join.
mod operation;
mod policy;
mod types;
use super::super::annular_join::{AnnularFrozenDomain, NamedAnnularDomain};
use super::{domain::FaceDomainPolicy, execution::*, model::*, types::*};
use crate::sketch_region_prism::frozen::{
    AuthoredPrismId, FrozenPrismKey, FrozenPrismProjection, FrozenPrismUse, FrozenUseKey,
};
pub(in crate::sketch_region_prism) use operation::certify_annular_frozen_topology;
pub(in crate::sketch_region_prism) use types::*;
