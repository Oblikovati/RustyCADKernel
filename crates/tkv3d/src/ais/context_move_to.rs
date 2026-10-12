//! Point and axis detection preserve filter, highlight and redraw ordering.
use super::{AisInteractiveContext, AisStatusOfDetection};
use crate::v3d::V3dViewSource;
use crate::{select_mgr::*, v3d::V3dView};
use rustycad_tkernel::standard::{Failure, Handle, exceptions::program_error};
use rustycad_tkmath::gp::Axis1;
impl AisInteractiveContext {
    /// Detect a pixel through the context selector; `context.move_to(40,30,&view,true)?`.
    pub fn move_to(
        &self,
        x: i32,
        y: i32,
        view: &Handle<dyn V3dViewSource>,
        redraw: bool,
    ) -> Result<AisStatusOfDetection, Failure> {
        self.validate_detection_view(view)?;
        self.detection_selector().pick_point(x, y, view)?;
        self.move_to_detected(view, redraw)
    }
    /// Detect along a world-space axis; `context.move_to_axis(Axis1::default(),&view,true)?`.
    pub fn move_to_axis(
        &self,
        axis: Axis1,
        view: &Handle<dyn V3dViewSource>,
        redraw: bool,
    ) -> Result<AisStatusOfDetection, Failure> {
        self.validate_detection_view(view)?;
        self.detection_selector().pick_axis(axis, view)?;
        self.move_to_detected(view, redraw)
    }
    fn validate_detection_view(&self, view: &Handle<dyn V3dViewSource>) -> Result<(), Failure> {
        if view.viewer() != *self.current_viewer() {
            return Err(program_error(&format!(
                "MoveTo view {:?}; expected a view belonging to the context viewer",
                view.object_address()
            )));
        }
        Ok(())
    }
    /// Consume current selector picks using native detection policy; `context.move_to_detected(&view,false)?`.
    pub fn move_to_detected(
        &self,
        view: &Handle<dyn V3dViewSource>,
        redraw: bool,
    ) -> Result<AisStatusOfDetection, Failure> {
        self.reset_detected_cursors();
        self.fields.write().last_active_view = view.downgrade();
        let rank = self.collect_detected_ranks()?;
        let (status, update) = if rank >= 1 {
            self.accept_detected_rank(rank, view)?
        } else {
            self.forget_detected_owner(view)?
        };
        if update && redraw {
            redraw_detection_view(view);
        }
        Ok(status)
    }
    fn collect_detected_ranks(&self) -> Result<i32, Failure> {
        let count = self.main_selector().nb_picked();
        let (mut first, mut ignore_top) = (0, false);
        for rank in 1..=count {
            let owner = self.detection_selector().picked(rank)?;
            let filters = self.fields.read().filters.clone();
            if owner.is_null() || !filters.is_ok(&owner)? {
                ignore_top |=
                    self.fields.read().picking_strategy == SelectMgrPickingStrategy::OnlyTopmost;
                continue;
            }
            if first < 1 && !ignore_top {
                first = rank;
            }
            self.fields.write().detected.append(rank);
        }
        Ok(first)
    }
    fn accept_detected_rank(
        &self,
        rank: i32,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(AisStatusOfDetection, bool), Failure> {
        self.fields.write().current_highlighted = 1;
        let owner = self.detection_selector().picked(rank)?;
        let last = self.fields.read().last_picked.clone();
        if owner == last && !owner.is_forced_hilight() {
            return Ok((self.last_detection_status(), false));
        }
        let mut update = self.unhighlight_detected_owner(view)?;
        self.fields.write().last_picked = owner;
        if !self.last_picked_has_selectable() {
            return Ok((AisStatusOfDetection::Nothing, update));
        }
        if self.should_highlight_detected() {
            self.invalidate_slow_highlight(view)?;
            self.highlight_owner_slot(&|| self.fields.read().last_picked.clone(), &view.viewer())?;
            update = true;
        }
        Ok((self.last_detection_status(), update))
    }
    fn should_highlight_detected(&self) -> bool {
        let fields = self.fields.read();
        fields.automatic_hilight
            && (!fields.last_picked.is_selected() || fields.to_hilight_selected)
    }
    fn last_detection_status(&self) -> AisStatusOfDetection {
        if self.fields.read().last_picked.is_selected() {
            AisStatusOfDetection::Selected
        } else {
            AisStatusOfDetection::OnlyOneDetected
        }
    }
    fn forget_detected_owner(
        &self,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(AisStatusOfDetection, bool), Failure> {
        let automatic = self.fields.read().automatic_hilight;
        let update = if automatic {
            self.unhighlight_detected_owner(view)?
        } else {
            false
        };
        self.fields.write().last_picked = Handle::default();
        Ok((AisStatusOfDetection::Nothing, update))
    }
    fn unhighlight_detected_owner(
        &self,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<bool, Failure> {
        if !self.last_picked_has_selectable() {
            return Ok(false);
        }
        self.invalidate_slow_highlight(view)?;
        self.clear_dynamic_highlight();
        Ok(true)
    }
    fn invalidate_slow_highlight(&self, view: &Handle<dyn V3dViewSource>) -> Result<(), Failure> {
        let owner = self.fields.read().last_picked.clone();
        if self.is_slow_hi_style(&owner, &view.viewer())? {
            view.viewer().invalidate();
        }
        Ok(())
    }
}
fn redraw_detection_view(view: &V3dView) {
    if view.computed_mode() {
        view.viewer().update();
    } else if view.is_invalidated() {
        view.viewer().redraw();
    } else {
        view.viewer().redraw_immediate();
    }
}
