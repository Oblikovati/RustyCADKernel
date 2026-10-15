//! The default drawer members of `AIS_InteractiveContext` (trihedron and plane sizes, hidden
//! lines, initial attributes), the sub-intensity highlighting, the status text, the selected
//! shape, the entity owners, the sensitive debug display and DumpJson.
use super::{
    AisInteractiveContext, AisInteractiveObject, AisKindOfInteractive, AisSelectionMembers,
};
use crate::prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers};
use crate::prs3d::{Prs3dDatumParts, Prs3dLineAspect};
use crate::select_mgr::*;
use crate::std_select::{StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource};
use crate::v3d::{V3dView, V3dViewSource};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::{
    Graphic3dMaterialAspect, Graphic3dNameOfMaterial, Graphic3dPresentationAttributesMembers,
    Graphic3dPresentationAttributesSource,
};
use rustycad_tkservice::json_dump::{class_begin, field_integer, field_pointer};
use std::io;

/// The entity owners of an object, by identity (`NCollection_IndexedMap` of owner handles).
pub type AisEntityOwners = IndexedMap<Handle<dyn SelectMgrEntityOwnerSource>>;

impl AisInteractiveContext {
    /// `Redisplay(kind, signature, update)`: every object of the kind (the signature is
    /// ignored natively); the viewer is updated when a displayed one was redisplayed.
    /// Example: `context.redisplay_kind(AisKindOfInteractive::Datum, 3, true)?`.
    pub fn redisplay_kind(
        &self,
        kind: AisKindOfInteractive,
        _signature: i32,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let objects: Vec<_> = self
            .fields
            .read()
            .objects
            .items()
            .map(|(object, _)| object.clone())
            .collect();
        let mut redisplayed = false;
        for object in objects {
            let Some(value) = object.get() else {
                continue;
            };
            if value.kind_of_interactive() != kind {
                continue;
            }
            self.redisplay(&object, false, false)?;
            redisplayed = value.display_status() == PrsMgrDisplayStatus::Displayed || redisplayed;
        }
        if update_viewer && redisplayed {
            self.current_viewer().update();
        }
        Ok(())
    }
    /// `SetTrihedronSize`: the default axis lengths, then the trihedra and axes redisplayed.
    /// Example: `context.set_trihedron_size(50., true)?`.
    pub fn set_trihedron_size(&self, size: f64, _update_viewer: bool) -> Result<(), Failure> {
        self.default_drawer()
            .datum_aspect()
            .set_axis_length(size, size, size);
        self.redisplay_kind(AisKindOfInteractive::Datum, 3, false)?;
        self.redisplay_kind(AisKindOfInteractive::Datum, 4, true)
    }
    /// `TrihedronSize`: the X axis length; `context.trihedron_size()`.
    pub fn trihedron_size(&self) -> f64 {
        self.default_drawer()
            .datum_aspect()
            .axis_length(Prs3dDatumParts::XAxis)
    }
    /// `SetPlaneSize(x, y, update)`; `context.set_plane_size_xy(10., 20., true)?`.
    pub fn set_plane_size_xy(&self, x: f64, y: f64, update_viewer: bool) -> Result<(), Failure> {
        self.default_drawer().plane_aspect().set_plane_length(x, y);
        self.redisplay_kind(AisKindOfInteractive::Datum, 7, update_viewer)
    }
    /// `SetPlaneSize(size, update)`; `context.set_plane_size(10., true)?`.
    pub fn set_plane_size(&self, size: f64, update_viewer: bool) -> Result<(), Failure> {
        self.set_plane_size_xy(size, size, update_viewer)
    }
    /// `PlaneSize`: the plane lengths and whether they are equal within Confusion.
    /// Example: `let (equal, x, y) = context.plane_size();`.
    pub fn plane_size(&self) -> (bool, f64, f64) {
        let aspect = self.default_drawer().plane_aspect();
        let (x, y) = (aspect.plane_x_length(), aspect.plane_y_length());
        ((x - y).abs() <= precision::confusion(), x, y)
    }
    /// `HiddenLineAspect`; `context.hidden_line_aspect()`.
    pub fn hidden_line_aspect(&self) -> Handle<Prs3dLineAspect> {
        self.default_drawer().hidden_line_aspect()
    }
    /// `SetHiddenLineAspect`; `context.set_hidden_line_aspect(&aspect)`.
    pub fn set_hidden_line_aspect(&self, aspect: &Handle<Prs3dLineAspect>) {
        self.default_drawer().set_hidden_line_aspect(aspect.clone());
    }
    /// `DrawHiddenLine`; `context.draw_hidden_line()`.
    pub fn draw_hidden_line(&self) -> bool {
        self.default_drawer().draw_hidden_line()
    }
    /// `EnableDrawHiddenLine`; `context.enable_draw_hidden_line()`.
    pub fn enable_draw_hidden_line(&self) {
        self.default_drawer().enable_draw_hidden_line();
    }
    /// `DisableDrawHiddenLine`; `context.disable_draw_hidden_line()`.
    pub fn disable_draw_hidden_line(&self) {
        self.default_drawer().disable_draw_hidden_line();
    }
    /// `InitAttributes`: the default brass material, gray dashed hidden lines, pixel tolerance
    /// 2, 100 long steel blue trihedron axes and 200 wide sky blue planes.
    /// Example: `context.init_attributes()?`.
    pub fn init_attributes(&self) -> Result<(), Failure> {
        let drawer = self.default_drawer().clone();
        let material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Brass);
        drawer.shading_aspect().set_material(
            &material,
            rustycad_tkservice::aspect::AspectTypeOfFacingModel::BothSide,
        );
        let hidden = drawer.hidden_line_aspect();
        hidden.set_color(Color::named(NamedColor::GRAY20));
        hidden.set_width(1.0)?;
        hidden.set_type_of_line(AspectTypeOfLine::Dash);
        self.set_pixel_tolerance(2);
        let datum = drawer.datum_aspect();
        datum.set_axis_length(100., 100., 100.);
        let color = Color::named(NamedColor::LIGHTSTEELBLUE4);
        for part in [
            Prs3dDatumParts::XAxis,
            Prs3dDatumParts::YAxis,
            Prs3dDatumParts::ZAxis,
        ] {
            datum.line_aspect(part).set_color(color);
        }
        let plane = drawer.plane_aspect();
        plane.set_plane_length(200., 200.);
        plane
            .edges_aspect()
            .set_color(Color::named(NamedColor::SKYBLUE));
        Ok(())
    }

    /// `SetSubIntensityColor`; `context.set_sub_intensity_color(color)`.
    pub fn set_sub_intensity_color(&self, color: Color) {
        self.fields.read().styles[5].set_color(color);
    }
    /// `turnOnSubintensity(object, mode, displayed only)` (protected): the sub-intensity
    /// style (with the selection transparency) on a registered object, or on every object for
    /// a null one (natively a null dereference when only displayed objects are asked: a
    /// `Standard_NullObject` failure here). Example: `context.turn_on_subintensity(&object, -1, true)?`.
    pub fn turn_on_subintensity(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        display_mode: i32,
        displayed_only: bool,
    ) -> Result<(), Failure> {
        let (sub_style, selected) = {
            let fields = self.fields.read();
            (fields.styles[5].clone(), fields.styles[1].clone())
        };
        sub_style.set_transparency(selected.transparency());
        let manager = self.main_prs_mgr().clone();
        let entries: Vec<_> = if object.is_null() {
            if displayed_only {
                return Err(Failure::with_message(
                    "Standard_NullObject",
                    "AIS_InteractiveContext::turnOnSubintensity: null object",
                ));
            }
            self.fields
                .read()
                .objects
                .items()
                .map(|(object, status)| (object.clone(), status.clone()))
                .collect()
        } else {
            let Ok(status) = self.fields.read().objects.find(object).cloned() else {
                return Ok(());
            };
            if object.display_status() != PrsMgrDisplayStatus::Displayed && displayed_only {
                return Ok(());
            }
            vec![(object.clone(), status)]
        };
        for (object, status) in entries {
            status.set_sub_intensity(true);
            let mode = if display_mode != -1 {
                display_mode
            } else {
                status.display_mode()
            };
            manager.color(
                &object.cast(),
                &sub_style,
                mode,
                &Handle::default(),
                rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOPMOST,
            )?;
        }
        Ok(())
    }
    /// `SubIntensityOn`; `context.sub_intensity_on(&object, true)?`.
    pub fn sub_intensity_on(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        self.turn_on_subintensity(object, -1, true)?;
        if update_viewer {
            self.current_viewer().update();
        }
        Ok(())
    }
    /// `SubIntensityOff`: back to the plain (or selection) highlighting.
    /// Example: `context.sub_intensity_off(&object, true)?`.
    pub fn sub_intensity_off(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Ok(status) = self.fields.read().objects.find(object).cloned() else {
            return Ok(());
        };
        if !status.is_sub_intensity_on() {
            return Ok(());
        }
        status.set_sub_intensity(false);
        let mut update_main = false;
        if object.display_status() == PrsMgrDisplayStatus::Displayed {
            self.main_prs_mgr().unhighlight(&object.cast());
            update_main = true;
        }
        if self.is_selected_object(object) {
            self.highlight_selected(&object.global_sel_owner())?;
        }
        if update_viewer && update_main {
            self.current_viewer().update();
        }
        Ok(())
    }

    /// `Status`: the native report of the display and selection state of a registered object.
    /// Example: `context.status(&object)`.
    pub fn status(&self, object: &Handle<dyn AisInteractiveObject>) -> ExtendedString {
        let mut text = String::new();
        let Some(value) = object.get() else {
            return ExtendedString::new("");
        };
        let Ok(status) = self.fields.read().objects.find(object).cloned() else {
            return ExtendedString::new("");
        };
        text += "\t ____________________________________________";
        text += "\t| Known at Neutral Point:\n\tDisplayStatus:";
        match value.display_status() {
            PrsMgrDisplayStatus::Displayed => text += "\t| -->Displayed\n",
            PrsMgrDisplayStatus::Erased => text += "\t| -->Erased\n",
            _ => {}
        }
        text += "\t| Active Display Modes in the MainViewer :\n";
        text += "\t|\t Mode ";
        text += &status.display_mode().to_string();
        text += "\n";
        if self.is_selected_object(object) {
            text += "\t| Selected\n";
        }
        text += "\t| Active Selection Modes in the MainViewer :\n";
        for mode in status.selection_modes().iter() {
            text += "\t\t Mode ";
            text += &mode.to_string();
            text += "\n";
        }
        text += "\t ____________________________________________";
        ExtendedString::new(&text)
    }

    /// The BRep owner at the selection cursor, if any.
    fn current_brep_owner(&self) -> Option<Handle<dyn SelectMgrEntityOwnerSource>> {
        let selection = self.selection().clone();
        if !selection.more() {
            return None;
        }
        let owner = selection.value().ok()?.clone();
        (!owner.cast::<dyn StdSelectBRepOwnerSource>().is_null()).then_some(owner)
    }
    /// `HasSelectedShape`: a BRep owner with a shape at the selection cursor.
    /// Example: `context.has_selected_shape()`.
    pub fn has_selected_shape(&self) -> bool {
        self.current_brep_owner().is_some_and(|owner| {
            owner
                .cast::<dyn StdSelectBRepOwnerSource>()
                .get()
                .is_some_and(|brep| brep.has_shape())
        })
    }
    /// `SelectedShape`: the shape of the BRep owner at the cursor moved by the owner location
    /// (null without a selectable). Example: `context.selected_shape()?`.
    pub fn selected_shape(&self) -> Result<Shape, Failure> {
        let Some(owner) = self.current_brep_owner() else {
            return Ok(Shape::default());
        };
        let Some(value) = owner.get() else {
            return Ok(Shape::default());
        };
        if !value.has_selectable() {
            return Ok(Shape::default());
        }
        let brep = owner.cast::<dyn StdSelectBRepOwnerSource>();
        let shape = brep
            .get()
            .map(|brep| brep.shape_ref().clone_shape())
            .unwrap_or_default();
        let location = value.location().multiplied(shape.location())?;
        shape.located(&location, true)
    }
    /// `EntityOwners`: appends the owners of the sensitive entities of the given mode (every
    /// activated mode for -1) to the map. Example: `context.entity_owners(&mut owners, &object, -1)`.
    pub fn entity_owners(
        &self,
        owners: &mut AisEntityOwners,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
    ) {
        let Some(value) = object.get() else {
            return;
        };
        let mut modes = List::default();
        if mode == -1 {
            self.activated_modes(object, &mut modes);
        } else {
            modes.append(mode);
        }
        for mode in modes.iter() {
            let selection = value.selection(*mode);
            let Some(selection) = selection.get() else {
                continue;
            };
            for entity in selection.entities() {
                let sensitive = entity.base_sensitive();
                if let Some(sensitive) = sensitive.get() {
                    let owner =
                        crate::select3d::Select3DSensitiveEntityMembers::owner_id(sensitive);
                    if !owner.is_null() {
                        owners.add(owner);
                    }
                }
            }
        }
    }
    /// `RebuildSelectionStructs`: the object tree of the main selector, forced.
    /// Example: `context.rebuild_selection_structs()?`.
    pub fn rebuild_selection_structs(&self) -> Result<(), Failure> {
        self.main_selector().rebuild_objects_tree(true)
    }
    /// `DisplayActiveSensitive(view)`: the sensitive entities of the main selector.
    /// Example: `context.display_active_sensitive(&view)?`.
    pub fn display_active_sensitive(&self, view: &V3dView) -> Result<(), Failure> {
        self.main_selector().display_sensitive(view)
    }
    /// `DisplayActiveSensitive(object, view)`: the active selections of an object.
    /// Example: `context.display_object_active_sensitive(&object, &view)?`.
    pub fn display_object_active_sensitive(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        view: &Handle<dyn V3dViewSource>,
    ) -> Result<(), Failure> {
        let Ok(status) = self.fields.read().objects.find(object).cloned() else {
            return Ok(());
        };
        let selector = self.main_selector().clone();
        for mode in status.selection_modes().iter() {
            let selection = object.selection(*mode);
            selector.display_sensitive_selection(
                &selection,
                &object.transformation(),
                view,
                false,
            )?;
        }
        Ok(())
    }
    /// `ClearActiveSensitive(view)`; `context.clear_active_sensitive(&view)?`.
    pub fn clear_active_sensitive(&self, view: &Handle<dyn V3dViewSource>) -> Result<(), Failure> {
        self.main_selector().clear_sensitive(view)
    }
    /// `RedrawImmediate(viewer)`: the immediate presentations of the main manager.
    /// Example: `context.redraw_immediate(&viewer)?`.
    pub fn redraw_immediate(&self, viewer: &Handle<crate::v3d::V3dViewer>) -> Result<(), Failure> {
        self.main_prs_mgr().redraw_immediate(viewer)
    }
    /// `UpdateSelected`: the same as `HilightSelected`; `context.update_selected(true)?`.
    pub fn update_selected(&self, update_viewer: bool) -> Result<(), Failure> {
        self.hilight_selected(update_viewer)
    }
    /// `ObjectIterator`: the registered objects with their global status, in map order.
    /// Example: `for (object, status) in context.object_iterator() {}`.
    pub fn object_iterator(
        &self,
    ) -> Vec<(
        Handle<dyn AisInteractiveObject>,
        Handle<super::AisGlobalStatus>,
    )> {
        self.fields
            .read()
            .objects
            .items()
            .map(|(object, status)| (object.clone(), status.clone()))
            .collect()
    }

    /// `DumpJson`: the native fields (object count, manager, viewer, view, picked owner,
    /// selection, filters, drawers and detection state).
    /// Example: `context.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, _depth: i32) -> io::Result<()> {
        let fields = self.fields.read();
        class_begin(stream, "AIS_InteractiveContext")?;
        field_integer(stream, "myObjects.Size()", fields.objects.extent() as i32)?;
        field_pointer(
            stream,
            "mgrSelector.get()",
            fields.selection_manager.object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myMainPM.get()",
            fields.presentation_manager.object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myMainVwr.get()",
            fields.viewer.object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myLastActiveView",
            fields.last_active_view.upgrade().object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myLastPicked.get()",
            fields.last_picked.object_address() as usize,
        )?;
        field_integer(
            stream,
            "myToHilightSelected",
            i32::from(fields.to_hilight_selected),
        )?;
        field_pointer(
            stream,
            "mySelection.get()",
            fields.selection.object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myFilters.get()",
            fields.filters.object_address() as usize,
        )?;
        field_pointer(
            stream,
            "myDefaultDrawer.get()",
            fields.default_drawer.object_address() as usize,
        )?;
        for (index, name) in [
            (1, "myStyles[Prs3d_TypeOfHighlight_Selected]"),
            (2, "myStyles[Prs3d_TypeOfHighlight_Dynamic]"),
            (3, "myStyles[Prs3d_TypeOfHighlight_LocalSelected]"),
            (4, "myStyles[Prs3d_TypeOfHighlight_LocalDynamic]"),
            (5, "myStyles[Prs3d_TypeOfHighlight_SubIntensity]"),
        ] {
            field_pointer(stream, name, fields.styles[index].object_address() as usize)?;
        }
        field_integer(stream, "myDetectedSeq.Size()", fields.detected.length())?;
        field_integer(stream, "myCurDetected", fields.current_detected)?;
        field_integer(stream, "myCurHighlighted", fields.current_highlighted)?;
        field_integer(stream, "myPickingStrategy", fields.picking_strategy as i32)?;
        field_integer(stream, "myAutoHilight", i32::from(fields.automatic_hilight))?;
        field_integer(
            stream,
            "myIsAutoActivateSelMode",
            i32::from(fields.auto_activate_selection),
        )
    }
}
