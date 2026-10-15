// SPDX-License-Identifier: GPL-2.0-only
//! Certified sketch line incidences (ADR 0060) against native OCCT, the mapped
//! Rust intersector and splines built by the owned sketch constructor.
mod corpus;
mod native;
mod mapped_offsets;
mod native_curves;
mod native_offsets;
mod owned_splines;
