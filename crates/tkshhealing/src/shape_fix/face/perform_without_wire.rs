//! Native aggregate stages when FixWireMode is explicitly disabled.
use super::{ShapeExtend, ShapeExtendStatus, ShapeFixFace, ShapeFixFaceWireMap};
use crate::shape_fix::{FixRoot, root::need_fix};
use rustycad_tkbrep::{
    brep_tools::{self, ReShape},
    topexp::Explorer,
    topods::Shape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
impl ShapeFixFace {
    pub(super) fn perform_without_wire(&self) -> Result<bool, Failure> {
        *self.status.write() = 0;
        self.wire_tool.set_context(&self.context());
        let initial = self.face()?;
        if self.face_already_replaced(&initial)? {
            return Ok(false);
        }
        let saved_precision = self.precision();
        *self.result.write() = initial.clone().into_shape();
        self.perform_periodic_stages()?;
        let results = self.result();
        for shape in Explorer::new(&results, ShapeEnum::Face, ShapeEnum::Shape)? {
            *self.face_state.write() = shape;
            self.perform_face_stages()?;
        }
        self.set_precision(saved_precision);
        self.perform_small_area_stages()?;
        self.finish_without_wire(&initial)
    }
    fn face_already_replaced(&self, initial: &Shape) -> Result<bool, Failure> {
        let context_handle = self.context();
        let Some(context) = context_handle.get() else {
            return Ok(false);
        };
        let applied = ReShape::apply(context, initial, ShapeEnum::Shape)?;
        if !applied.is_null() && applied.shape_type()? == ShapeEnum::Face {
            return Ok(false);
        }
        *self.result.write() = applied;
        Ok(true)
    }
    pub(super) fn perform_periodic_stages(&self) -> Result<(), Failure> {
        let modes = *self.modes.read();
        if need_fix(modes.periodic_degenerated, true) {
            self.fix_periodic_degenerated()?;
        }
        if need_fix(modes.missing_seam, true) && self.fix_missing_seam()? {
            self.record_face_stage(ShapeExtendStatus::Done3);
        }
        Ok(())
    }
    pub(super) fn perform_face_stages(&self) -> Result<(), Failure> {
        let modes = *self.modes.read();
        if self.fix_wires_two_coinc_edges()? {
            self.record_face_stage(ShapeExtendStatus::Done7);
        }
        if need_fix(modes.intersecting_wires, true) && self.fix_intersecting_wires()? {
            self.record_face_stage(ShapeExtendStatus::Done6);
        }
        self.perform_orientation_and_bounds()
    }
    fn perform_orientation_and_bounds(&self) -> Result<(), Failure> {
        let modes = *self.modes.read();
        let mut groups = ShapeFixFaceWireMap::default();
        if need_fix(modes.orientation, true) && self.fix_orientation_with_map(&mut groups)? {
            self.record_face_stage(ShapeExtendStatus::Done2);
        }
        brep_tools::update_face(&self.face()?)?;
        // Native invokes this unconditionally; the repair checks its own mode.
        if self.fix_add_natural_bound()? {
            self.record_face_stage(ShapeExtendStatus::Done5);
            return Ok(());
        }
        if need_fix(modes.split_face, true)
            && groups.extent() > 1
            && self.fix_split_face(&groups)?
        {
            self.record_face_stage(ShapeExtendStatus::Done8);
        }
        Ok(())
    }
    pub(super) fn perform_small_area_stages(&self) -> Result<(), Failure> {
        let modes = *self.modes.read();
        let result = self.result();
        for shape in Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape)? {
            *self.face_state.write() = shape;
            if need_fix(modes.small_area_wire, false)
                && self.fix_small_area_wire(need_fix(modes.remove_small_area_face, false))?
            {
                self.record_face_stage(ShapeExtendStatus::Done4);
            }
        }
        Ok(())
    }
    fn finish_without_wire(&self, initial: &Shape) -> Result<bool, Failure> {
        let done = self.status(ShapeExtendStatus::Done);
        if let Some(context) = self.context().get() {
            *self.result.write() = ReShape::apply(context, initial, ShapeEnum::Shape)?;
        } else if !done {
            *self.result.write() = initial.clone();
        }
        Ok(done)
    }
    pub(super) fn record_face_stage(&self, status: ShapeExtendStatus) {
        *self.status.write() |= ShapeExtend::encode_status(status);
    }
}
