//! Cycling is independent from detection iteration and automatic-highlight flags.
use super::AisInteractiveContext;
use crate::{select_mgr::SelectMgrViewerSelectorResultMembers, v3d::V3dView};
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    /// Highlight the next detected owner, wrapping at the end; `context.hilight_next_detected(&view,true)?`.
    pub fn hilight_next_detected(
        &self,
        view: &Handle<V3dView>,
        redraw: bool,
    ) -> Result<i32, Failure> {
        self.cycle_detected_owner(view, redraw, 1)
    }
    /// Highlight the previous detected owner, wrapping at the start; `context.hilight_previous_detected(&view,true)?`.
    pub fn hilight_previous_detected(
        &self,
        view: &Handle<V3dView>,
        redraw: bool,
    ) -> Result<i32, Failure> {
        self.cycle_detected_owner(view, redraw, -1)
    }
    fn cycle_detected_owner(
        &self,
        view: &Handle<V3dView>,
        redraw: bool,
        step: i32,
    ) -> Result<i32, Failure> {
        self.detection_presentations().clear_immediate_draw();
        let Some(rank) = self.cycle_detected_rank(step)? else {
            return Ok(0);
        };
        let owner = self.detection_selector().picked(rank)?;
        if owner.is_null() {
            return Ok(0);
        }
        self.highlight_with_color(&owner, &view.viewer())?;
        self.fields.write().last_picked = owner;
        if redraw {
            self.detection_presentations()
                .redraw_immediate(&view.viewer())?;
            self.detection_viewer().redraw_immediate();
        }
        Ok(self.fields.read().current_highlighted)
    }
    fn cycle_detected_rank(&self, step: i32) -> Result<Option<i32>, Failure> {
        let mut fields = self.fields.write();
        if fields.detected.is_empty() {
            return Ok(None);
        }
        fields.current_highlighted += step;
        if step > 0 && fields.current_highlighted > fields.detected.upper() as i32 {
            fields.current_highlighted = 1;
        }
        if step < 0 && fields.current_highlighted < 1 {
            fields.current_highlighted = fields.detected.upper() as i32;
        }
        Ok(Some(
            *fields.detected.value(fields.current_highlighted as usize)?,
        ))
    }
}
