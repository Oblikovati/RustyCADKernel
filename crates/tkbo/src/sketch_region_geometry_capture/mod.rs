// SPDX-License-Identifier: GPL-2.0-only
//! Shared owned analytic snapshots; copying a Shape is not an isolation certificate.
mod cause;
mod curves;
mod placement;
pub(crate) use cause::CapturedGeometryCause;
pub(crate) use curves::{CapturedMetricCurve, capture_plane_curve, capture_space_curve};
pub(crate) use placement::{captured_placement, captured_plane_frame};

mod tolerances;
pub(crate) use tolerances::capture_raw_entity_tolerance;
