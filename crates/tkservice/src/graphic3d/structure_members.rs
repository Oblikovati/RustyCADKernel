//! The non-virtual accessors of `Graphic3d_Structure`: the driver structure flags, groups,
//! layer, transformation, owner and the manager notifications.
use super::{
    GRAPHIC3D_ZLAYER_ID_UNKNOWN, Graphic3dCStructure, Graphic3dCStructureMembers as _,
    Graphic3dDataStructureManager, Graphic3dDisplayPriority, Graphic3dGroup,
    Graphic3dGroupMembers as _, Graphic3dPresentationAttributesSource,
    Graphic3dSequenceOfHClipPlane, Graphic3dStructureManager, Graphic3dStructureSource,
    Graphic3dTransformPersSource, Graphic3dTypeOfStructure, Graphic3dZLayerId,
};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::TransformForm;
use rustycad_tkmath::toploc::Datum3;

/// The non-virtual members of every structure.
pub trait Graphic3dStructureMembers: Graphic3dStructureSource {
    /// `CStructure`: the driver structure, null once removed; `structure.cstructure()`.
    fn cstructure(&self) -> Handle<dyn Graphic3dCStructure> {
        self.structure_base().fields.read().cstructure.clone()
    }
    /// `StructureManager`; `structure.structure_manager()`.
    fn structure_manager(&self) -> Handle<Graphic3dStructureManager> {
        self.structure_base()
            .fields
            .read()
            .structure_manager
            .upgrade()
    }
    /// Runs `body` on the driver structure when the structure is not removed.
    fn with_cstructure<R>(&self, body: impl FnOnce(&dyn Graphic3dCStructure) -> R) -> Option<R> {
        let cstructure = self.cstructure();
        cstructure.get().map(body)
    }
    /// `DisplayPriority` (`Invalid` once removed, natively a null dereference); `structure.display_priority()`.
    fn display_priority(&self) -> Graphic3dDisplayPriority {
        self.with_cstructure(|c| c.priority())
            .unwrap_or(Graphic3dDisplayPriority::Invalid)
    }
    /// `SetInfiniteState`; `structure.set_infinite_state(true)`.
    fn set_infinite_state(&self, to_set: bool) {
        self.with_cstructure(|c| c.cstructure_base().fields.write().is_infinite = to_set);
    }
    /// `SetZLayer`: moves the displayed structure in the views, then sets the driver layer;
    /// `structure.set_z_layer(GRAPHIC3D_ZLAYER_ID_TOP)`.
    fn set_z_layer(&self, layer_id: Graphic3dZLayerId) {
        if self.is_deleted() {
            return;
        }
        if let Some(manager) = self.structure_manager().get() {
            manager.change_z_layer(&self.structure_base().this(), layer_id);
        }
        self.with_cstructure(|c| c.set_z_layer(layer_id));
    }
    /// `GetZLayer` (`UNKNOWN` once removed); `structure.get_z_layer()`.
    fn get_z_layer(&self) -> Graphic3dZLayerId {
        self.with_cstructure(|c| c.z_layer())
            .unwrap_or(GRAPHIC3D_ZLAYER_ID_UNKNOWN)
    }
    /// `SetClipPlanes`; `structure.set_clip_planes(&planes)`.
    fn set_clip_planes(&self, planes: &Handle<Graphic3dSequenceOfHClipPlane>) {
        self.with_cstructure(|c| c.set_clip_planes(planes));
    }
    /// `ClipPlanes`; `structure.clip_planes()`.
    fn clip_planes(&self) -> Handle<Graphic3dSequenceOfHClipPlane> {
        self.with_cstructure(|c| c.clip_planes())
            .unwrap_or_default()
    }
    /// `SetIsForHighlight`; `structure.set_is_for_highlight(true)`.
    fn set_is_for_highlight(&self, value: bool) {
        self.with_cstructure(|c| c.cstructure_base().fields.write().is_for_highlight = value);
    }
    /// `ReCompute()`: recomputes the structure in every view; `structure.re_compute()`.
    fn re_compute(&self) {
        if let Some(manager) = self.structure_manager().get() {
            manager.re_compute(&self.structure_base().this());
        }
    }
    /// `ReCompute(projector)`: recomputes the structure in one view; `structure.re_compute_in(&view)`.
    fn re_compute_in(&self, projector: &Handle<dyn Graphic3dDataStructureManager>) {
        if let Some(manager) = self.structure_manager().get() {
            manager.re_compute_in(&self.structure_base().this(), projector);
        }
    }
    /// `Groups`; `structure.groups().len()`.
    fn groups(&self) -> Vec<Handle<dyn Graphic3dGroup>> {
        self.with_cstructure(|c| c.groups()).unwrap_or_default()
    }
    /// `NumberOfGroups`; `structure.number_of_groups()`.
    fn number_of_groups(&self) -> i32 {
        self.groups().len() as i32
    }
    /// `NewGroup`: a new group made by the driver structure; `structure.new_group()?`.
    fn new_group(&self) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        let this = self.structure_base().this();
        let cstructure = self.cstructure();
        let Some(cstructure) = cstructure.get() else {
            return Err(program_error(
                "Graphic3d_Structure::NewGroup: the structure is removed; expected a live structure",
            ));
        };
        cstructure.new_group(&this)
    }
    /// `CurrentGroup`: the last group, or a new one; `structure.current_group()?`.
    fn current_group(&self) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        match self.groups().last() {
            Some(group) => Ok(group.clone()),
            None => self.new_group(),
        }
    }
    /// `HighlightStyle`; `structure.highlight_style()`.
    fn highlight_style(&self) -> Handle<dyn Graphic3dPresentationAttributesSource> {
        self.with_cstructure(|c| c.highlight_style())
            .unwrap_or_default()
    }
    /// `IsDeleted`: the driver structure is gone; `structure.is_deleted()`.
    fn is_deleted(&self) -> bool {
        self.cstructure().is_null()
    }
    /// `IsInfinite`: deleted or infinite; `structure.is_infinite()`.
    fn is_infinite(&self) -> bool {
        self.with_cstructure(|c| c.cstructure_base().fields.read().is_infinite)
            .unwrap_or(true)
    }
    /// `IsTransformed`: a non-identity transformation; `structure.is_transformed()`.
    fn is_transformed(&self) -> bool {
        let trsf = self
            .with_cstructure(|c| c.transformation())
            .unwrap_or_default();
        trsf.get()
            .is_some_and(|t| t.form() != TransformForm::Identity)
    }
    /// `IsVisible`; `structure.is_visible()`.
    fn is_visible(&self) -> bool {
        self.with_cstructure(|c| c.is_visible()).unwrap_or(false)
    }
    /// `IsEmpty`: every group and every descendant is empty; `structure.is_empty()`.
    fn is_empty(&self) -> bool {
        if self.is_deleted() {
            return true;
        }
        if self
            .groups()
            .iter()
            .filter_map(Handle::get)
            .any(|g| !g.is_empty())
        {
            return false;
        }
        let descendants = self.structure_base().fields.read().descendants.clone();
        descendants
            .iter()
            .map(|d| d.upgrade())
            .all(|d| d.get().is_none_or(|d| d.is_empty()))
    }
    /// `Visual`; `structure.visual()`.
    fn visual(&self) -> Graphic3dTypeOfStructure {
        self.structure_base().fields.read().visual
    }
    /// `SetOwner`; `structure.set_owner(&owner)`.
    fn set_owner(&self, owner: &Handle<StandardTransient>) {
        self.structure_base().fields.write().owner = owner.downgrade();
    }
    /// `Owner`; `structure.owner()`.
    fn owner(&self) -> Handle<StandardTransient> {
        self.structure_base().fields.read().owner.upgrade()
    }
    /// `SetHLRValidation`; `structure.set_hlr_validation(true)`.
    fn set_hlr_validation(&self, flag: bool) {
        self.with_cstructure(|c| c.cstructure_base().fields.write().hlr_validation = flag);
    }
    /// `HLRValidation`: an owner and the flag; `structure.hlr_validation()`.
    fn hlr_validation(&self) -> bool {
        let has_owner = !self.structure_base().fields.read().owner.is_null();
        has_owner
            && self
                .with_cstructure(|c| c.cstructure_base().fields.read().hlr_validation)
                .unwrap_or(false)
    }
    /// `Transformation`; `structure.transformation()`.
    fn transformation(&self) -> Handle<Datum3> {
        self.with_cstructure(|c| c.transformation())
            .unwrap_or_default()
    }
    /// `SetTransformation`: stores the transformation (null for the identity), recomputes a
    /// structure that is or was transformed, notifies the views and updates the layer;
    /// `structure.set_transformation(&datum)`.
    fn set_transformation(&self, trsf: &Handle<Datum3>) {
        if self.is_deleted() {
            return;
        }
        let was_transformed = self.is_transformed();
        let identity = trsf
            .get()
            .is_some_and(|t| t.trsf().form() == TransformForm::Identity);
        let stored = if identity {
            Handle::default()
        } else {
            trsf.clone()
        };
        self.with_cstructure(|c| c.set_transformation(&stored));
        if self.is_transformed() || was_transformed {
            self.re_compute();
        }
        if let Some(manager) = self.structure_manager().get() {
            manager.set_transform(&self.structure_base().this(), trsf);
        }
        self.update(true);
    }
    /// `SetTransformPersistence`; `structure.set_transform_persistence(&pers)`.
    fn set_transform_persistence(&self, trsf_pers: &Handle<dyn Graphic3dTransformPersSource>) {
        self.with_cstructure(|c| c.set_transform_persistence(trsf_pers));
    }
    /// `TransformPersistence`; `structure.transform_persistence()`.
    fn transform_persistence(&self) -> Handle<dyn Graphic3dTransformPersSource> {
        self.with_cstructure(|c| c.transform_persistence())
            .unwrap_or_default()
    }
    /// `SetMutable`; `structure.set_mutable(true)`.
    fn set_mutable(&self, is_mutable: bool) {
        self.with_cstructure(|c| c.cstructure_base().fields.write().is_mutable = is_mutable);
    }
    /// `IsMutable`; `structure.is_mutable()`.
    fn is_mutable(&self) -> bool {
        self.with_cstructure(|c| c.cstructure_base().fields.read().is_mutable)
            .unwrap_or(false)
    }
    /// `ComputeVisual`; `structure.compute_visual()`.
    fn compute_visual(&self) -> Graphic3dTypeOfStructure {
        self.structure_base().fields.read().compute_visual
    }
    /// `SetComputeVisual`: ignores the computed visual; `structure.set_compute_visual(Graphic3dTypeOfStructure::Shading)`.
    fn set_compute_visual(&self, visual: Graphic3dTypeOfStructure) {
        if visual != Graphic3dTypeOfStructure::Computed {
            self.structure_base().fields.write().compute_visual = visual;
        }
    }
    /// `GraphicConnect`: connects the driver structures; `structure.graphic_connect(&child)`.
    fn graphic_connect(&self, daughter: &Handle<dyn Graphic3dStructureSource>) {
        let other = daughter.get().map(|d| d.cstructure()).unwrap_or_default();
        if let Some(other) = other.get() {
            self.with_cstructure(|c| c.connect(other));
        }
    }
    /// `GraphicDisconnect`: disconnects the driver structures; `structure.graphic_disconnect(&child)`.
    fn graphic_disconnect(&self, daughter: &Handle<dyn Graphic3dStructureSource>) {
        let other = daughter.get().map(|d| d.cstructure()).unwrap_or_default();
        if let Some(other) = other.get() {
            self.with_cstructure(|c| c.disconnect(other));
        }
    }
    /// `GraphicTransform`: the driver transformation only; `structure.graphic_transform(&datum)`.
    fn graphic_transform(&self, trsf: &Handle<Datum3>) {
        self.with_cstructure(|c| c.set_transformation(trsf));
    }
    /// `Identification` (-1 once removed); `structure.identification()`.
    fn identification(&self) -> i32 {
        self.with_cstructure(|c| c.identification()).unwrap_or(-1)
    }
    /// `Update`: updates the views (the structure layer when asked); `structure.update(true)`.
    fn update(&self, update_layer: bool) {
        let Some(layer) = self.with_cstructure(|c| c.z_layer()) else {
            return;
        };
        if let Some(manager) = self.structure_manager().get() {
            manager.update(if update_layer {
                layer
            } else {
                GRAPHIC3D_ZLAYER_ID_UNKNOWN
            });
        }
    }
    /// `Remove(const handle<Graphic3d_Group>&)`: removes the group at `address` when it
    /// belongs to this structure and detaches it; `structure.remove_group_at(address)`.
    fn remove_group_at(&self, address: *const ()) {
        let groups = self.groups();
        let Some(group) = groups.iter().find(|g| g.object_address() == address) else {
            return;
        };
        let owner = group
            .get()
            .map(|g| g.group_base().fields.read().structure.object_address());
        if owner != Some(super::structure_network::address_of(self)) {
            return;
        }
        self.with_cstructure(|c| c.remove_group(group));
        if let Some(group) = group.get() {
            group.group_base().fields.write().structure = Default::default();
        }
    }
}
impl<T: Graphic3dStructureSource + ?Sized> Graphic3dStructureMembers for T {}
