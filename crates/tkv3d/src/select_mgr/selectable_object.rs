//! `SelectMgr_SelectableObject`: a presentable object with selection modes (v3d-design.md
//! section 6). The abstract class is the [`SelectMgrSelectableObject`] trait (a sub-trait of
//! [`PrsMgrPresentableObject`]) over [`SelectMgrSelectableObjectBase`]. Its overrides of the
//! presentable virtual members are the `select_mgr_*` functions, which a concrete type puts in
//! its `PrsMgrPresentableObject` implementation with [`select_mgr_presentable_overrides`].
use super::{
    SelectMgrEntityOwnerSource, SelectMgrSelection, SelectMgrTypeOfBvhUpdate, SelectMgrTypeOfUpdate,
};
use crate::prs_mgr::{
    PrsMgrPresentableObject, PrsMgrPresentableObjectMembers as _, PrsMgrPresentationManager,
    prs_mgr_presentable_dump_json, prs_mgr_set_z_layer, prs_mgr_update_clipping,
    prs_mgr_update_transformation,
};
use crate::prs3d::Prs3dDrawer;
use crate::select3d::Select3DSensitiveEntityMembers as _;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::not_implemented;
use rustycad_tkernel::standard::{Failure, Handle, SharedState};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::{
    Graphic3dStructureMembers as _, Graphic3dStructureSource, Graphic3dZLayerId,
};
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;

/// The protected fields of `SelectMgr_SelectableObject`.
#[derive(Clone)]
pub struct SelectMgrSelectableObjectFields {
    /// `myselections`.
    pub selections: Vec<Handle<SelectMgrSelection>>,
    /// `mySelectionPrs`.
    pub selection_prs: Handle<dyn Graphic3dStructureSource>,
    /// `myHilightPrs`.
    pub hilight_prs: Handle<dyn Graphic3dStructureSource>,
    /// `myGlobalSelMode`.
    pub global_sel_mode: i32,
    /// `myAutoHilight`.
    pub auto_hilight: bool,
}

/// The selection state every selectable object embeds (next to its presentable base).
pub struct SelectMgrSelectableObjectBase {
    /// The protected fields.
    pub fields: SharedState<SelectMgrSelectableObjectFields>,
}

impl Default for SelectMgrSelectableObjectBase {
    /// The constructor state: no selection, global mode 0, auto-highlighted.
    fn default() -> Self {
        let fields = SelectMgrSelectableObjectFields {
            selections: Vec::new(),
            selection_prs: Handle::default(),
            hilight_prs: Handle::default(),
            global_sel_mode: 0,
            auto_hilight: true,
        };
        Self {
            fields: SharedState::new(fields),
        }
    }
}

impl SelectMgrSelectableObjectBase {
    /// A copy of the fields; `base.read()`.
    pub fn read(&self) -> SelectMgrSelectableObjectFields {
        self.fields.read().clone()
    }
}

impl Drop for SelectMgrSelectableObjectBase {
    /// The destructor clears every selection.
    fn drop(&mut self) {
        let selections = self.fields.read().selections.clone();
        selections
            .iter()
            .filter_map(Handle::get)
            .for_each(SelectMgrSelection::clear);
    }
}

/// A presentable object with selection modes; the default bodies are the native ones.
pub trait SelectMgrSelectableObject: PrsMgrPresentableObject {
    /// The embedded selection state; `object.selectable_base()`.
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase;
    /// `ComputeSelection` (pure virtual): fills the selection of a mode with sensitive entities.
    /// Example: `object.compute_selection(&selection, 0)?`.
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure>;
    /// `AcceptShapeDecomposition`: false by default; `object.accept_shape_decomposition()`.
    fn accept_shape_decomposition(&self) -> bool {
        false
    }
    /// `UpdateTransformations`: the owners of the selection take the object location.
    /// Example: `object.update_transformations(&selection)`.
    fn update_transformations(&self, selection: &Handle<SelectMgrSelection>) {
        let location = Location::new(self.transformation());
        for entity in selection
            .get()
            .map(SelectMgrSelection::entities)
            .unwrap_or_default()
            .iter()
            .filter_map(Handle::get)
        {
            let owner = entity
                .base_sensitive()
                .get()
                .map(|s| s.owner_id())
                .unwrap_or_default();
            if let Some(owner) = owner.get() {
                owner.set_location(&location);
            }
        }
    }
    /// `HilightSelected`: raises `Standard_NotImplemented` by default.
    /// Example: `object.hilight_selected(&manager, &owners)?`.
    fn hilight_selected(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    ) -> Result<(), Failure> {
        let _ = (manager, owners);
        Err(not_implemented(
            "SelectMgr_SelectableObject::HilightSelected",
        ))
    }
    /// `ClearSelected`: clears the selection presentation; `object.clear_selected()`.
    fn clear_selected(&self) {
        if let Some(prs) = self.selectable_base().read().selection_prs.get() {
            prs.clear(true);
        }
    }
    /// `ClearDynamicHighlight`: clears the immediate draw of the manager; `object.clear_dynamic_highlight(&manager)`.
    fn clear_dynamic_highlight(&self, manager: &Handle<PrsMgrPresentationManager>) {
        if let Some(manager) = manager.get() {
            manager.clear_immediate_draw();
        }
    }
    /// `HilightOwnerWithColor`: raises `Standard_NotImplemented` by default.
    /// Example: `object.hilight_owner_with_color(&manager, &style, &owner)?`.
    fn hilight_owner_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        let _ = (manager, style, owner);
        Err(not_implemented(
            "SelectMgr_SelectableObject::HilightOwnerWithColor",
        ))
    }
    /// `IsAutoHilight`; `object.is_auto_hilight()`.
    fn is_auto_hilight(&self) -> bool {
        self.selectable_base().read().auto_hilight
    }
    /// `SetAutoHilight`; `object.set_auto_hilight(false)`.
    fn set_auto_hilight(&self, auto_hilight: bool) {
        self.selectable_base().fields.write().auto_hilight = auto_hilight;
    }
    /// `ErasePresentations`: erases the selection and highlight presentations, and forgets
    /// them when asked. Example: `object.erase_presentations(true)`.
    fn erase_presentations(&self, to_remove: bool) {
        let fields = self.selectable_base().read();
        for prs in [&fields.selection_prs, &fields.hilight_prs]
            .into_iter()
            .filter_map(Handle::get)
        {
            prs.erase();
            if to_remove {
                prs.clear(true);
            }
        }
        if to_remove {
            let mut own = self.selectable_base().fields.write();
            (own.selection_prs, own.hilight_prs) = (Handle::default(), Handle::default());
        }
    }
    /// `GlobalSelOwner`: the owner of the first entity of the global selection mode.
    /// Example: `object.global_sel_owner()`.
    fn global_sel_owner(&self) -> Handle<dyn SelectMgrEntityOwnerSource> {
        let mode = self.selectable_base().read().global_sel_mode;
        let selection = select_mgr_selection_of(self, mode);
        let first = selection
            .get()
            .and_then(|s| s.entities().first().cloned())
            .unwrap_or_default();
        first
            .get()
            .and_then(|e| e.base_sensitive().get().map(|s| s.owner_id()))
            .unwrap_or_default()
    }
    /// `GetAssemblyOwner`: null by default; `object.get_assembly_owner()`.
    fn get_assembly_owner(&self) -> Handle<dyn SelectMgrEntityOwnerSource> {
        Handle::default()
    }
    /// `updateSelection` (protected): flags the selection of the mode (all for -1) for a full update.
    /// Example: `object.update_selection_internal(-1)`.
    fn update_selection_internal(&self, mode: i32) {
        for selection in self
            .selectable_base()
            .read()
            .selections
            .iter()
            .filter_map(Handle::get)
        {
            if mode == -1 || selection.mode() == mode {
                selection.set_update_status(SelectMgrTypeOfUpdate::Full);
                if mode != -1 {
                    return;
                }
            }
        }
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn SelectMgrSelectableObject,
    "SelectMgr_SelectableObject",
    dyn PrsMgrPresentableObject
);

/// `Selection(mode)`: the selection of a mode, null for -1 or an unknown mode.
/// Example: `select_mgr_selection_of(&*object, 0)`.
pub fn select_mgr_selection_of<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
    mode: i32,
) -> Handle<SelectMgrSelection> {
    if mode == -1 {
        return Handle::default();
    }
    let selections = object.selectable_base().read().selections;
    selections
        .into_iter()
        .find(|s| s.get().is_some_and(|v| v.mode() == mode))
        .unwrap_or_default()
}

/// Flags every selection for a partial update.
fn flag_partial<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
    bvh: Option<SelectMgrTypeOfBvhUpdate>,
) {
    for selection in object
        .selectable_base()
        .read()
        .selections
        .iter()
        .filter_map(Handle::get)
    {
        selection.set_update_status(SelectMgrTypeOfUpdate::Partial);
        if let Some(status) = bvh {
            selection.update_bvh_status(status);
        }
    }
}

/// `SelectMgr_SelectableObject::ResetTransformation`; `select_mgr_reset_transformation(&*object)?`.
pub fn select_mgr_reset_transformation<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    flag_partial(object, Some(SelectMgrTypeOfBvhUpdate::None));
    object.set_local_transformation_datum(&Handle::default())
}

/// `SelectMgr_SelectableObject::UpdateTransformation`: the presentable update, then the
/// selection and highlight presentations follow. Example: `select_mgr_update_transformation(&*object)?`.
pub fn select_mgr_update_transformation<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    flag_partial(object, None);
    prs_mgr_update_transformation(object)?;
    let (fields, trsf) = (
        object.selectable_base().read(),
        object.transformation_geom(),
    );
    for prs in [&fields.selection_prs, &fields.hilight_prs]
        .into_iter()
        .filter_map(Handle::get)
    {
        prs.set_transformation(&trsf);
    }
    Ok(())
}

/// `SelectMgr_SelectableObject::SetZLayer`: the presentable layer, the selection and
/// highlight presentations and the owners. Example: `select_mgr_set_z_layer(&*object, layer)`.
pub fn select_mgr_set_z_layer<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
    layer_id: Graphic3dZLayerId,
) {
    prs_mgr_set_z_layer(object, layer_id);
    let fields = object.selectable_base().read();
    for prs in [&fields.selection_prs, &fields.hilight_prs]
        .into_iter()
        .filter_map(Handle::get)
    {
        prs.set_z_layer(layer_id);
    }
    for selection in fields.selections.iter().filter_map(Handle::get) {
        for entity in selection.entities().iter().filter_map(Handle::get) {
            let owner = entity
                .base_sensitive()
                .get()
                .map(|s| s.owner_id())
                .unwrap_or_default();
            if let Some(owner) = owner.get() {
                owner.set_z_layer(layer_id);
            }
        }
    }
}

/// `SelectMgr_SelectableObject::UpdateClipping`; `select_mgr_update_clipping(&*object)`.
pub fn select_mgr_update_clipping<O: SelectMgrSelectableObject + ?Sized>(object: &O) {
    prs_mgr_update_clipping(object);
    let (fields, planes) = (object.selectable_base().read(), object.clip_planes());
    for prs in [&fields.selection_prs, &fields.hilight_prs]
        .into_iter()
        .filter_map(Handle::get)
    {
        prs.set_clip_planes(&planes);
    }
}

/// `SelectMgr_SelectableObject::DumpJson`; `select_mgr_selectable_dump_json(&*object, &mut stream, -1)?`.
pub fn select_mgr_selectable_dump_json<O: SelectMgrSelectableObject + ?Sized>(
    object: &O,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, "SelectMgr_SelectableObject")?;
    base_class(stream, depth, &|s, d| {
        prs_mgr_presentable_dump_json(object, s, d)
    })?;
    let fields = object.selectable_base().read();
    if let Some(prs) = fields.selection_prs.get() {
        field_dumped(stream, depth, "mySelectionPrs.get()", &|s, d| {
            prs.dump_json(s, d)
        })?;
    }
    if let Some(prs) = fields.hilight_prs.get() {
        field_dumped(stream, depth, "myHilightPrs.get()", &|s, d| {
            prs.dump_json(s, d)
        })?;
    }
    for selection in fields.selections.iter().filter_map(Handle::get) {
        field_dumped(stream, depth, "aSelection.get()", &|s, d| {
            selection.dump_json(s, d)
        })?;
    }
    field_integer(stream, "myGlobalSelMode", fields.global_sel_mode)?;
    field_integer(stream, "myAutoHilight", i32::from(fields.auto_hilight))
}

/// The `PrsMgrPresentableObject` overrides of `SelectMgr_SelectableObject`, to put in the
/// presentable implementation of a selectable type:
/// `impl PrsMgrPresentableObject for MyObject { ...; select_mgr_presentable_overrides!(); }`.
#[macro_export]
macro_rules! select_mgr_presentable_overrides {
    () => {
        fn reset_transformation(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::select_mgr::select_mgr_reset_transformation(self)
        }
        fn update_transformation(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::select_mgr::select_mgr_update_transformation(self)
        }
        fn set_z_layer(&self, layer_id: rustycad_tkservice::graphic3d::Graphic3dZLayerId) {
            $crate::select_mgr::select_mgr_set_z_layer(self, layer_id);
        }
        fn update_clipping(&self) {
            $crate::select_mgr::select_mgr_update_clipping(self);
        }
        fn dump_json(
            &self,
            stream: &mut rustycad_tkernel::standard::dump::DumpStream,
            depth: i32,
        ) -> std::io::Result<()> {
            $crate::select_mgr::select_mgr_selectable_dump_json(self, stream, depth)
        }
    };
}
