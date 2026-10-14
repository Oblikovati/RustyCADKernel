// SPDX-License-Identifier: GPL-2.0-only
//! Preserve the frozen V1 member and original native error attribution through shared raw access.
use super::types::{CaptureResult, PrismFreezeCause};
use crate::sketch_region_geometry_capture::capture_raw_entity_tolerance;
use rustycad_tkbrep::topods::Shape;

pub(super) fn capture_stored_tolerance(actual: &Shape) -> CaptureResult<Option<f64>> {
    capture_raw_entity_tolerance(actual, "Martensite frozen prism raw tolerance")
        .map_err(PrismFreezeCause::Native)
}
