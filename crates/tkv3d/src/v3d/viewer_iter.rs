//! Deprecated V3d_Viewer iteration retains actual list-node cursors in the viewer.
//! Native empty-list Next guards preserve the cursor; invalid end-node references have no
//! native success contract and produce Martensite's typed diagnostic rather than a copied null.
use super::V3dViewer;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::standard::{Handle, HandleReference as _};
use rustycad_tkservice::graphic3d::Graphic3dCLight;

/// Native view cursor slots remain distinct from the light cursor storage.
const ACTIVE_VIEWS: usize = 0;
const DEFINED_VIEWS: usize = 1;

impl V3dViewer {
    /// Deprecated native first-node initialization; `viewer.init_active_views()`.
    pub fn init_active_views(&self) {
        let mut state = self.state.write();
        state.view_cursors[ACTIVE_VIEWS] = state.active_views.cursor();
    }
    /// Deprecated native node check; `viewer.more_active_views()`.
    pub fn more_active_views(&self) -> bool {
        self.state.read().view_cursors[ACTIVE_VIEWS].more()
    }
    /// Deprecated native advance after callbacks; `viewer.next_active_views()`.
    pub fn next_active_views(&self) {
        let mut state = self.state.write();
        let mut cursor = state.view_cursors[ACTIVE_VIEWS];
        cursor.next(&state.active_views);
        state.view_cursors[ACTIVE_VIEWS] = cursor;
    }
    /// Explicit owning snapshot convenience; native ActiveView binds active_view_ref; `viewer.active_view()`.
    pub fn active_view(&self) -> Handle<dyn V3dViewSource> {
        self.active_view_ref().clone_handle()
    }
    /// Deprecated native first-node initialization; `viewer.init_defined_views()`.
    pub fn init_defined_views(&self) {
        let mut state = self.state.write();
        state.view_cursors[DEFINED_VIEWS] = state.defined_views.cursor();
    }
    /// Deprecated native node check; `viewer.more_defined_views()`.
    pub fn more_defined_views(&self) -> bool {
        self.state.read().view_cursors[DEFINED_VIEWS].more()
    }
    /// Deprecated native advance after callbacks; `viewer.next_defined_views()`.
    pub fn next_defined_views(&self) {
        let mut state = self.state.write();
        let mut cursor = state.view_cursors[DEFINED_VIEWS];
        cursor.next(&state.defined_views);
        state.view_cursors[DEFINED_VIEWS] = cursor;
    }
    /// Explicit owning snapshot convenience; native DefinedView binds defined_view_ref; `viewer.defined_view()`.
    pub fn defined_view(&self) -> Handle<dyn V3dViewSource> {
        self.defined_view_ref().clone_handle()
    }
    /// Initialize native active-light node; `viewer.init_active_lights()`.
    pub fn init_active_lights(&self) {
        let mut fields = self.state.write();
        fields.light_cursors[0] = fields.active_lights.cursor();
    }
    /// Test native active-light node; `viewer.more_active_lights()`.
    pub fn more_active_lights(&self) -> bool {
        self.state.read().light_cursors[0].more()
    }
    /// Advance native active-light node; `viewer.next_active_lights()`.
    pub fn next_active_lights(&self) {
        let mut fields = self.state.write();
        let mut cursor = fields.light_cursors[0];
        cursor.next(&fields.active_lights);
        fields.light_cursors[0] = cursor;
    }
    /// Explicit owning convenience; native ActiveLight binds active_light_ref; `viewer.active_light()`.
    pub fn active_light(&self) -> Handle<Graphic3dCLight> {
        self.active_light_ref().clone_handle()
    }
    /// Initialize native defined-light node; `viewer.init_defined_lights()`.
    pub fn init_defined_lights(&self) {
        let mut fields = self.state.write();
        fields.light_cursors[1] = fields.defined_lights.cursor();
    }
    /// Test native defined-light node; `viewer.more_defined_lights()`.
    pub fn more_defined_lights(&self) -> bool {
        self.state.read().light_cursors[1].more()
    }
    /// Advance native defined-light node; `viewer.next_defined_lights()`.
    pub fn next_defined_lights(&self) {
        let mut fields = self.state.write();
        let mut cursor = fields.light_cursors[1];
        cursor.next(&fields.defined_lights);
        fields.light_cursors[1] = cursor;
    }
    /// Explicit owning convenience; native DefinedLight binds defined_light_ref; `viewer.defined_light()`.
    pub fn defined_light(&self) -> Handle<Graphic3dCLight> {
        self.defined_light_ref().clone_handle()
    }
    /// `ActiveViewIterator`: the active views to iterate; `viewer.active_view_iterator()`.
    pub fn active_view_iterator(&self) -> super::V3dViewCursor<'_> {
        self.active_views_ref().cursor()
    }
    /// `DefinedViewIterator`; `viewer.defined_view_iterator()`.
    pub fn defined_view_iterator(&self) -> super::V3dViewCursor<'_> {
        self.defined_views_ref().cursor()
    }
    /// `ActiveLightIterator`; `viewer.active_light_iterator()`.
    pub fn active_light_iterator(&self) -> super::V3dLightCursor<'_> {
        self.active_lights_ref().cursor()
    }
    /// `DefinedLightIterator`; `viewer.defined_light_iterator()`.
    pub fn defined_light_iterator(&self) -> super::V3dLightCursor<'_> {
        self.defined_lights_ref().cursor()
    }
}
