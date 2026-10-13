// SPDX-License-Identifier: GPL-2.0-only
//! Keep scale separate from the actual unscaled gp_Trsf matrix; no rounded matrix product.
use super::tolerance_types::PlanarMetricCause;
use rustycad_tkmath::{
    gp::{Axis3, Transform3},
    predicates::roots::metric_bounds::MetricEmbedding,
};
pub(super) fn captured_placement(
    transform: Transform3,
) -> Result<MetricEmbedding<3>, PlanarMetricCause> {
    let mut linear = [[0.; 3]; 3];
    for (row, coefficients) in linear.iter_mut().enumerate() {
        for (column, value) in coefficients.iter_mut().enumerate() {
            *value = transform
                .matrix()
                .value(row as i32 + 1, column as i32 + 1)?;
        }
    }
    Ok(MetricEmbedding {
        linear,
        origin: *transform.translation_part().coordinates(),
        scale: transform.scale_factor(),
    })
}
pub(super) fn captured_plane_frame(frame: Axis3) -> MetricEmbedding<2> {
    let x = frame.x_direction();
    let y = frame.y_direction();
    let origin = frame.location();
    MetricEmbedding {
        linear: [[x.x(), y.x()], [x.y(), y.y()], [x.z(), y.z()]],
        origin: [origin.x(), origin.y(), origin.z()],
        scale: 1.,
    }
}
