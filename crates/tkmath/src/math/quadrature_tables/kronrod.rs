#![allow(clippy::excessive_precision)]
const POINT: &[&[f64]] = &[
    include!("kronrod_point_0.rs"),
    include!("kronrod_point_1.rs"),
    include!("kronrod_point_2.rs"),
    include!("kronrod_point_3.rs"),
    include!("kronrod_point_4.rs"),
    include!("kronrod_point_5.rs"),
    include!("kronrod_point_6.rs"),
    include!("kronrod_point_7.rs"),
];
pub(super) fn point(index: usize) -> f64 {
    POINT[index / 256][index % 256]
}
const WEIGHT: &[&[f64]] = &[
    include!("kronrod_weight_0.rs"),
    include!("kronrod_weight_1.rs"),
    include!("kronrod_weight_2.rs"),
    include!("kronrod_weight_3.rs"),
    include!("kronrod_weight_4.rs"),
    include!("kronrod_weight_5.rs"),
    include!("kronrod_weight_6.rs"),
    include!("kronrod_weight_7.rs"),
];
pub(super) fn weight(index: usize) -> f64 {
    WEIGHT[index / 256][index % 256]
}
