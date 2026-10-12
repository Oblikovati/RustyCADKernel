//! Martensite UI/render queue transfer preserves pending render requests.
use super::view_input_motion::*;
use super::{AisInteractiveContext, AisViewController, AisViewInputBuffer, AisViewSelectionTool};
use crate::v3d::V3dViewSource;
use rustycad_tkernel::standard::Handle;

impl AisViewController {
    /// Transfer queued input without applying camera events; `controller.flush_buffers(&context, &view)`.
    pub fn flush_buffers(
        &mut self,
        _context: &Handle<AisInteractiveContext>,
        _view: &Handle<dyn V3dViewSource>,
    ) {
        self.navigation_state.to_ask_next_frame = false;
        transfer_input_requests(&mut self.ui_input, &mut self.render_input);
    }
}

fn transfer_input_requests(ui: &mut AisViewInputBuffer, render: &mut AisViewInputBuffer) {
    render.is_new_gesture = std::mem::take(&mut ui.is_new_gesture);
    render.zoom_actions.clear(None);
    render.zoom_actions.append_sequence(&mut ui.zoom_actions);
    ui.zoom_actions.clear(None);
    transfer_orientation_highlight(ui, render);
    transfer_selection(ui, render);
    transfer_panning(&mut ui.panning, &mut render.panning);
    transfer_dragging(&mut ui.dragging, &mut render.dragging);
    transfer_orbit_rotation(&mut ui.orbit_rotation, &mut render.orbit_rotation);
    transfer_view_rotation(&mut ui.view_rotation, &mut render.view_rotation);
    if std::mem::take(&mut ui.z_rotate.to_rotate) {
        render.z_rotate = ui.z_rotate;
        render.z_rotate.to_rotate = true;
    }
}

fn transfer_orientation_highlight(ui: &mut AisViewInputBuffer, render: &mut AisViewInputBuffer) {
    render.orientation.to_fit_all = std::mem::take(&mut ui.orientation.to_fit_all);
    if std::mem::take(&mut ui.orientation.to_set_view_orient) {
        render.orientation.to_set_view_orient = true;
        render.orientation.view_orient = ui.orientation.view_orient;
    }
    if std::mem::take(&mut ui.move_to.to_hilight) {
        render.move_to.to_hilight = true;
        render.move_to.point = ui.move_to.point;
    }
}

fn transfer_selection(ui: &mut AisViewInputBuffer, render: &mut AisViewInputBuffer) {
    render.selection.tool = ui.selection.tool;
    render.selection.scheme = ui.selection.scheme;
    // Assignment retains the render allocator, unlike replacing the sequence with a clone.
    render.selection.points.assign(&ui.selection.points);
    if ui.selection.tool == AisViewSelectionTool::Picking {
        ui.selection.points.clear(None);
    }
    if std::mem::take(&mut ui.selection.to_apply_tool) {
        render.selection.to_apply_tool = true;
        ui.selection.points.clear(None);
    }
}

fn transfer_panning(ui: &mut AisViewInputPanning, render: &mut AisViewInputPanning) {
    if std::mem::take(&mut ui.to_start) {
        render.to_start = true;
        render.point_start = ui.point_start;
    }
    if std::mem::take(&mut ui.to_pan) {
        render.to_pan = true;
        render.delta = ui.delta;
    }
}

fn transfer_dragging(ui: &mut AisViewInputDragging, render: &mut AisViewInputDragging) {
    render.point_to = ui.point_to;
    // OCCT 3d097a0 defers lower-priority requests, so abort/stop cannot drain the queue.
    if std::mem::take(&mut ui.to_abort) {
        render.to_abort = true;
        return;
    }
    if std::mem::take(&mut ui.to_stop) {
        render.to_stop = true;
        return;
    }
    if std::mem::take(&mut ui.to_start) {
        render.to_start = true;
        render.point_start = ui.point_start;
    }
    render.to_confirm |= std::mem::take(&mut ui.to_confirm);
    render.to_move |= std::mem::take(&mut ui.to_move);
}

macro_rules! transfer_rotation {
    ($function:ident, $record:ty) => {
        fn $function(ui: &mut $record, render: &mut $record) {
            if std::mem::take(&mut ui.to_start) {
                render.to_start = true;
                render.point_start = ui.point_start;
            }
            if std::mem::take(&mut ui.to_rotate) {
                render.to_rotate = true;
                render.point_to = ui.point_to;
            }
        }
    };
}
transfer_rotation!(transfer_orbit_rotation, AisViewInputOrbitRotation);
transfer_rotation!(transfer_view_rotation, AisViewInputRotation);
