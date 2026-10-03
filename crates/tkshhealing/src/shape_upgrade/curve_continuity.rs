//! Policies shared by the 2D and 3D continuity splitters.
use super::SplitCurveData;
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkernel::ncollection::HSequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::geom_abs::Shape;

#[derive(Clone, Copy)]
pub(super) struct ContinuitySettings {
    pub(super) criterion: Shape,
    pub(super) tolerance: f64,
    pub(super) order: i32,
}

impl ContinuitySettings {
    pub(super) fn with_tolerance(tolerance: f64) -> Self {
        Self {
            criterion: Shape::C1,
            tolerance,
            order: 1,
        }
    }

    pub(super) fn set_criterion(&mut self, criterion: Shape) {
        self.criterion = criterion;
        self.order = continuity_order(criterion);
    }
}

pub(super) fn continuity_order(criterion: Shape) -> i32 {
    match criterion {
        Shape::C0 => 0,
        Shape::C2 => 2,
        Shape::C3 => 3,
        Shape::CN => 4,
        _ => 1,
    }
}

pub(super) fn offset_criterion(criterion: Shape) -> Shape {
    match criterion {
        Shape::C2 => Shape::C3,
        Shape::C3 | Shape::CN => Shape::CN,
        _ => Shape::C2,
    }
}

pub(super) fn replace_status(data: &SplitCurveData, status: ShapeExtendStatus) {
    data.with_mut(|fields| fields.status = ShapeExtend::encode_status(status));
}

pub(super) fn add_status(data: &SplitCurveData, status: ShapeExtendStatus) {
    data.with_mut(|fields| fields.status |= ShapeExtend::encode_status(status));
}

pub(super) fn copy_nested_results(
    source_data: &SplitCurveData,
    source_values: &Handle<HSequence<f64>>,
    target_data: &SplitCurveData,
    target_values: &Handle<HSequence<f64>>,
) -> Result<(), Failure> {
    let sequence = source_values.sequence(Clone::clone);
    target_values.change_sequence(|values| *values = sequence);
    let status = source_data.with(|fields| fields.status);
    target_data.with_mut(|fields| fields.status |= status);
    Ok(())
}

pub(super) struct KnotRemoval {
    pub(super) degree: i32,
    pub(super) knot: i32,
    pub(super) multiplicity: i32,
    pub(super) tolerance: f64,
    pub(super) order: i32,
}

pub(super) fn remove_knot_and_recheck(
    policy: KnotRemoval,
    remove: impl FnOnce(i32, i32, f64) -> Result<bool, Failure>,
    current_multiplicity: impl FnOnce(i32) -> Result<i32, Failure>,
) -> Result<bool, Failure> {
    let corrected = remove(policy.knot, policy.multiplicity, policy.tolerance).unwrap_or(false);
    if corrected && policy.multiplicity > 0 {
        return Ok(policy.degree - current_multiplicity(policy.knot)? >= policy.order);
    }
    Ok(corrected)
}
