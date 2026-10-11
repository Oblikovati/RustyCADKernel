//! Martensite touch transitions mirror native controller/base virtual dispatch.
use super::AisViewController;
use rustycad_tkernel::{ncollection::Vec2, osd::Timer, standard::Failure};
use rustycad_tkservice::aspect::{
    ASPECT_TOUCH_ID_ALL, ASPECT_VKEY_FLAGS_NONE, ASPECT_VKEY_MOUSE_LEFT_BUTTON, AspectTouch,
};
pub(super) struct ViewControllerTouchDispatch {
    pub(super) click: AspectTouch,
    pub(super) click_threshold_px: f32,
    pub(super) double_tap_timer: Timer,
    pub(super) start_rotation: Vec2<f64>,
    pub(super) start_pan: Vec2<f64>,
    pub(super) update_start_rotation: bool,
    pub(super) update_start_pan: bool,
}
impl ViewControllerTouchDispatch {
    pub(super) fn with_timer(mut timer: Timer) -> Self {
        timer.reset();
        Self {
            click: AspectTouch::default(),
            click_threshold_px: 3.0,
            double_tap_timer: timer,
            start_rotation: Vec2::new(0.0, 0.0),
            start_pan: Vec2::new(0.0, 0.0),
            update_start_rotation: true,
            update_start_pan: true,
        }
    }
}
impl AisViewController {
    /// Queue a touch, dispatching clear-before through controller removal; `controller.add_touch_point(1, point, false)?`.
    pub fn add_touch_point(
        &mut self,
        id: usize,
        point: Vec2<f64>,
        clear_before: bool,
    ) -> Result<(), Failure> {
        self.ui_input.move_to.to_hilight = false;
        if clear_before {
            self.remove_touch_point(ASPECT_TOUCH_ID_ALL, false)?;
        }
        self.input_base
            .touch_points
            .add(id, AspectTouch::new(point, false));
        self.touch_dispatch.click.from = Vec2::new(-1.0, -1.0);
        self.begin_touch_gesture(point);
        self.ui_input.is_new_gesture = true;
        Ok(())
    }
    fn begin_touch_gesture(&mut self, point: Vec2<f64>) {
        match self.input_base.touch_points.extent() {
            1 => self.begin_single_touch(point),
            2 => {
                self.ui_input.dragging.to_abort = true;
                self.touch_dispatch.update_start_pan = true;
                self.touch_dispatch.start_pan = point;
            }
            _ => (),
        }
    }
    fn begin_single_touch(&mut self, point: Vec2<f64>) {
        self.touch_dispatch.click.from = point;
        self.touch_dispatch.update_start_rotation = true;
        self.touch_dispatch.start_rotation = point;
        if !self.to_allow_dragging() {
            return;
        }
        self.ui_input.dragging.to_start = true;
        self.ui_input.dragging.point_start = Vec2::new(point.x as i32, point.y as i32);
    }
    /// Remove a touch or all touches, preserving pinch transitions; `controller.remove_touch_point(1, true)?`.
    pub fn remove_touch_point(
        &mut self,
        id: usize,
        clear_selection: bool,
    ) -> Result<bool, Failure> {
        if id == ASPECT_TOUCH_ID_ALL {
            self.input_base.touch_points.clear(false);
        } else if !self.input_base.touch_points.remove_key(&id) {
            return Ok(false);
        }
        self.end_touch_gesture(id, clear_selection)?;
        self.ui_input.is_new_gesture = true;
        Ok(true)
    }
    fn end_touch_gesture(&mut self, id: usize, clear_selection: bool) -> Result<(), Failure> {
        match self.input_base.touch_points.extent() {
            0 => self.finish_last_touch(id, clear_selection)?,
            1 => {
                let first = self.input_base.touch_points.change_from_index(1)?;
                first.to = first.from;
                self.touch_dispatch.start_rotation = first.to;
                self.touch_dispatch.update_start_rotation = true;
            }
            2 => {
                self.touch_dispatch.start_pan = self.input_base.touch_points.find_from_index(1)?.to;
                self.touch_dispatch.update_start_pan = true;
            }
            _ => (),
        }
        Ok(())
    }
    fn finish_last_touch(&mut self, id: usize, clear_select_points: bool) -> Result<(), Failure> {
        if clear_select_points {
            self.ui_input.selection.to_apply_tool = true;
        }
        self.ui_input.dragging.to_stop = true;
        if id == ASPECT_TOUCH_ID_ALL {
            self.touch_dispatch.click.from = Vec2::new(-1.0, -1.0);
            return Ok(());
        }
        self.dispatch_touch_click()
    }
    fn dispatch_touch_click(&mut self) -> Result<(), Failure> {
        let point = self.touch_dispatch.click.from;
        let minimum = if point.x < point.y { point.x } else { point.y };
        if minimum < 0.0 || minimum.is_nan() {
            return Ok(());
        }
        let double_click = self.touch_double_click();
        self.update_mouse_click(
            Vec2::new(point.x as i32, point.y as i32),
            ASPECT_VKEY_MOUSE_LEFT_BUTTON,
            ASPECT_VKEY_FLAGS_NONE,
            double_click,
        )?;
        Ok(())
    }
    pub(super) fn touch_double_click(&mut self) -> bool {
        let interval = self.mouse_double_click_interval();
        let timer = &mut self.touch_dispatch.double_tap_timer;
        if timer.is_started() && timer.elapsed_time() <= interval {
            return true;
        }
        timer.stop();
        timer.reset();
        timer.start();
        false
    }
    /// Move an existing touch or queue an unknown id through add; `controller.update_touch_point(1, point)?`.
    pub fn update_touch_point(&mut self, id: usize, point: Vec2<f64>) -> Result<(), Failure> {
        if let Some(touch) = self.input_base.touch_points.change_seek(&id) {
            touch.to = point;
        } else {
            self.add_touch_point(id, point, false)?;
        }
        let tolerance =
            self.touch_tolerance_scale() as f64 * self.touch_dispatch.click_threshold_px as f64;
        let displacement = self.touch_dispatch.click.from - point;
        let x = displacement.x.abs();
        let y = displacement.y.abs();
        // OCCT Vec2::maxComp uses x > y ? x : y, including its NaN branch.
        let maximum = if x > y { x } else { y };
        if self.input_base.touch_points.extent() == 1 && maximum > tolerance {
            self.touch_dispatch.click.from = Vec2::new(-1.0, -1.0);
        }
        Ok(())
    }
}
