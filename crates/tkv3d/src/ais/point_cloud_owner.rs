//! Forced subset owners snapshot native detection maps into independent highlight sets.
use super::AisPointCloudSource;
use crate::{
    prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentationManager},
    prs3d::Prs3dDrawer,
    select_mgr::*,
    select3d::{Select3DSensitiveEntityMembers, Select3DSensitivePrimitiveArray},
};
use rustycad_tkernel::{
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcol_std::HPackedMapOfInteger,
};
use rustycad_tkservice::graphic3d::*;
/// Native protected subset handles, replaceable by derived owners; `base.fields.write().selected = replacement`.
pub struct AisPointCloudOwnerFields {
    /// Shared zero-based last detected subset.
    pub detected: Handle<HPackedMapOfInteger>,
    /// Shared zero-based selected subset.
    pub selected: Handle<HPackedMapOfInteger>,
}
/// Native protected owner storage for derived subset owners; `AisPointCloudOwnerBase::new(&cloud.cast())`.
pub struct AisPointCloudOwnerBase {
    /// Inherited selectable/priority state.
    pub owner: SelectMgrEntityOwnerBase,
    /// Replaceable native protected handles.
    pub fields: SharedState<AisPointCloudOwnerFields>,
}
impl AisPointCloudOwnerBase {
    /// Retain independent maps and priority five; `AisPointCloudOwnerBase::new(&cloud.cast())`.
    pub fn new(origin: &Handle<dyn AisPointCloudSource>) -> Self {
        Self {
            owner: SelectMgrEntityOwnerBase::new(&origin.cast(), 5),
            fields: SharedState::new(AisPointCloudOwnerFields {
                detected: Handle::new(HPackedMapOfInteger::default()),
                selected: Handle::new(HPackedMapOfInteger::default()),
            }),
        }
    }
}
/// Native subset owner interface with overridable inherited highlight hooks; `owner.selected_points()`.
pub trait AisPointCloudOwnerSource: SelectMgrEntityOwnerSource {
    /// Embedded protected storage; `owner.point_cloud_owner_base()`.
    fn point_cloud_owner_base(&self) -> &AisPointCloudOwnerBase;
    /// Stable shared selected set, indexed from zero; `owner.selected_points().map().contains(0)`.
    fn selected_points(&self) -> Handle<HPackedMapOfInteger> {
        self.point_cloud_owner_base().fields.read().selected.clone()
    }
    /// Stable shared detected set, indexed from zero; `owner.detected_points().map().contains(0)`.
    fn detected_points(&self) -> Handle<HPackedMapOfInteger> {
        self.point_cloud_owner_base().fields.read().detected.clone()
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisPointCloudOwnerSource,
    "AIS_PointCloudOwner",
    dyn SelectMgrEntityOwnerSource
);
/// Selected/detected point indices are zero-based; `AisPointCloudOwner::new(&cloud.cast())`.
pub struct AisPointCloudOwner {
    transient: StandardTransient,
    base: AisPointCloudOwnerBase,
}
rustycad_tkernel::impl_transient!(AisPointCloudOwner,"AIS_PointCloudOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn AisPointCloudOwnerSource,dyn SelectMgrEntityOwnerSource]);
impl AisPointCloudOwner {
    /// Bind an owner with priority five and independent maps; `AisPointCloudOwner::new(&cloud.cast())`.
    pub fn new(origin: &Handle<dyn AisPointCloudSource>) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisPointCloudOwnerBase::new(origin),
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
}
impl AisPointCloudOwnerSource for AisPointCloudOwner {
    fn point_cloud_owner_base(&self) -> &AisPointCloudOwnerBase {
        &self.base
    }
}
fn cloud_origin<O: AisPointCloudOwnerSource + ?Sized>(
    owner: &O,
) -> Result<Handle<dyn AisPointCloudSource>, Failure> {
    let origin = owner.selectable().cast::<dyn AisPointCloudSource>();
    if origin.is_null() {
        return Err(Failure::with_type(
            "Standard_ProgramError",
            &format!(
                "AIS_PointCloudOwner selectable {:?}: expected AIS_PointCloud",
                owner.selectable().object_address()
            ),
            "",
        ));
    }
    Ok(origin)
}
fn refresh_cloud_subset<O: AisPointCloudOwnerSource + ?Sized>(
    owner: &O,
    origin: &Handle<dyn AisPointCloudSource>,
    subset: &Handle<HPackedMapOfInteger>,
) {
    subset.change_map().clear();
    for selection in origin.selections() {
        copy_selection_subset(owner, &selection, subset);
    }
}
fn copy_selection_subset<O: AisPointCloudOwnerSource + ?Sized>(
    owner: &O,
    selection: &Handle<SelectMgrSelection>,
    subset: &Handle<HPackedMapOfInteger>,
) {
    for entity in selection.entities() {
        let sensitive = entity.base_sensitive();
        if sensitive.owner_id() != owner.owner_base().this() {
            continue;
        }
        let array = sensitive.cast::<Select3DSensitivePrimitiveArray>();
        if array.is_null() {
            continue;
        }
        *subset.change_map() = array.last_detected_element_map().map().clone();
        if array.last_detected_element() != -1 {
            subset.change_map().add(array.last_detected_element());
        }
        break;
    }
}
fn draw_cloud_subset(
    origin: &Handle<dyn AisPointCloudSource>,
    subset: &Handle<HPackedMapOfInteger>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    style: &Handle<Prs3dDrawer>,
) -> Result<bool, Failure> {
    if subset.map().is_empty() {
        return Ok(false);
    }
    let all_points = origin.get_points();
    if all_points.is_null() {
        return Ok(false);
    }
    let points = Handle::new(Graphic3dArrayOfPoints::new(subset.map().extent(), 0)?);
    for index in subset.map().iter() {
        points.add_vertex_pnt(&all_points.vertice(index + 1)?)?;
    }
    let group = prs.new_group()?;
    group.set_group_primitives_aspect(&style.point_aspect().aspect().cast());
    group.add_primitive_array(&points.cast(), true);
    Ok(true)
}
impl SelectMgrEntityOwnerSource for AisPointCloudOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base.owner
    }
    fn is_forced_hilight(&self) -> bool {
        true
    }
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        _: i32,
    ) -> Result<(), Failure> {
        ais_point_cloud_owner_hilight_with_color(self, manager, style)
    }
    fn clear(&self, manager: &Handle<PrsMgrPresentationManager>, mode: i32) {
        // OCCT 3d097a0 delegates to the base owner's no-op Clear, preserving subset maps.
        let _ = (manager, mode);
    }
    fn unhilight(&self, _: &Handle<PrsMgrPresentationManager>, _: i32) {
        let origin = self.selectable();
        let Some(origin) = origin.get() else { return };
        let Ok(prs) = origin.get_select_presentation(&Handle::default()) else {
            return;
        };
        if let Some(prs) = prs.get() {
            prs.erase();
        }
    }
}

fn prepare_cloud_subset_layer(
    origin: &Handle<dyn AisPointCloudSource>,
    prs: &Handle<dyn Graphic3dStructureSource>,
    style: &Handle<Prs3dDrawer>,
    immediate: bool,
) {
    let fallback = if immediate {
        GRAPHIC3D_ZLAYER_ID_TOP
    } else {
        origin.z_layer()
    };
    let layer = if style.z_layer() == -1 {
        fallback
    } else {
        style.z_layer()
    };
    prs.clear(true);
    if prs.get_z_layer() != layer {
        prs.set_z_layer(layer);
    }
}

/// Native forced-subset highlight body for derived owners; `ais_point_cloud_owner_hilight_with_color(&*owner, &manager, &style)?`.
pub fn ais_point_cloud_owner_hilight_with_color<O: AisPointCloudOwnerSource + ?Sized>(
    owner: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let origin = cloud_origin(owner)?;
    let target = CloudSubsetTarget::new(owner, &origin, manager)?;
    refresh_cloud_subset(owner, &origin, &target.subset);
    prepare_cloud_subset_layer(&origin, &target.prs, style, target.immediate);
    if !draw_cloud_subset(&origin, &target.subset, &target.prs, style)? {
        return Ok(());
    }
    if target.immediate {
        manager.add_to_immediate_list(&target.prs);
    } else {
        target.prs.display();
    }
    Ok(())
}

struct CloudSubsetTarget {
    subset: Handle<HPackedMapOfInteger>,
    prs: Handle<dyn Graphic3dStructureSource>,
    immediate: bool,
}
impl CloudSubsetTarget {
    fn new<O: AisPointCloudOwnerSource + ?Sized>(
        owner: &O,
        origin: &Handle<dyn AisPointCloudSource>,
        manager: &Handle<PrsMgrPresentationManager>,
    ) -> Result<Self, Failure> {
        let immediate = manager.is_immediate_mode_on();
        let subset = if immediate {
            owner.detected_points()
        } else {
            owner.selected_points()
        };
        let prs = if immediate {
            origin.get_hilight_presentation(manager)?
        } else {
            origin.get_select_presentation(manager)?
        };
        Ok(Self {
            subset,
            prs,
            immediate,
        })
    }
}
