//! The per-object attribute members of `AIS_InteractiveContext`: color, width, material,
//! transparency, polygon offsets, facing model, local drawer, deviation, display mode,
//! display priority, Z layer, transformation persistence, pixel tolerance, selection
//! sensitivity, the selection filters and the selection mode activation.
use super::{
    AisInteractiveContext, AisInteractiveObject, AisKindOfInteractive,
    AisSelectionModesConcurrency, AisShapeParameters, AisShapeSource,
};
use crate::prs_mgr::{
    PrsMgrDisplayStatus, PrsMgrPresentableObject, PrsMgrPresentableObjectMembers,
    PrsMgrPresentableObjectUpdates,
};
use crate::prs3d::Prs3dDrawer;
use crate::select_mgr::*;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::aspect::AspectTypeOfFacingModel;
use rustycad_tkservice::graphic3d::{
    GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dDisplayPriority, Graphic3dMaterialAspect,
    Graphic3dPresentationAttributesMembers, Graphic3dPresentationAttributesSource,
    Graphic3dTransformPersSource, Graphic3dZLayerId,
};

impl AisInteractiveContext {
    /// Ends an appearance edit: the presentations, then the viewer when asked.
    fn finish_appearance(
        &self,
        object: &dyn AisInteractiveObject,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        object.update_presentations(false)?;
        if update_viewer {
            self.update_current_viewer();
        }
        Ok(())
    }

    /// `SetColor(object, color, update)`: the object color, then its presentations.
    /// Example: `context.set_color(&object, red, true)?`.
    pub fn set_color(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        color: Color,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        value.set_color(color)?;
        self.finish_appearance(value, update_viewer)
    }
    /// `UnsetColor(object, update)`; `context.unset_color(&object, true)?`.
    pub fn unset_color(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        value.unset_color()?;
        self.finish_appearance(value, update_viewer)
    }
    /// `SetWidth(object, width, update)`: the width, then the highlight of the last picked
    /// owner of the object is redrawn with its selection or dynamic style.
    /// Example: `context.set_width(&object, 3., true)?`.
    pub fn set_width(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        width: f64,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        value.set_width(width)?;
        value.update_presentations(false)?;
        let last_picked = self.fields.read().last_picked.clone();
        if let Some(owner) = last_picked.get()
            && owner.is_same_selectable(&object.cast())
        {
            let style = if owner.is_selected() {
                self.get_sel_style(object, &last_picked).read()
            } else {
                self.get_hi_style(object, &last_picked).read()
            };
            let manager = self.detection_presentations();
            if owner.is_auto_hilight() {
                let mode = if value.has_hilight_mode() {
                    value.hilight_mode()
                } else {
                    0
                };
                owner.hilight_with_color(&manager, &style, mode)?;
            } else {
                value.hilight_owner_with_color(&manager, &style, &last_picked)?;
            }
        }
        if update_viewer {
            self.update_current_viewer();
        }
        Ok(())
    }
    /// `UnsetWidth(object, update)`; `context.unset_width(&object, true)?`.
    pub fn unset_width(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        value.unset_width()?;
        self.finish_appearance(value, update_viewer)
    }
    /// `SetMaterial(object, material, update)`; `context.set_material(&object, &gold, true)?`.
    pub fn set_material(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        material: &Graphic3dMaterialAspect,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        value.set_material(material)?;
        self.finish_appearance(value, update_viewer)
    }
    /// `UnsetMaterial(object, update)`; `context.unset_material(&object, true)?`.
    pub fn unset_material(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        value.unset_material()?;
        self.finish_appearance(value, update_viewer)
    }
    /// `SetTransparency(object, value, update)`: nothing for an opaque object and a value up to
    /// 0.005, which unsets the transparency of a transparent one.
    /// Example: `context.set_transparency(&object, 0.5, true)?`.
    pub fn set_transparency(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        transparency: f64,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        if !value.is_transparent() && transparency <= 0.005 {
            return Ok(());
        }
        if transparency <= 0.005 {
            return self.unset_transparency(object, update_viewer);
        }
        value.set_transparency(transparency)?;
        self.finish_appearance(value, update_viewer)
    }
    /// `UnsetTransparency(object, update)`; `context.unset_transparency(&object, true)?`.
    pub fn unset_transparency(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        value.unset_transparency()?;
        self.finish_appearance(value, update_viewer)
    }
    /// `SetPolygonOffsets`: the offsets of the object; the viewer is updated for a registered
    /// displayed object when asked. Example: `context.set_polygon_offsets(&object, 1, 1., 0., true)?`.
    pub fn set_polygon_offsets(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        factor: f32,
        units: f32,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        value.set_polygon_offsets(mode, factor, units)?;
        if update_viewer
            && self.fields.read().objects.is_bound(object)
            && value.display_status() == PrsMgrDisplayStatus::Displayed
        {
            self.current_viewer().update();
        }
        Ok(())
    }
    /// `HasPolygonOffsets`; `context.has_polygon_offsets(&object)`.
    pub fn has_polygon_offsets(&self, object: &Handle<dyn AisInteractiveObject>) -> bool {
        object
            .get()
            .is_some_and(|value| value.has_polygon_offsets())
    }
    /// `PolygonOffsets`: the offsets of an object that has them.
    /// Example: `let offsets = context.polygon_offsets(&object);`.
    pub fn polygon_offsets(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Option<(i32, f32, f32)> {
        if !self.has_polygon_offsets(object) {
            return None;
        }
        object.get().and_then(|value| value.polygon_offsets())
    }
    /// `Color`: the object color; a null object is a `Standard_NullObject` failure.
    /// Example: `context.color(&object)?`.
    pub fn color(&self, object: &Handle<dyn AisInteractiveObject>) -> Result<Color, Failure> {
        Ok(PrsMgrPresentableObject::color(required(object)?))
    }
    /// `Width`; `context.width(&object)?`.
    pub fn width(&self, object: &Handle<dyn AisInteractiveObject>) -> Result<f64, Failure> {
        Ok(required(object)?.width())
    }
    /// `HasColor`; `context.has_color(&object)?`.
    pub fn has_color(&self, object: &Handle<dyn AisInteractiveObject>) -> Result<bool, Failure> {
        Ok(required(object)?.has_color())
    }
    /// `SetCurrentFacingModel`; `context.set_current_facing_model(&object, model)`.
    pub fn set_current_facing_model(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        model: AspectTypeOfFacingModel,
    ) {
        if let Some(value) = object.get() {
            value.set_current_facing_model(model);
        }
    }
    /// `SetLocalAttributes`: the object drawer, then an update.
    /// Example: `context.set_local_attributes(&object, &drawer, true)?`.
    pub fn set_local_attributes(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        drawer: &Handle<Prs3dDrawer>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        value.set_attributes(drawer);
        self.update(object, update_viewer)
    }
    /// `UnsetLocalAttributes`; `context.unset_local_attributes(&object, true)?`.
    pub fn unset_local_attributes(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        value.unset_attributes();
        self.update(object, update_viewer)
    }
    /// `SetAngleAndDeviation`: for an AIS_Shape (kind Shape, signature 0) only.
    /// Example: `context.set_angle_and_deviation(&object, 0.2, true)?`.
    pub fn set_angle_and_deviation(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        angle: f64,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        if value.kind_of_interactive() != AisKindOfInteractive::Shape || value.signature() != 0 {
            return Ok(());
        }
        let shape = object.cast::<dyn AisShapeSource>();
        let shape = required_shape(&shape)?;
        shape.set_angle_and_deviation(angle);
        shape.update_presentations(false)?;
        if update_viewer {
            self.update_current_viewer();
        }
        Ok(())
    }

    /// The display mode a registered object is shown in.
    fn effective_display_mode(&self, value: &dyn AisInteractiveObject) -> i32 {
        let default_mode = self.default_drawer().display_mode();
        if value.has_display_mode() {
            value.display_mode()
        } else if value.accept_display_mode(default_mode) {
            default_mode
        } else {
            0
        }
    }
    /// `DisplayPriority`: the priority of the presentation of a displayed or erased object,
    /// INVALID otherwise. Example: `context.display_priority(&object)`.
    pub fn display_priority(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Graphic3dDisplayPriority {
        let Some(value) = object.get() else {
            return Graphic3dDisplayPriority::Invalid;
        };
        let shown = matches!(
            value.display_status(),
            PrsMgrDisplayStatus::Displayed | PrsMgrDisplayStatus::Erased
        );
        if !self.fields.read().objects.is_bound(object) || !shown {
            return Graphic3dDisplayPriority::Invalid;
        }
        let mode = self.effective_display_mode(value);
        self.main_prs_mgr().display_priority(&object.cast(), mode)
    }
    /// `SetDisplayPriority`; `context.set_display_priority(&object, Graphic3dDisplayPriority::Above1)?`.
    pub fn set_display_priority(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        priority: Graphic3dDisplayPriority,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        self.set_context_to_object(object)?;
        let shown = matches!(
            value.display_status(),
            PrsMgrDisplayStatus::Displayed | PrsMgrDisplayStatus::Erased
        );
        if self.fields.read().objects.is_bound(object) && shown {
            let mode = self.effective_display_mode(value);
            let manager = self.main_prs_mgr().clone();
            manager.set_display_priority(&object.cast(), mode, priority)?;
        }
        Ok(())
    }
    /// `GetZLayer`: UNKNOWN for a null object; `context.get_z_layer(&object)`.
    pub fn get_z_layer(&self, object: &Handle<dyn AisInteractiveObject>) -> Graphic3dZLayerId {
        object
            .get()
            .map_or(GRAPHIC3D_ZLAYER_ID_UNKNOWN, |value| value.z_layer())
    }
    /// `SetZLayer`; `context.set_z_layer(&object, GRAPHIC3D_ZLAYER_ID_TOP)`.
    pub fn set_z_layer(&self, object: &Handle<dyn AisInteractiveObject>, layer: Graphic3dZLayerId) {
        if let Some(value) = object.get() {
            value.set_z_layer(layer);
        }
    }
    /// `SetTransformPersistence`: the persistence, then for a registered object its selection
    /// and the BVH data and Z layer bounds of the active views.
    /// Example: `context.set_transform_persistence(&object, &persistence)?`.
    pub fn set_transform_persistence(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        persistence: &Handle<dyn Graphic3dTransformPersSource>,
    ) -> Result<(), Failure> {
        required(object)?.set_transform_persistence(persistence);
        if !self.fields.read().objects.is_bound(object) {
            return Ok(());
        }
        self.selection_manager().update_selection(&object.cast())?;
        let layer = required(object)?.z_layer();
        for view in self.current_viewer().active_views() {
            if let Some(view) = view.get() {
                let cview = view.view_base().view();
                if let Some(cview) = cview.get() {
                    cview.invalidate_bvh_data(layer);
                    cview.invalidate_z_layer_bounding_box(layer);
                }
            }
        }
        Ok(())
    }
    /// `SetPixelTolerance`: the tolerance of the main selector; `context.set_pixel_tolerance(2)`.
    pub fn set_pixel_tolerance(&self, precision: i32) {
        self.main_selector().set_pixel_tolerance(precision);
    }
    /// `PixelTolerance`; `context.pixel_tolerance()`.
    pub fn pixel_tolerance(&self) -> i32 {
        self.main_selector().pixel_tolerance()
    }
    /// `SetSelectionSensitivity`; `context.set_selection_sensitivity(&object, 0, 4)?`.
    pub fn set_selection_sensitivity(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        sensitivity: i32,
    ) -> Result<(), Failure> {
        self.selection_manager()
            .set_selection_sensitivity(&object.cast(), mode, sensitivity)
    }

    /// `DisplayMode`: the default display mode; `context.display_mode()`.
    pub fn display_mode(&self) -> i32 {
        self.default_drawer().display_mode()
    }
    /// `SetDisplayMode(mode, update)`: the default mode, applied to the displayed shapes and
    /// connected objects without own mode that accept it.
    /// Example: `context.set_default_display_mode(1, true)?`.
    pub fn set_default_display_mode(&self, mode: i32, update_viewer: bool) -> Result<(), Failure> {
        if mode == self.default_drawer().display_mode() {
            return Ok(());
        }
        let entries: Vec<_> = self
            .fields
            .read()
            .objects
            .items()
            .map(|(object, status)| (object.clone(), status.clone()))
            .collect();
        let manager = self.main_prs_mgr().clone();
        for (object, status) in entries {
            let Some(value) = object.get() else {
                continue;
            };
            let to_process = [
                "AIS_Shape",
                "AIS_ConnectedInteractive",
                "AIS_MultipleConnectedInteractive",
            ]
            .iter()
            .any(|name| object.is_kind_name(name));
            if !to_process || value.has_display_mode() || !value.accept_display_mode(mode) {
                continue;
            }
            status.set_display_mode(mode);
            if value.display_status() == PrsMgrDisplayStatus::Displayed {
                manager.display(&object.cast(), mode)?;
                let last_picked = self.fields.read().last_picked.clone();
                if last_picked
                    .get()
                    .is_some_and(|owner| owner.is_same_selectable(&object.cast()))
                {
                    manager.begin_immediate_draw();
                    self.unhighlight_global(&object)?;
                    manager.end_immediate_draw(&self.current_viewer().clone())?;
                }
                if status.is_sub_intensity_on() {
                    self.highlight_object_with_subintensity(&object, mode)?;
                }
                manager.set_visibility(&object.cast(), self.default_drawer().display_mode(), false);
            }
        }
        self.default_drawer().set_display_mode(mode);
        if update_viewer {
            self.current_viewer().update();
        }
        Ok(())
    }
    /// `SetDisplayMode(object, mode, update)`: the mode of an object; a registered displayed
    /// object hides its old mode, shows the new one and keeps its highlighting.
    /// Example: `context.set_display_mode(&object, 1, true)?`.
    pub fn set_display_mode(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        self.set_context_to_object(object)?;
        let value = required(object)?;
        let status = self.fields.read().objects.find(object).ok().cloned();
        let Some(status) = status else {
            value.set_display_mode(mode);
            return Ok(());
        };
        if !value.accept_display_mode(mode) {
            return Ok(());
        }
        if value.display_status() != PrsMgrDisplayStatus::Displayed {
            status.set_display_mode(mode);
            value.set_display_mode(mode);
            return Ok(());
        }
        let manager = self.main_prs_mgr().clone();
        let old_mode = status.display_mode();
        if old_mode != mode {
            if manager.is_highlighted(&object.cast(), old_mode) {
                self.unhighlight_global(object)?;
            }
            manager.set_visibility(&object.cast(), old_mode, false);
        }
        status.set_display_mode(mode);
        manager.display(&object.cast(), mode)?;
        if status.is_hilighted() {
            let owner = value.global_sel_owner();
            let style = self.get_sel_style(object, &owner).read();
            self.highlight_global(object, &style, mode)?;
        }
        if status.is_sub_intensity_on() {
            self.highlight_object_with_subintensity(object, mode)?;
        }
        if update_viewer {
            self.current_viewer().update();
        }
        value.set_display_mode(mode);
        Ok(())
    }
    /// `UnsetDisplayMode`: back to the default mode; `context.unset_display_mode(&object, true)?`.
    pub fn unset_display_mode(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        if !value.has_display_mode() {
            return Ok(());
        }
        let status = self.fields.read().objects.find(object).ok().cloned();
        let Some(status) = status else {
            value.unset_display_mode();
            return Ok(());
        };
        let old_mode = value.display_mode();
        let default_mode = self.default_drawer().display_mode();
        if default_mode == old_mode {
            return Ok(());
        }
        status.set_display_mode(default_mode);
        if value.display_status() == PrsMgrDisplayStatus::Displayed {
            let manager = self.main_prs_mgr().clone();
            if manager.is_highlighted(&object.cast(), old_mode) {
                self.unhighlight_global(object)?;
            }
            manager.set_visibility(&object.cast(), old_mode, false);
            manager.display(&object.cast(), default_mode)?;
            if status.is_hilighted() {
                self.highlight_selected(&value.global_sel_owner())?;
            }
            if status.is_sub_intensity_on() {
                self.highlight_object_with_subintensity(object, default_mode)?;
            }
            if update_viewer {
                self.current_viewer().update();
            }
        }
        value.unset_display_mode();
        Ok(())
    }

    /// `FilterType`: the combination of the context filters; `context.filter_type()`.
    pub fn filter_type(&self) -> SelectMgrFilterType {
        self.fields.read().filters.filter_type()
    }
    /// `SetFilterType`; `context.set_filter_type(SelectMgrFilterType::And)`.
    pub fn set_filter_type(&self, kind: SelectMgrFilterType) {
        self.fields.read().filters.set_filter_type(kind);
    }
    /// `Filters`: the stored filters; `context.filters()`.
    pub fn filters(&self) -> Vec<Handle<dyn SelectMgrFilter>> {
        self.fields.read().filters.stored_filters()
    }
    /// `GlobalFilter`: the AND/OR filter of the context; `context.global_filter()`.
    pub fn global_filter(&self) -> Handle<SelectMgrAndOrFilter> {
        self.fields.read().filters.clone()
    }
    /// `AddFilter`; `context.add_filter(&filter)`.
    pub fn add_filter(&self, filter: &Handle<dyn SelectMgrFilter>) {
        self.fields.read().filters.add(filter);
    }
    /// `RemoveFilter`; `context.remove_filter(&filter)`.
    pub fn remove_filter(&self, filter: &Handle<dyn SelectMgrFilter>) {
        self.fields.read().filters.remove(filter);
    }
    /// `RemoveFilters`; `context.remove_filters()`.
    pub fn remove_filters(&self) {
        self.fields.read().filters.clear();
    }

    /// `Activate(object, mode, force)`: a selection mode of an object, the other modes kept
    /// unless they mix the global mode and local ones. Example: `context.activate(&object, 0, false)?`.
    pub fn activate(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        force: bool,
    ) -> Result<(), Failure> {
        self.set_selection_mode_active(
            object,
            mode,
            true,
            AisSelectionModesConcurrency::GlobalOrLocal,
            force,
        )
    }
    /// `Activate(mode, force)`: the mode of every displayed object (loaded first).
    /// Example: `context.activate_all(2, false)?`.
    pub fn activate_all(&self, mode: i32, force: bool) -> Result<(), Failure> {
        let mut objects = List::default();
        self.displayed_objects(&mut objects)?;
        for object in objects.iter() {
            self.load(object, -1)?;
            self.activate(object, mode, force)?;
        }
        Ok(())
    }
    /// `Deactivate(object)`: every selection mode of an object.
    /// Example: `context.deactivate(&object)?`.
    pub fn deactivate(&self, object: &Handle<dyn AisInteractiveObject>) -> Result<(), Failure> {
        self.set_selection_mode_active(
            object,
            -1,
            false,
            AisSelectionModesConcurrency::Single,
            false,
        )
    }
    /// `Deactivate(object, mode)`; `context.deactivate_mode(&object, 2)?`.
    pub fn deactivate_mode(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.set_selection_mode_active(
            object,
            mode,
            false,
            AisSelectionModesConcurrency::Multiple,
            false,
        )
    }
    /// `Deactivate(mode)`: the mode of every displayed object; `context.deactivate_all_mode(2)?`.
    pub fn deactivate_all_mode(&self, mode: i32) -> Result<(), Failure> {
        let mut objects = List::default();
        self.displayed_objects(&mut objects)?;
        for object in objects.iter() {
            self.deactivate_mode(object, mode)?;
        }
        Ok(())
    }
    /// `Deactivate()`: every mode of every displayed object; `context.deactivate_all()?`.
    pub fn deactivate_all(&self) -> Result<(), Failure> {
        let mut objects = List::default();
        self.displayed_objects(&mut objects)?;
        for object in objects.iter() {
            self.deactivate(object)?;
        }
        Ok(())
    }
    /// `SetSelectionModeActive`: activates or deactivates a selection mode of a registered
    /// object (-1 for every mode), the other active modes treated by the concurrency policy;
    /// the selector is touched for a displayed object or when forced.
    /// Example: `context.set_selection_mode_active(&object, 2, true, AisSelectionModesConcurrency::Multiple, false)?`.
    pub fn set_selection_mode_active(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        is_active: bool,
        concurrency: AisSelectionModesConcurrency,
        force: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        let status = self.fields.read().objects.find(object).ok().cloned();
        let Some(status) = status else {
            return Ok(());
        };
        let manager = self.selection_manager().clone();
        let selectable: Handle<dyn SelectMgrSelectableObject> = object.cast();
        let touch = value.display_status() == PrsMgrDisplayStatus::Displayed || force;
        if !is_active || (mode == -1 && concurrency == AisSelectionModesConcurrency::Single) {
            if touch {
                if mode == -1 {
                    for active in status.selection_modes().iter() {
                        manager.deactivate(&selectable, *active)?;
                    }
                } else {
                    manager.deactivate(&selectable, mode)?;
                }
            }
            if mode == -1 {
                status.clear_selection_modes();
            } else {
                status.remove_selection_mode(mode);
            }
            return Ok(());
        } else if mode == -1 {
            return Ok(());
        }
        let modes: Vec<i32> = status.selection_modes().iter().copied().collect();
        if modes.len() == 1 && modes[0] == mode {
            return Ok(());
        }
        if touch {
            match concurrency {
                AisSelectionModesConcurrency::Single => {
                    for active in &modes {
                        manager.deactivate(&selectable, *active)?;
                    }
                    status.clear_selection_modes();
                }
                AisSelectionModesConcurrency::GlobalOrLocal => {
                    let global = value.global_selection_mode();
                    let mut removed = Vec::new();
                    for active in &modes {
                        if (mode == global && *active != global)
                            || (mode != global && *active == global)
                        {
                            manager.deactivate(&selectable, *active)?;
                            removed.push(*active);
                        }
                    }
                    if removed.len() == modes.len() {
                        status.clear_selection_modes();
                    } else {
                        for active in removed {
                            status.remove_selection_mode(active);
                        }
                    }
                }
                AisSelectionModesConcurrency::Multiple => {}
            }
            manager.activate(&selectable, mode)?;
        }
        status.add_selection_mode(mode);
        Ok(())
    }
}

/// The object of a handle; a null handle is a `Standard_NullObject` failure (a null
/// dereference natively).
fn required(
    object: &Handle<dyn AisInteractiveObject>,
) -> Result<&dyn AisInteractiveObject, Failure> {
    object.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "AIS_InteractiveContext: null interactive object",
        )
    })
}

fn required_shape(object: &Handle<dyn AisShapeSource>) -> Result<&dyn AisShapeSource, Failure> {
    object.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "AIS_InteractiveContext: not an AIS_Shape",
        )
    })
}
