//! The immediate mode members of `PrsMgr_PresentationManager`: the immediate lists, the
//! highlight `Color` and the transformation of the highlight shadows.
use super::{
    PrsMgrDisplayStatus, PrsMgrPresentableObject, PrsMgrPresentableObjectMembers as _,
    PrsMgrPresentationManager,
};
use crate::prs3d::{Prs3dDrawer, Prs3dPresentationShadow};
use crate::v3d::{V3dView, V3dViewer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::graphic3d::{
    Graphic3dCStructureMembers as _, Graphic3dCView, Graphic3dCViewMembers as _,
    Graphic3dCViewStructures as _, Graphic3dStructureDisplay as _, Graphic3dStructureMembers as _,
    Graphic3dStructureSource, Graphic3dViewAffinity, Graphic3dZLayerId,
};

/// The driver views of the active views of a viewer.
fn active_cviews(viewer: &Handle<V3dViewer>) -> Vec<Handle<dyn Graphic3dCView>> {
    let views = viewer
        .get()
        .map(V3dViewer::active_views)
        .unwrap_or_default();
    views
        .iter()
        .filter_map(Handle::get)
        .map(V3dView::view)
        .collect()
}

/// A highlighted shadow of `parent` in `layer` with the parent's clipping planes (`Color`
/// and `displayImmediate`).
fn highlight_shadow(
    manager: &Handle<rustycad_tkservice::graphic3d::Graphic3dStructureManager>,
    parent: &Handle<dyn Graphic3dStructureSource>,
    layer: Graphic3dZLayerId,
    style: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dPresentationAttributesSource>,
) -> Result<Handle<Prs3dPresentationShadow>, Failure> {
    let shadow = Prs3dPresentationShadow::new(manager, parent)?;
    shadow.set_z_layer(layer);
    shadow.set_clip_planes(&parent.get().map(|p| p.clip_planes()).unwrap_or_default());
    shadow.set_is_for_highlight(true);
    shadow.highlight(style, true);
    Ok(shadow)
}

/// Shows a shadow in one view only, as visible there as its parent (`displayImmediate`).
fn show_in_view(shadow: &Prs3dPresentationShadow, view: &dyn Graphic3dCView) {
    if !shadow.is_displayed() {
        let affinity = Graphic3dViewAffinity::default();
        affinity.set_visible(false);
        if let Some(cstructure) = shadow.cstructure().get() {
            cstructure.cstructure_base().fields.write().view_affinity = Handle::new(affinity);
        }
        shadow.display();
    }
    let view_id = view.identification();
    let parent = shadow.parent_affinity();
    let is_parent_visible = parent.get().is_none_or(|a| a.is_visible(view_id));
    let affinity = shadow
        .cstructure()
        .get()
        .map(|c| c.cstructure_base().fields.read().view_affinity.clone());
    if let Some(affinity) = affinity.as_ref().and_then(Handle::get) {
        affinity.set_view_visible(view_id, is_parent_visible);
    }
}

/// Sets the transformation of the shadows of the parent structure `ref_id`.
fn update_prs_transformation(
    list: &[Handle<dyn Graphic3dStructureSource>],
    ref_id: i32,
    trsf: &Handle<Datum3>,
) {
    for prs in list {
        let shadow: Handle<Prs3dPresentationShadow> = Handle::down_cast(prs);
        let Some(shadow) = shadow.get().filter(|s| s.parent_id() == ref_id) else {
            continue;
        };
        if let Some(cstructure) = shadow.cstructure().get() {
            cstructure.set_transformation(trsf);
        }
    }
}

impl PrsMgrPresentationManager {
    /// `IsImmediateModeOn`; `manager.is_immediate_mode_on()`.
    pub fn is_immediate_mode_on(&self) -> bool {
        self.immediate.read().mode_on > 0
    }
    /// `BeginImmediateDraw`: enters the immediate mode (nested), clearing the lists on entry.
    /// Example: `manager.begin_immediate_draw()`.
    pub fn begin_immediate_draw(&self) {
        let level = {
            let mut state = self.immediate.write();
            state.mode_on += 1;
            state.mode_on
        };
        if level <= 1 {
            self.clear_immediate_draw();
        }
    }
    /// `ClearImmediateDraw`: erases and forgets the immediate structures; `manager.clear_immediate_draw()`.
    pub fn clear_immediate_draw(&self) {
        let state = {
            let mut own = self.immediate.write();
            let copy = own.clone();
            own.list.clear();
            own.view_dependent_list.clear();
            copy
        };
        state
            .list
            .iter()
            .chain(&state.view_dependent_list)
            .filter_map(Handle::get)
            .for_each(|p| p.erase());
    }
    /// `AddToImmediateList`: queues a structure once, in immediate mode only.
    /// Example: `manager.add_to_immediate_list(&prs)`.
    pub fn add_to_immediate_list(&self, prs: &Handle<dyn Graphic3dStructureSource>) {
        let mut state = self.immediate.write();
        if state.mode_on < 1 || state.list.iter().any(|p| p == prs) {
            return;
        }
        state.list.push(prs.clone());
    }
    /// `EndImmediateDraw`: leaves the immediate mode, displaying the queue in the active views
    /// of the viewer when the outermost level ends. Example: `manager.end_immediate_draw(&viewer)?`.
    pub fn end_immediate_draw(&self, viewer: &Handle<V3dViewer>) -> Result<(), Failure> {
        let level = {
            let mut state = self.immediate.write();
            state.mode_on -= 1;
            state.mode_on
        };
        if level > 0 {
            return Ok(());
        }
        self.display_immediate(viewer)
    }
    /// `RedrawImmediate`: erases and displays the queue again; `manager.redraw_immediate(&viewer)?`.
    pub fn redraw_immediate(&self, viewer: &Handle<V3dViewer>) -> Result<(), Failure> {
        let state = self.immediate.read().clone();
        if state.list.is_empty() {
            return Ok(());
        }
        state
            .list
            .iter()
            .chain(&state.view_dependent_list)
            .filter_map(Handle::get)
            .for_each(|p| p.erase());
        self.immediate.write().view_dependent_list.clear();
        self.display_immediate(viewer)
    }
    /// `displayImmediate` (private): a plain structure is displayed; a highlight shadow is
    /// shown per view, replaced by a shadow of the view's computed structure when the view
    /// computes its parent.
    fn display_immediate(&self, viewer: &Handle<V3dViewer>) -> Result<(), Failure> {
        for cview in active_cviews(viewer) {
            let Some(view) = cview.get() else { continue };
            let list = self.immediate.read().list.clone();
            for prs in list.iter().filter(|p| !p.is_null()) {
                self.display_immediate_in(view, prs)?;
            }
        }
        Ok(())
    }
    /// One structure of `displayImmediate` in one view.
    fn display_immediate_in(
        &self,
        view: &dyn Graphic3dCView,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let mut shadow: Handle<Prs3dPresentationShadow> = Handle::down_cast(prs);
        let Some(current) = shadow.get() else {
            if let Some(value) = prs.get() {
                value.display();
            }
            return Ok(());
        };
        let computed = view.is_computed(current.parent_id());
        if !computed.is_null() {
            let style = prs.get().map(|p| p.highlight_style()).unwrap_or_default();
            shadow = highlight_shadow(
                &self.structure_manager(),
                &computed,
                current.get_z_layer(),
                &style,
            )?;
            self.immediate
                .write()
                .view_dependent_list
                .push(shadow.clone().cast());
        }
        if let Some(value) = shadow.get() {
            show_in_view(value, view);
        }
        Ok(())
    }
    /// `Color`: highlights the presentation of the mode with a style (non-erased children
    /// first), in immediate mode as a highlight shadow in `immediate_layer_id`.
    /// Example: `manager.color(&object, &style, 0, &Handle::default(), GRAPHIC3D_ZLAYER_ID_TOPMOST)?`.
    pub fn color(
        &self,
        object: &Handle<dyn PrsMgrPresentableObject>,
        style: &Handle<Prs3dDrawer>,
        mode: i32,
        sel_obj: &Handle<dyn PrsMgrPresentableObject>,
        immediate_layer_id: Graphic3dZLayerId,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        if value.to_propagate_visual_state() {
            for child in value.children() {
                if child
                    .get()
                    .is_some_and(|c| c.display_status() != PrsMgrDisplayStatus::Erased)
                {
                    self.color(&child, style, mode, &Handle::default(), immediate_layer_id)?;
                }
            }
        }
        if !value.has_own_presentations() {
            return Ok(());
        }
        let prs = self.presentation(object, mode, true, sel_obj)?;
        if prs
            .get()
            .is_some_and(super::PrsMgrPresentation::must_be_updated)
        {
            self.update(object, mode)?;
        }
        if !self.is_immediate_mode_on() {
            if let Some(prs) = prs.get() {
                prs.highlight_with(style);
            }
            return Ok(());
        }
        let shadow = highlight_shadow(
            &self.structure_manager(),
            &prs.cast(),
            immediate_layer_id,
            &style.clone().cast(),
        )?;
        self.add_to_immediate_list(&shadow.cast());
        Ok(())
    }
    /// `UpdateHighlightTrsf`: gives the local transformation of `object` to the highlight
    /// shadows of its presentation (or of `sel_obj`'s), also in the views that compute it.
    /// Example: `manager.update_highlight_trsf(&viewer, &object, 0, &Handle::default())?`.
    pub fn update_highlight_trsf(
        &self,
        viewer: &Handle<V3dViewer>,
        object: &Handle<dyn PrsMgrPresentableObject>,
        mode: i32,
        sel_obj: &Handle<dyn PrsMgrPresentableObject>,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        let owner = if sel_obj.is_null() { object } else { sel_obj };
        let prs = self.presentation(owner, mode, false, &Handle::default())?;
        let Some(prs) = prs.get() else {
            return Ok(());
        };
        let trsf = value.local_transformation_geom();
        let parent_id = prs.cstructure().get().map_or(0, |c| c.identification());
        let state = self.immediate.read().clone();
        update_prs_transformation(&state.list, parent_id, &trsf);
        if state.view_dependent_list.is_empty() {
            return Ok(());
        }
        for cview in active_cviews(viewer) {
            let computed = cview
                .get()
                .map(|v| v.is_computed(parent_id))
                .unwrap_or_default();
            if let Some(id) = computed
                .get()
                .and_then(|c| c.cstructure().get().map(|s| s.identification()))
            {
                update_prs_transformation(&state.view_dependent_list, id, &trsf);
            }
        }
        Ok(())
    }
}
