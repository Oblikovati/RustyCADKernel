//! Selection picks remember the view before picking and then apply the current scheme.
use super::*;
use crate::v3d::V3dViewSource;
use crate::{select_basics::SelectBasicsSelectingVolumeManager, select_mgr::*};
use rustycad_tkernel::{
    ncollection::{Array1, Vec2},
    precision,
    standard::{Failure, Handle, catch_native_failure, exceptions::program_error},
};
use rustycad_tkmath::gp::Point2;
use rustycad_tkservice::aspect::{ASPECT_VKEY_FLAGS_NONE, ASPECT_VKEY_MOUSE_LEFT_BUTTON};
impl AisInteractiveContext {
    /// Select a pixel rectangle using the supplied scheme; `context.select_rectangle(Vec2::new(0,0),Vec2::new(40,30),&view,AisSelectionScheme::Replace)?`.
    pub fn select_rectangle(
        &self,
        minimum: Vec2<i32>,
        maximum: Vec2<i32>,
        view: &Handle<dyn V3dViewSource>,
        scheme: AisSelectionScheme,
    ) -> Result<AisStatusOfPick, Failure> {
        self.prepare_selection_view(view, "SelectRectangle")?;
        self.detection_selector().pick_box(
            minimum.x(),
            minimum.y(),
            maximum.x(),
            maximum.y(),
            view,
        )?;
        self.select_current_picks(scheme)
    }
    /// Select a pixel polygon using the supplied scheme; `context.select_polygon(&pixels,&view,AisSelectionScheme::Add)?`.
    pub fn select_polygon(
        &self,
        pixels: &Array1<'_, Point2>,
        view: &Handle<dyn V3dViewSource>,
        scheme: AisSelectionScheme,
    ) -> Result<AisStatusOfPick, Failure> {
        self.prepare_selection_view(view, "SelectPolygon")?;
        self.detection_selector().pick_polyline(pixels, view)?;
        self.select_current_picks(scheme)
    }
    /// Select a pixel using the supplied scheme; `context.select_point(Vec2::new(40,30),&view,AisSelectionScheme::Xor)?`.
    pub fn select_point(
        &self,
        point: Vec2<i32>,
        view: &Handle<dyn V3dViewSource>,
        scheme: AisSelectionScheme,
    ) -> Result<AisStatusOfPick, Failure> {
        self.prepare_selection_view(view, "SelectPoint")?;
        self.detection_selector()
            .pick_point(point.x(), point.y(), view)?;
        self.select_current_picks(scheme)
    }
    fn prepare_selection_view(
        &self,
        view: &Handle<dyn V3dViewSource>,
        operation: &str,
    ) -> Result<(), Failure> {
        if view.viewer() != self.detection_viewer() {
            return Err(program_error(&format!(
                "{operation}: view {:?}; expected the context viewer",
                view.object_address()
            )));
        }
        self.fields.write().last_active_view = view.downgrade();
        Ok(())
    }
    fn select_current_picks(&self, scheme: AisSelectionScheme) -> Result<AisStatusOfPick, Failure> {
        let count = self.detection_selector().nb_picked();
        let mut owners = if count > 0 {
            Array1::with_bounds(1, count).expect("positive picked count")
        } else {
            Array1::default()
        };
        let mut rank = 1;
        while rank <= self.detection_selector().nb_picked() {
            *owners.change_value(rank).map_err(|error| {
                rustycad_tkernel::standard::exceptions::out_of_range(&error.to_string())
            })? = self.detection_selector().picked(rank)?;
            rank += 1;
        }
        self.select_owners(&owners, scheme)
    }
    /// Select the detected owner unless it consumes the mouse click; `context.select_detected(AisSelectionScheme::Replace)?`.
    pub fn select_detected(&self, scheme: AisSelectionScheme) -> Result<AisStatusOfPick, Failure> {
        catch_native_failure(|| self.select_detected_checked(scheme))
    }
    fn select_detected_checked(
        &self,
        scheme: AisSelectionScheme,
    ) -> Result<AisStatusOfPick, Failure> {
        if self.detected_click_consumed(scheme) {
            return Ok(AisStatusOfPick::NothingSelected);
        }
        let owner = self.fields.read().last_picked.clone();
        self.select_owners(&Array1::from_shared(&[owner], 1), scheme)
    }
    fn detected_click_consumed(&self, scheme: AisSelectionScheme) -> bool {
        if !click_selection_scheme(scheme) || !self.has_detected() {
            return false;
        }
        let pixels = self.detected_click_position();
        // OCCT 3d097a0 invokes the field directly; a storage lease preserves native owner counts.
        let owner = self.fields.read().last_picked.observe();
        let Some(owner) = owner.get() else {
            return false;
        };
        owner.handle_mouse_click(
            pixels,
            ASPECT_VKEY_MOUSE_LEFT_BUTTON,
            ASPECT_VKEY_FLAGS_NONE,
            false,
        )
    }
    fn detected_click_position(&self) -> Vec2<i32> {
        let selector = self.detection_selector();
        let volume = selector
            .viewer_selector_base()
            .selecting_volume
            .read()
            .clone();
        let point = volume.get_mouse_position();
        if precision::is_infinite(point.x()) || precision::is_infinite(point.y()) {
            Vec2::new(-1, -1)
        } else {
            Vec2::new(point.x() as i32, point.y() as i32)
        }
    }
}

fn click_selection_scheme(scheme: AisSelectionScheme) -> bool {
    !matches!(
        scheme,
        AisSelectionScheme::Remove | AisSelectionScheme::Clear
    )
}
