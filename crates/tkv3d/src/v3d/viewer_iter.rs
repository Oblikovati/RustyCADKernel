//! The deprecated iteration members of `V3d_Viewer` (`InitActiveViews`...). The native
//! iterators are list iterators kept in the viewer; here each is a position in its list.
//! Native quirk kept: `Next*` does nothing on an empty list, and `ActiveView` past the end
//! gives a null handle (natively undefined behavior).
use super::V3dViewer;
use crate::v3d::V3dViewSource;
use rustycad_tkernel::standard::{Handle, HandleReference as _};
use rustycad_tkservice::graphic3d::Graphic3dCLight;

/// The cursor slots of the four deprecated iterations.
const ACTIVE_VIEWS: usize = 0;
const DEFINED_VIEWS: usize = 1;
const ACTIVE_LIGHTS: usize = 2;
const DEFINED_LIGHTS: usize = 3;

impl V3dViewer {
    fn cursor(&self, slot: usize) -> usize {
        self.state.read().iterators[slot]
    }
    fn set_cursor(&self, slot: usize, value: usize) {
        self.state.write().iterators[slot] = value;
    }
    fn advance(&self, slot: usize, list_len: usize) {
        if list_len != 0 {
            self.set_cursor(slot, self.cursor(slot) + 1);
        }
    }
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
    /// Deprecated `InitActiveLights`; `viewer.init_active_lights()`.
    pub fn init_active_lights(&self) {
        self.set_cursor(ACTIVE_LIGHTS, 0);
    }
    /// Deprecated `MoreActiveLights`; `viewer.more_active_lights()`.
    pub fn more_active_lights(&self) -> bool {
        self.cursor(ACTIVE_LIGHTS) < self.state.read().active_lights.len()
    }
    /// Deprecated `NextActiveLights` (advances even on an empty list, as natively).
    /// Example: `viewer.next_active_lights()`.
    pub fn next_active_lights(&self) {
        self.set_cursor(ACTIVE_LIGHTS, self.cursor(ACTIVE_LIGHTS) + 1);
    }
    /// Deprecated `ActiveLight`; `viewer.active_light()`.
    pub fn active_light(&self) -> Handle<Graphic3dCLight> {
        let index = self.cursor(ACTIVE_LIGHTS);
        self.state
            .read()
            .active_lights
            .get(index)
            .cloned()
            .unwrap_or_default()
    }
    /// Deprecated `InitDefinedLights`; `viewer.init_defined_lights()`.
    pub fn init_defined_lights(&self) {
        self.set_cursor(DEFINED_LIGHTS, 0);
    }
    /// Deprecated `MoreDefinedLights`; `viewer.more_defined_lights()`.
    pub fn more_defined_lights(&self) -> bool {
        self.cursor(DEFINED_LIGHTS) < self.state.read().defined_lights.len()
    }
    /// Deprecated `NextDefinedLights`; `viewer.next_defined_lights()`.
    pub fn next_defined_lights(&self) {
        let len = self.state.read().defined_lights.len();
        self.advance(DEFINED_LIGHTS, len);
    }
    /// Deprecated `DefinedLight`; `viewer.defined_light()`.
    pub fn defined_light(&self) -> Handle<Graphic3dCLight> {
        let index = self.cursor(DEFINED_LIGHTS);
        self.state
            .read()
            .defined_lights
            .get(index)
            .cloned()
            .unwrap_or_default()
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
    pub fn active_light_iterator(&self) -> std::vec::IntoIter<Handle<Graphic3dCLight>> {
        self.active_lights().into_iter()
    }
    /// `DefinedLightIterator`; `viewer.defined_light_iterator()`.
    pub fn defined_light_iterator(&self) -> std::vec::IntoIter<Handle<Graphic3dCLight>> {
        self.defined_lights().into_iter()
    }
}
