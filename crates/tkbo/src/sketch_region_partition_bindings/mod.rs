// SPDX-License-Identifier: GPL-2.0-only
//! Actual DS incidence is independent of canonical algebraic contact identity.
mod inspect;
mod outcome;
pub(crate) use inspect::inspect_partition_bindings;
pub(crate) use outcome::*;
#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod native_tests;
#[cfg(test)]
mod span_tests;
mod spans;

#[cfg(test)]
mod image_link_tests;
mod image_links;
mod integrity;
#[cfg(test)]
mod integrity_tests;
