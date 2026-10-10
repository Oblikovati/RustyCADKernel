//! Borrowed context handles and scalar state preserve native storage identity.
use super::{AisInteractiveContext, AisInteractiveContextFields, AisSelectionSource};
use crate::{
    prs_mgr::PrsMgrPresentationManager,
    prs3d::{Prs3dDrawer, Prs3dTypeOfHighlight},
    select_mgr::*,
    v3d::{V3dView, V3dViewer},
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, exceptions::out_of_range};
use std::{ops::Deref, sync::RwLockReadGuard};
/// Borrow an actual retained context handle; release before changing context state.
/// Example: `context.default_drawer().display_mode()`.
pub struct AisContextHandle<'a, T: ?Sized + RuntimeTyped> {
    fields: RwLockReadGuard<'a, AisInteractiveContextFields>,
    project: fn(&AisInteractiveContextFields, usize) -> &Handle<T>,
    slot: usize,
}
impl<T: ?Sized + RuntimeTyped> Deref for AisContextHandle<'_, T> {
    type Target = Handle<T>;
    fn deref(&self) -> &Self::Target {
        (self.project)(&self.fields, self.slot)
    }
}
impl AisInteractiveContext {
    pub(super) fn borrow_handle<T: ?Sized + RuntimeTyped>(
        &self,
        slot: usize,
        project: fn(&AisInteractiveContextFields, usize) -> &Handle<T>,
    ) -> AisContextHandle<'_, T> {
        AisContextHandle {
            fields: self.fields.read(),
            project,
            slot,
        }
    }
    /// Borrow the default drawing attributes; `context.default_drawer()`.
    pub fn default_drawer(&self) -> AisContextHandle<'_, Prs3dDrawer> {
        self.borrow_handle(0, |f, _| &f.default_drawer)
    }
    /// Replace default and None style without relinking existing objects; `context.set_default_drawer(&drawer)`.
    pub fn set_default_drawer(&self, drawer: &Handle<Prs3dDrawer>) {
        let mut fields = self.fields.write();
        fields.default_drawer = drawer.clone();
        fields.styles[0] = drawer.clone();
    }
    /// Borrow the current viewer; `context.current_viewer()`.
    pub fn current_viewer(&self) -> AisContextHandle<'_, V3dViewer> {
        self.borrow_handle(0, |f, _| &f.viewer)
    }
    /// Borrow the selection manager; `context.selection_manager()`.
    pub fn selection_manager(&self) -> AisContextHandle<'_, SelectMgrSelectionManager> {
        self.borrow_handle(0, |f, _| &f.selection_manager)
    }
    /// Borrow the presentation manager; `context.main_prs_mgr()`.
    pub fn main_prs_mgr(&self) -> AisContextHandle<'_, PrsMgrPresentationManager> {
        self.borrow_handle(0, |f, _| &f.presentation_manager)
    }
    /// Borrow the selection manager's actual selector; `context.main_selector()`.
    pub fn main_selector(&self) -> AisContextHandle<'_, dyn SelectMgrViewerSelectorSource> {
        self.borrow_handle(0, |f, _| f.selection_manager.selector())
    }
    /// Borrow the selected-owner container; `context.selection()`.
    pub fn selection(&self) -> AisContextHandle<'_, dyn AisSelectionSource> {
        self.borrow_handle(0, |f, _| &f.selection)
    }
    /// Replace the selected-owner container without clearing either container; `context.set_selection(&selection.cast())`.
    pub fn set_selection(&self, selection: &Handle<dyn AisSelectionSource>) {
        self.fields.write().selection = selection.clone();
    }
    /// Borrow one of the six style handles; `context.highlight_style(Prs3dTypeOfHighlight::Selected)?`.
    pub fn highlight_style(
        &self,
        kind: Prs3dTypeOfHighlight,
    ) -> Result<AisContextHandle<'_, Prs3dDrawer>, Failure> {
        Ok(self.borrow_handle(context_style_index(kind)?, |f, i| &f.styles[i]))
    }
    /// Replace the style, also replacing the default drawer for None; `context.set_highlight_style(kind,&drawer)?`.
    pub fn set_highlight_style(
        &self,
        kind: Prs3dTypeOfHighlight,
        style: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let index = context_style_index(kind)?;
        let mut fields = self.fields.write();
        fields.styles[index] = style.clone();
        if index == 0 {
            fields.default_drawer = style.clone();
        }
        Ok(())
    }
    /// Borrow dynamic highlighting style; `context.dynamic_highlight_style()`.
    pub fn dynamic_highlight_style(&self) -> AisContextHandle<'_, Prs3dDrawer> {
        self.borrow_handle(2, |f, i| &f.styles[i])
    }
    /// Replace dynamic highlighting style; `context.set_dynamic_highlight_style(&style)`.
    pub fn set_dynamic_highlight_style(&self, style: &Handle<Prs3dDrawer>) {
        self.fields.write().styles[2] = style.clone();
    }
    /// Borrow selection highlighting style; `context.selection_style()`.
    pub fn selection_style(&self) -> AisContextHandle<'_, Prs3dDrawer> {
        self.borrow_handle(1, |f, i| &f.styles[i])
    }
    /// Replace selection highlighting style; `context.set_selection_style(&style)`.
    pub fn set_selection_style(&self, style: &Handle<Prs3dDrawer>) {
        self.fields.write().styles[1] = style.clone();
    }
    /// Update the current viewer when present; `context.update_current_viewer()`.
    pub fn update_current_viewer(&self) {
        let viewer = self.current_viewer().clone();
        if let Some(viewer) = viewer.get() {
            viewer.update();
        }
    }
    /// Return the remembered view only while it is in the viewer's active list; `context.last_active_view()`.
    pub fn last_active_view(&self) -> Handle<V3dView> {
        let fields = self.fields.read();
        let address = fields.last_active_view.object_address();
        let viewer = fields.viewer.clone();
        drop(fields);
        viewer
            .get()
            .and_then(|viewer| {
                viewer
                    .active_views()
                    .into_iter()
                    .find(|view| view.object_address() == address)
            })
            .unwrap_or_default()
    }
}
fn context_style_index(kind: Prs3dTypeOfHighlight) -> Result<usize, Failure> {
    let index = kind as usize;
    if index >= 6 {
        return Err(out_of_range(&format!(
            "highlight style {index}; expected index 0..6"
        )));
    }
    Ok(index)
}
macro_rules! context_flag {
    ($get:ident,$set:ident,$field:ident) => {
        #[doc=concat!("Read native `",stringify!($field),"`; `context.",stringify!($get),"()`.")]
        pub fn $get(&self)->bool {self.fields.read().$field}
        #[doc=concat!("Store native `",stringify!($field),"`; `context.",stringify!($set),"(true)`.")]
        pub fn $set(&self,value:bool) {self.fields.write().$field=value;}
    };
}
impl AisInteractiveContext {
    context_flag!(
        to_hilight_selected,
        set_to_hilight_selected,
        to_hilight_selected
    );
    context_flag!(automatic_hilight, set_automatic_hilight, automatic_hilight);
    context_flag!(
        get_auto_activate_selection,
        set_auto_activate_selection,
        auto_activate_selection
    );
    /// Read the detection filtering policy; `context.picking_strategy()`.
    pub fn picking_strategy(&self) -> SelectMgrPickingStrategy {
        self.fields.read().picking_strategy
    }
    /// Store the detection filtering policy; `context.set_picking_strategy(strategy)`.
    pub fn set_picking_strategy(&self, strategy: SelectMgrPickingStrategy) {
        self.fields.write().picking_strategy = strategy;
    }
}
