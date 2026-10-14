// SPDX-License-Identifier: GPL-2.0-only
//! Private ADR0014 paired-contact construction bridge; mapped observations never certify search.
mod types;
pub(crate) use types::*;

mod exact_sources;
mod generated;
#[cfg(test)]
mod tests;
mod topology;
mod transaction;
#[allow(unused_imports)]
// ADR0014 keeps this facade private until whole-operation consumers pass.
pub(crate) use transaction::transport_partition_contacts;

#[cfg(test)]
mod native_tests;
mod world;

mod progress;

mod source_cache;

mod world_points;

mod admission;

#[cfg(test)]
mod general_tests;

#[cfg(test)]
mod mutation_tests;

#[cfg(test)]
mod retained_transaction_tests;

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod declared_plane_tests;

#[cfg(test)]
pub(crate) mod test_fixture;

#[cfg(test)]
mod revision_tests;

pub(crate) use admission::BoundedContactConstruction;
pub(crate) use generated::observe_generated_vertex;

pub(crate) use world::recertify_contact_point;
