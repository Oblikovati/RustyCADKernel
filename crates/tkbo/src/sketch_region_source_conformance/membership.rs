// SPDX-License-Identifier: GPL-2.0-only
//! Exact native-bracket membership is not algebraic contact identity or a snap operation.
use super::types::NativeParameterMembership;
use rustycad_tkmath::predicates::roots::{
    DyadicBound,
    metric_bounds::{MetricParameterMap, ResidualError},
};
/// Compare the actual mapped parameter to an exact affine bracket without division.
/// Example: `native_parameter_membership(map, &root_bounds, native_parameter)`.
pub(crate) fn native_parameter_membership(
    map: MetricParameterMap,
    bounds: &[DyadicBound; 2],
    parameter: f64,
) -> Result<NativeParameterMembership, ResidualError> {
    let native_bracket = map.image_bounds(bounds)?;
    let supplied_parameter =
        DyadicBound::from_f64(parameter).map_err(|_| ResidualError::Value {
            role: "native parameter",
            index: 0,
            value: parameter,
        })?;
    let belongs =
        native_bracket[0] <= supplied_parameter && supplied_parameter <= native_bracket[1];
    let singleton_equal = native_bracket[0] == native_bracket[1] && belongs;
    Ok(NativeParameterMembership {
        native_bracket,
        supplied_parameter,
        belongs,
        singleton_equal,
    })
}
