// SPDX-License-Identifier: GPL-2.0-only
//! ADR0047 read adapters; semantic correspondence and complete-body projection remain prerequisites.
mod geometry;
pub(crate) use geometry::{PrismBodySpaceCurveView, PrismBodySurfaceView};
#[cfg(test)]
mod geometry_tests;
