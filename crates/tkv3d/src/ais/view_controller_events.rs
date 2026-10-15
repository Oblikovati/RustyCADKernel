//! `AIS_ViewController` event dispatch: the per-frame handling of the queued input (subview
//! picking, view orientation keys, navigation, highlighting, selection, dragging, camera
//! actions, XR and redraw).
use super::view_controller_lazy_move::camera_action_view;
use super::*;
use crate::prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers};
use crate::select_mgr::*;
use crate::v3d::{V3dTypeOfOrientation, V3dViewSource};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::{Array1, Vec2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Point2;
use rustycad_tkservice::aspect::{AspectDisplayConnection, AspectVKey, AspectVKeyBasic};
use rustycad_tkservice::graphic3d::{Graphic3dCViewSubviews, Graphic3dCViewXr, Graphic3dCamera};

/// `THE_VIEW_KEYS`: the view keys and their orientation (none for roll and fit-all keys).
const VIEW_KEYS: [(AspectVKey, Option<V3dTypeOfOrientation>); 11] = [
    (
        AspectVKeyBasic::ViewTop as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_TOP),
    ),
    (
        AspectVKeyBasic::ViewBottom as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_BOTTOM),
    ),
    (
        AspectVKeyBasic::ViewLeft as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_LEFT),
    ),
    (
        AspectVKeyBasic::ViewRight as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_RIGHT),
    ),
    (
        AspectVKeyBasic::ViewFront as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_FRONT),
    ),
    (
        AspectVKeyBasic::ViewBack as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_BACK),
    ),
    (
        AspectVKeyBasic::ViewAxoLeftProj as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_AXO_LEFT),
    ),
    (
        AspectVKeyBasic::ViewAxoRightProj as AspectVKey,
        Some(V3dTypeOfOrientation::ZUP_AXO_RIGHT),
    ),
    (AspectVKeyBasic::ViewRoll90CW as AspectVKey, None),
    (AspectVKeyBasic::ViewRoll90CCW as AspectVKey, None),
    (AspectVKeyBasic::ViewFitAll as AspectVKey, None),
];

impl AisViewController {
    /// `HandleViewEvents`: process the rendering-thread queue on a view (immediate update
    /// suspended): subview picking, orientation keys, navigation, XR input, highlighting and
    /// selection, camera actions, XR presentations and redraw, then the queue is reset.
    /// Example: `controller.handle_view_events(&context, &view)?`.
    pub fn handle_view_events(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        let was_immediate_update = actual.set_immediate_update(false);
        let mut picked_view: Handle<dyn V3dViewSource> = Handle::default();
        if actual.is_subview() || !actual.subviews().is_empty() {
            let selection = &self.render_input.selection;
            if selection.tool == AisViewSelectionTool::Picking && !selection.points.is_empty() {
                let mut click = *selection.points.last().expect("nonempty points");
                if actual.is_subview() {
                    let corner = actual.view().subview_top_left();
                    click = Vec2::new(click.x + corner.x, click.y + corner.y);
                }
                let parent = if actual.is_subview() {
                    actual.parent_view()
                } else {
                    view.clone()
                };
                picked_view = camera_action_view(&parent)?.pick_subview(click);
            }
        }
        self.handle_view_orientation_keys(context, view)?;
        let walk = self.handle_navigation_keys(context, view)?;
        self.handle_xr_input(context, view, &walk)?;
        if actual.view().is_active_xr() {
            actual.view().setup_xr_posed_camera();
        }
        self.handle_move_to(context, view)?;
        self.handle_camera_actions(context, view, &walk)?;
        // Camera actions may move the posed camera: carry it to the base camera.
        actual.view().synchronize_xr_posed_to_base_camera()?;
        self.handle_xr_presentations(context, view)?;
        self.handle_view_redraw(context, view)?;
        actual.view().unset_xr_posed_camera()?;
        actual.set_immediate_update(was_immediate_update);
        if !picked_view.is_null() && picked_view != *view {
            self.on_subview_changed(context, view, &picked_view)?;
        }
        self.render_input.reset();
        self.navigation_state.to_ask_next_frame = false;
        Ok(())
    }

    /// `FlushViewEvents`: move the UI queue to the rendering queue (with the touch gestures),
    /// shift its pixels into a subview and optionally handle it.
    /// Example: `controller.flush_view_events(&context, &view, true)?`.
    pub fn flush_view_events(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
        to_handle: bool,
    ) -> Result<(), Failure> {
        self.flush_buffers(context, view);
        self.flush_gestures(context, view)?;
        let actual = camera_action_view(view)?;
        if actual.is_subview() {
            let delta = actual.view().subview_top_left();
            let shift = |p: &mut Vec2<i32>| *p = Vec2::new(p.x - delta.x, p.y - delta.y);
            let shift_f = |p: &mut Vec2<f64>| {
                *p = Vec2::new(p.x - f64::from(delta.x), p.y - f64::from(delta.y));
            };
            let input = &mut self.render_input;
            if input.move_to.to_hilight || input.dragging.to_start {
                shift(&mut input.move_to.point);
            }
            if input.panning.to_start {
                shift(&mut input.panning.point_start);
            }
            if input.dragging.to_start {
                shift(&mut input.dragging.point_start);
            }
            if input.dragging.to_move {
                shift(&mut input.dragging.point_to);
            }
            if input.orbit_rotation.to_start {
                shift_f(&mut input.orbit_rotation.point_start);
            }
            if input.orbit_rotation.to_rotate {
                shift_f(&mut input.orbit_rotation.point_to);
            }
            if input.view_rotation.to_start {
                shift_f(&mut input.view_rotation.point_start);
            }
            if input.view_rotation.to_rotate {
                shift_f(&mut input.view_rotation.point_to);
            }
            for point in input.selection.points.iter_mut() {
                shift(point);
            }
            for zoom in input.zoom_actions.iter_mut() {
                shift(&mut zoom.point);
            }
        }
        if to_handle {
            self.handle_view_events(context, view)?;
        }
        Ok(())
    }

    /// `handleMoveTo`: point selection, dynamic highlighting and dragging, then rubber-band
    /// and polygon selection; `controller.handle_move_to(&context, &view)?`.
    pub fn handle_move_to(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        self.handle_selection_pick(context, view)?;
        self.handle_dynamic_highlight(context, view)?;
        self.handle_selection_poly(context, view)
    }

    /// `handleSelectionPick`: each queued pick point detects and selects with the queued
    /// scheme (the previous highlighting pixel kept only when it was defined).
    /// Example: `controller.handle_selection_pick(&context, &view)?`.
    pub fn handle_selection_pick(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let selection = &self.render_input.selection;
        if selection.tool != AisViewSelectionTool::Picking || selection.points.is_empty() {
            return Ok(());
        }
        let points: Vec<Vec2<i32>> = selection.points.iter().copied().collect();
        for point in points {
            let had_previous_move_to = self.has_previous_move_to();
            self.context_lazy_move_to(context, view, &point)?;
            if !had_previous_move_to {
                self.reset_previous_move_to();
            }
            context.select_detected(self.render_input.selection.scheme)?;
            camera_action_view(view)?.viewer().invalidate();
            self.on_selection_changed(context, view)?;
        }
        self.render_input.selection.points.clear(None);
        Ok(())
    }

    /// `handleDynamicHighlight`: highlight under the queued cursor (except in walk mode),
    /// start dragging the detected object, then forward abort/stop/move to the dragged
    /// object (cancelling rotations, panning and zooming while it moves).
    /// Example: `controller.handle_dynamic_highlight(&context, &view)?`.
    pub fn handle_dynamic_highlight(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let allow_highlight = self.parameters.to_allow_highlight;
        if (self.render_input.move_to.to_hilight || self.render_input.dragging.to_start)
            && self.navigation_mode != AisNavigationMode::FirstPersonWalk
        {
            let input = &self.render_input;
            let point = if input.move_to.to_hilight {
                input.move_to.point
            } else {
                input.dragging.point_start
            };
            if input.dragging.to_start
                && (!input.move_to.to_hilight || !allow_highlight)
                && !self.has_previous_move_to()
            {
                self.context_lazy_move_to(context, view, &point)?;
                self.reset_previous_move_to();
                self.on_object_dragged(context, view, AisDragAction::Start)?;
                context.clear_detected(false);
            } else if allow_highlight {
                let actual = camera_action_view(view)?;
                if self.previous_move_to != point
                    || (!actual.view().is_active_xr()
                        && (input.orbit_rotation.to_rotate
                            || input.view_rotation.to_rotate
                            || actual.is_invalidated()))
                {
                    self.reset_previous_move_to();
                    self.context_lazy_move_to(context, view, &point)?;
                }
                if self.render_input.dragging.to_start {
                    self.on_object_dragged(context, view, AisDragAction::Start)?;
                }
            }
            self.render_input.move_to.to_hilight = false;
        }
        if self.dispatch.drag_object.is_null() {
            return Ok(());
        }
        let dragging = &self.render_input.dragging;
        if dragging.to_abort {
            self.on_object_dragged(context, view, AisDragAction::Abort)?;
            self.render_input.orbit_rotation.to_rotate = false;
            self.render_input.view_rotation.to_rotate = false;
        } else if dragging.to_stop {
            self.on_object_dragged(context, view, AisDragAction::Stop)?;
            self.render_input.orbit_rotation.to_rotate = false;
            self.render_input.view_rotation.to_rotate = false;
        } else if dragging.to_move {
            if dragging.to_confirm {
                self.on_object_dragged(context, view, AisDragAction::Confirmed)?;
            }
            self.on_object_dragged(context, view, AisDragAction::Update)?;
            let input = &mut self.render_input;
            input.orbit_rotation.to_rotate = false;
            input.view_rotation.to_rotate = false;
            input.panning.to_pan = false;
            input.zoom_actions.clear(None);
        }
        Ok(())
    }

    /// `handleSelectionPoly`: show the queued rubber band or polygon (hiding an emptied one),
    /// then on apply select inside it (or fit the view to a zoom window) and clear it.
    /// Example: `controller.handle_selection_poly(&context, &view)?`.
    pub fn handle_selection_poly(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let actual = camera_action_view(view)?;
        let band = self.dispatch.rubber_band.clone();
        let band_object: Handle<dyn AisInteractiveObject> = band.clone().into_cast();
        let tool = self.render_input.selection.tool;
        if tool != AisViewSelectionTool::Picking {
            let points: Vec<Vec2<i32>> =
                self.render_input.selection.points.iter().copied().collect();
            if let (Some(first), Some(last)) = (points.first(), points.last()) {
                band.clear_points();
                band.set_to_update_all();
                let is_rubber = tool == AisViewSelectionTool::RubberBand
                    || tool == AisViewSelectionTool::ZoomWindow;
                if is_rubber {
                    band.set_rectangle(first.x, -first.y, last.x, -last.y);
                } else {
                    let mut previous = Vec2::new(i32::MAX, i32::MAX);
                    for point in &points {
                        let flipped = Vec2::new(point.x, -point.y);
                        if flipped != previous {
                            previous = flipped;
                            band.add_point(flipped);
                        }
                    }
                }
                band.set_polygon_closed(is_rubber);
                if let Err(failure) = context.display_with_modes(
                    &band_object,
                    0,
                    -1,
                    false,
                    PrsMgrDisplayStatus::Displayed,
                ) {
                    Message::send_warning_text(&AsciiString::from(
                        format!(
                            "Internal error while displaying rubber-band: {}, {}",
                            failure.exception_type(),
                            String::from_utf8_lossy(failure.what())
                        )
                        .as_str(),
                    ));
                    band.clear_points();
                }
                let layer = actual.viewer().z_layer_settings(band.z_layer())?;
                if layer.is_immediate() {
                    actual.invalidate_immediate();
                } else {
                    actual.invalidate();
                }
            } else if band.has_interactive_context() {
                context.remove(&band_object, false)?;
                band.clear_points();
            }
        }
        if !self.render_input.selection.to_apply_tool {
            return Ok(());
        }
        self.render_input.selection.to_apply_tool = false;
        if !context.is_displayed(&band_object) {
            return Ok(());
        }
        context.remove(&band_object, false)?;
        let points: Vec<Vec2<i32>> = band.points().read().iter().copied().collect();
        let scheme = self.render_input.selection.scheme;
        if points.len() == 4
            && points[0].x == points[1].x
            && points[2].x == points[3].x
            && points[0].y == points[3].y
            && points[1].y == points[2].y
        {
            let (p1, p2) = (
                Vec2::new(points[0].x, -points[0].y),
                Vec2::new(points[2].x, -points[2].y),
            );
            if tool == AisViewSelectionTool::ZoomWindow {
                actual.window_fit_all(p1.x, p1.y, p2.x, p2.y)?;
                actual.invalidate();
            } else {
                let selector = context.main_selector();
                selector.allow_overlap_detection(p1.y != p1.y.min(p2.y));
                context.select_rectangle(
                    Vec2::new(p1.x.min(p2.x), p1.y.min(p2.y)),
                    Vec2::new(p1.x.max(p2.x), p1.y.max(p2.y)),
                    view,
                    scheme,
                )?;
                context.main_selector().allow_overlap_detection(false);
            }
        } else if points.len() >= 3 {
            let polyline: Vec<Point2> = points
                .iter()
                .map(|p| Point2::new(f64::from(p.x), -f64::from(p.y)))
                .collect();
            context.select_polygon(&Array1::from_shared(&polyline, 1), view, scheme)?;
            context.main_selector().allow_overlap_detection(false);
        }
        band.clear_points();
        if tool != AisViewSelectionTool::ZoomWindow {
            actual.viewer().invalidate();
            self.on_selection_changed(context, view)?;
        }
        Ok(())
    }

    /// `handleViewRedraw`: advance the running view and object animations (asking for the
    /// next frame), redraw the invalidated subviews then views, and ask the window for a
    /// new frame when needed; `controller.handle_view_redraw(&context, &view)?`.
    pub fn handle_view_redraw(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let _ = context;
        let actual = camera_action_view(view)?;
        let parent_view = if actual.is_subview() {
            actual.parent_view()
        } else {
            view.clone()
        };
        let view_animation = self.view_animation().clone();
        if !view_animation.is_stopped() {
            view_animation.update_timer()?;
            self.reset_previous_move_to();
            self.set_ask_next_frame(true);
        }
        let objects_animation = self.objects_animation().clone();
        if let Some(animation) = objects_animation.get()
            && !animation.is_stopped()
        {
            animation.update_timer()?;
            self.reset_previous_move_to();
            self.set_ask_next_frame(true);
        }
        if self.parameters.is_continuous_redraw || actual.view().is_active_xr() {
            self.navigation_state.to_ask_next_frame = true;
        }
        let viewer = actual.viewer();
        for sub_view_pass in [true, false] {
            for other in viewer.active_views() {
                let Some(other_view) = other.get() else {
                    continue;
                };
                if sub_view_pass && !other_view.is_subview() {
                    for sub in other_view.subviews() {
                        let Some(sub_view) = sub.get() else {
                            continue;
                        };
                        if sub_view.viewer() == viewer {
                            continue;
                        }
                        if sub_view.is_invalidated() {
                            if sub_view.computed_mode() {
                                sub_view.update()?;
                            } else {
                                sub_view.redraw();
                            }
                        } else if sub_view.is_invalidated_immediate() {
                            sub_view.redraw_immediate();
                        }
                    }
                    continue;
                } else if !sub_view_pass && other_view.is_subview() {
                    continue;
                }
                if other_view.is_invalidated()
                    || (self.navigation_state.to_ask_next_frame && other == *view)
                {
                    if other_view.computed_mode() {
                        other_view.update()?;
                    } else {
                        other_view.redraw();
                    }
                    if other_view.is_subview() {
                        camera_action_view(&other_view.parent_view())?.invalidate_immediate();
                    }
                } else if other_view.is_invalidated_immediate() {
                    if other_view.is_subview() {
                        camera_action_view(&other_view.parent_view())?.invalidate_immediate();
                    }
                    other_view.redraw_immediate();
                }
            }
        }
        let parent = camera_action_view(&parent_view)?;
        if actual.is_subview() && viewer != parent.viewer() {
            parent.redraw_immediate();
        }
        if self.navigation_state.to_ask_next_frame
            && let Some(window) = parent.window().get()
        {
            window.invalidate_content(&Handle::<AspectDisplayConnection>::default());
        }
        Ok(())
    }

    /// `UpdateViewOrientation`: queue a standard orientation (and fit-all) for the next frame;
    /// `controller.update_view_orientation(V3dTypeOfOrientation::ZUP_TOP, true)`.
    pub fn update_view_orientation(&mut self, orientation: V3dTypeOfOrientation, to_fit_all: bool) {
        let request = &mut self.ui_input.orientation;
        request.to_fit_all = to_fit_all;
        request.to_set_view_orient = true;
        request.view_orient = orientation;
    }

    /// `handleViewOrientationKeys`: the pressed view keys (released here) set a standard
    /// orientation with fit-all, roll by a quarter turn or fit all; the change is then
    /// animated from the previous camera (not in walk mode).
    /// Example: `controller.handle_view_orientation_keys(&context, &view)?`.
    pub fn handle_view_orientation_keys(
        &mut self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        if self.navigation_mode == AisNavigationMode::FirstPersonWalk {
            return Ok(());
        }
        let actual = camera_action_view(view)?;
        let mut camera_back: Handle<Graphic3dCamera> = Handle::default();
        let event_time = self.event_time();
        for (key, orientation) in VIEW_KEYS {
            if !self.input_base.keys.is_key_down(key)? {
                continue;
            }
            self.input_base.keys.key_up(key, event_time)?;
            if camera_back.is_null() {
                camera_back = actual.camera();
                let copy = Graphic3dCamera::from_camera(camera_back.get().ok_or_else(|| {
                    Failure::with_message("Standard_NullObject", "V3d_View: null camera")
                })?);
                actual.set_camera(&Handle::new(copy))?;
            }
            if let Some(orientation) = orientation {
                actual.set_proj_orientation(orientation, false)?;
                self.fit_all_auto(context, actual)?;
            } else if key == AspectVKeyBasic::ViewRoll90CW as AspectVKey {
                actual.set_twist(actual.twist() + std::f64::consts::FRAC_PI_2)?;
            } else if key == AspectVKeyBasic::ViewRoll90CCW as AspectVKey {
                actual.set_twist(actual.twist() - std::f64::consts::FRAC_PI_2)?;
            } else if key == AspectVKeyBasic::ViewFitAll as AspectVKey {
                self.fit_all_auto(context, actual)?;
            }
        }
        let Some(back) = camera_back.get() else {
            return Ok(());
        };
        let camera_new = actual.camera();
        actual.set_camera(&camera_back)?;
        let new = camera_new
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "V3d_View: null camera"))?;
        if back.orientation_matrix() != new.orientation_matrix() {
            let animation = self.view_animation().clone();
            animation.set_view(view);
            animation.set_start_pts(0.0);
            animation.set_camera_start(&Handle::new(Graphic3dCamera::from_camera(back)));
            animation.set_camera_end(&Handle::new(Graphic3dCamera::from_camera(new)));
            animation.start_timer(0.0, 1.0, true, false)?;
        }
        Ok(())
    }
}
