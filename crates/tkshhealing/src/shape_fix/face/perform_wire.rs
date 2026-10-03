//! Wire-enabled ShapeFix_Face::Perform orchestration.
use super::{ShapeExtend, ShapeExtendStatus, ShapeFixFace};
use crate::shape_fix::{FixRoot, need_fix};
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{Shape, ShapeIterator},
};
use rustycad_tkernel::{message::ProgressRange, standard::Failure};

impl ShapeFixFace {
    pub(super) fn perform_wire_enabled(
        &self,
        progress: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.reset_wire_perform();
        let initial = self.face_state.read().clone();
        if self.context_replaced_face(&initial)? {
            return Ok(false);
        }
        let (context, saved_precision) = self.perform_inputs();
        let mut reordered_wires = Vec::new();
        let is_replaced =
            self.perform_first_wire_pass(progress, saved_precision, &mut reordered_wires)?;
        self.initialize_wire_result();
        let saved_shape = self.face_state.read().clone();
        self.perform_periodic_stages()?;
        self.perform_remaining_wire_stages(saved_precision, &mut reordered_wires)?;
        self.finish_wire_perform(
            &initial,
            &saved_shape,
            is_replaced,
            &reordered_wires,
            &context,
            saved_precision,
        )
    }

    fn reset_wire_perform(&self) {
        *self.status.write() = 0;
        self.wire_tool.set_context(&self.context());
    }

    fn perform_inputs(
        &self,
    ) -> (
        rustycad_tkernel::standard::Handle<crate::shape_build::ShapeBuildReShape>,
        f64,
    ) {
        let context = self.context();
        let precision = self.precision();
        (context, precision)
    }

    fn context_replaced_face(&self, initial: &Shape) -> Result<bool, Failure> {
        let applied = super::apply_context(initial, &self.context())?;
        if super::is_face(&applied)? {
            return Ok(false);
        }
        *self.result.write() = applied;
        Ok(true)
    }

    fn perform_first_wire_pass(
        &self,
        progress: &ProgressRange<'_>,
        saved_precision: f64,
        reordered_wires: &mut Vec<(Shape, Shape)>,
    ) -> Result<bool, Failure> {
        if !need_fix(self.fix_wire_mode(), true) {
            return Ok(false);
        }
        first_pass::run(self, progress, saved_precision, reordered_wires)
    }

    fn initialize_wire_result(&self) {
        *self.result.write() = self.face_state.read().clone();
    }

    fn perform_remaining_wire_stages(
        &self,
        saved_precision: f64,
        reordered_wires: &mut Vec<(Shape, Shape)>,
    ) -> Result<(), Failure> {
        self.perform_second_wire_pass(reordered_wires)?;
        self.set_precision(saved_precision);
        self.wire_tool.set_precision(saved_precision);
        self.perform_small_area_stages()
    }

    fn finish_wire_perform(
        &self,
        initial: &Shape,
        saved_shape: &Shape,
        is_replaced: bool,
        reordered_wires: &[(Shape, Shape)],
        context: &rustycad_tkernel::standard::Handle<crate::shape_build::ShapeBuildReShape>,
        saved_precision: f64,
    ) -> Result<bool, Failure> {
        self.set_precision(saved_precision);
        self.wire_tool.set_precision(saved_precision);
        if !context.is_null() {
            if self.status(ShapeExtendStatus::Done) && !is_replaced && !initial.is_same(saved_shape)
            {
                replace_reordered_wire_chains(initial, context, reordered_wires)?;
                context.replace(initial, saved_shape)?;
            }
            let applied = super::apply_context(initial, context)?;
            *self.result.write() = applied;
        } else if !self.status(super::ShapeExtendStatus::Done) {
            *self.result.write() = initial.clone();
        }
        Ok(self.status(super::ShapeExtendStatus::Done))
    }

    fn record_wire_face_stage(&self, stage: ShapeExtendStatus) {
        *self.status.write() |= ShapeExtend::encode_status(stage);
    }
}

fn replace_reordered_wire_chains(
    initial: &Shape,
    context: &rustycad_tkernel::standard::Handle<crate::shape_build::ShapeBuildReShape>,
    reordered_wires: &[(Shape, Shape)],
) -> Result<(), Failure> {
    for original in ShapeIterator::new(initial, false, true) {
        replace_reordered_chain(context, original, reordered_wires)?;
    }
    Ok(())
}

fn replace_reordered_chain(
    context: &rustycad_tkernel::standard::Handle<crate::shape_build::ShapeBuildReShape>,
    mut current: Shape,
    reordered_wires: &[(Shape, Shape)],
) -> Result<(), Failure> {
    while let Some((_, replacement)) = reordered_wires
        .iter()
        .find(|(source, _)| source.is_same(&current))
    {
        let next = replacement.clone();
        context.replace(&current, &next)?;
        current = next;
    }
    Ok(())
}

mod first_pass;
mod second_pass;
mod split_wire;
