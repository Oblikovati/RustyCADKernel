//! The native bodies of the `PrsMgr_PresentableObject` virtual members for layers, mutability,
//! transformations, clipping, children and `Fill`; an override calls them as the native
//! `PrsMgr_PresentableObject::Member` call.
use super::{PrsMgrPresentableObject, PrsMgrPresentation, PrsMgrPresentationManager};
use rustycad_tkernel::standard::{Failure, Handle, WeakHandle};
use rustycad_tkmath::gp::{GeneralTransform3, Transform3, TransformForm};
use rustycad_tkmath::toploc::Datum3;
use rustycad_tkservice::graphic3d::{
    Graphic3dClipPlane, Graphic3dPresentationAttributesMembers as _,
    Graphic3dPresentationAttributesSource as _, Graphic3dSequenceOfHClipPlane,
    Graphic3dStructureMembers as _, Graphic3dTransformPersSource, Graphic3dZLayerId,
};

/// The presentations of an object (a copy, so the caller holds no lock).
fn presentations_of<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
) -> Vec<Handle<PrsMgrPresentation>> {
    object
        .presentable_base()
        .fields
        .read()
        .presentations
        .clone()
}

/// Whether a transformation handle is set and not the identity.
pub(crate) fn is_non_identity(trsf: &Handle<Datum3>) -> bool {
    trsf.get()
        .is_some_and(|t| t.form() != TransformForm::Identity)
}

/// `PrsMgr_PresentableObject::SetZLayer`; `prs_mgr_set_z_layer(&*object, layer)`.
pub fn prs_mgr_set_z_layer<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    layer_id: Graphic3dZLayerId,
) {
    let drawer = object.presentable_base().read().drawer;
    if drawer.z_layer() == layer_id {
        return;
    }
    drawer.set_z_layer(layer_id);
    presentations_of(object)
        .iter()
        .filter_map(Handle::get)
        .for_each(|p| p.set_z_layer(layer_id));
}

/// `PrsMgr_PresentableObject::SetMutable`; `prs_mgr_set_mutable(&*object, true)`.
pub fn prs_mgr_set_mutable<O: PrsMgrPresentableObject + ?Sized>(object: &O, is_mutable: bool) {
    {
        let mut fields = object.presentable_base().fields.write();
        if fields.is_mutable == is_mutable {
            return;
        }
        fields.is_mutable = is_mutable;
    }
    presentations_of(object)
        .iter()
        .filter_map(Handle::get)
        .for_each(|p| p.set_mutable(is_mutable));
}

/// `PrsMgr_PresentableObject::SetTransformPersistence`: every presentation takes the persistence
/// and is recomputed. Example: `prs_mgr_set_transform_persistence(&*object, &pers)`.
pub fn prs_mgr_set_transform_persistence<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    trsf_pers: &Handle<dyn Graphic3dTransformPersSource>,
) {
    object
        .presentable_base()
        .fields
        .write()
        .transform_persistence = trsf_pers.clone();
    for prs in presentations_of(object).iter().filter_map(Handle::get) {
        prs.set_transform_persistence(trsf_pers);
        prs.re_compute();
    }
}

/// The absolute transformation from the parent and local ones (null for the identity).
fn absolute_transformation(parent: &Handle<Datum3>, local: &Handle<Datum3>) -> Handle<Datum3> {
    match (
        parent.get().filter(|_| is_non_identity(parent)),
        local.get().filter(|_| is_non_identity(local)),
    ) {
        (Some(p), Some(l)) => Handle::new(Datum3::new(p.trsf().multiplied(*l.trsf()))),
        (Some(_), None) => parent.clone(),
        (None, Some(_)) => local.clone(),
        (None, None) => Handle::default(),
    }
}

/// `PrsMgr_PresentableObject::UpdateTransformation`; a singular transformation raises the
/// `Standard_ConstructionError` of `gp_Trsf::Inverted` with the new transformation stored and
/// the identity inverse, as natively. Example: `prs_mgr_update_transformation(&*object)?`.
pub fn prs_mgr_update_transformation<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let fields = object.presentable_base().read();
    let trsf = absolute_transformation(
        &fields.combined_parent_transform,
        &fields.local_transformation,
    );
    {
        let mut own = object.presentable_base().fields.write();
        own.transformation = trsf.clone();
        own.inv_transformation = GeneralTransform3::from(Transform3::default());
    }
    if let Some(value) = trsf.get() {
        let inverse = GeneralTransform3::from(value.trsf().inverted()?);
        object.presentable_base().fields.write().inv_transformation = inverse;
    }
    fields
        .presentations
        .iter()
        .filter_map(Handle::get)
        .for_each(|p| p.set_transformation(&trsf));
    for child in fields.children.iter().filter_map(Handle::get) {
        child.set_combined_parent_transform(&trsf)?;
    }
    Ok(())
}

/// `PrsMgr_PresentableObject::AddClipPlane`; `prs_mgr_add_clip_plane(&*object, &plane)`.
pub fn prs_mgr_add_clip_plane<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    plane: &Handle<Graphic3dClipPlane>,
) {
    let planes = {
        let mut fields = object.presentable_base().fields.write();
        if fields.clip_planes.is_null() {
            fields.clip_planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        }
        fields.clip_planes.clone()
    };
    if let Some(planes) = planes.get() {
        planes.append(plane.clone());
    }
    object.update_clipping();
}

/// `PrsMgr_PresentableObject::RemoveClipPlane`: removes the first occurrence of the plane.
/// Example: `prs_mgr_remove_clip_plane(&*object, &plane)`.
pub fn prs_mgr_remove_clip_plane<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    plane: &Handle<Graphic3dClipPlane>,
) {
    let planes = object.presentable_base().read().clip_planes;
    let Some(sequence) = planes.get() else {
        return;
    };
    if sequence.remove(plane) {
        object.update_clipping();
    }
}

/// `PrsMgr_PresentableObject::UpdateClipping`; `prs_mgr_update_clipping(&*object)`.
pub fn prs_mgr_update_clipping<O: PrsMgrPresentableObject + ?Sized>(object: &O) {
    let fields = object.presentable_base().read();
    let planes = fields.clip_planes;
    fields
        .presentations
        .iter()
        .filter_map(Handle::get)
        .for_each(|p| p.set_clip_planes(&planes));
}

/// `PrsMgr_PresentableObject::AddChild`; `prs_mgr_add_child(&*parent, &child)?`.
pub fn prs_mgr_add_child<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    child: &Handle<dyn PrsMgrPresentableObject>,
) -> Result<(), Failure> {
    let Some(value) = child.get() else {
        return Ok(());
    };
    let previous = value.presentable_base().read().parent.upgrade();
    if let Some(previous) = previous.get() {
        previous.remove_child(child)?;
    }
    let (this, trsf) = {
        let mut fields = object.presentable_base().fields.write();
        fields.children.push(child.clone());
        (
            object.presentable_base().this(),
            fields.transformation.clone(),
        )
    };
    value.presentable_base().fields.write().parent = this.downgrade();
    value.set_combined_parent_transform(&trsf)
}

/// `PrsMgr_PresentableObject::RemoveChild`: detaches the first occurrence of the child.
/// Example: `prs_mgr_remove_child(&*parent, &child)?`.
pub fn prs_mgr_remove_child<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    child: &Handle<dyn PrsMgrPresentableObject>,
) -> Result<(), Failure> {
    let children = object.presentable_base().read().children;
    let Some(index) = children.iter().position(|c| c == child) else {
        return Ok(());
    };
    if let Some(value) = child.get() {
        value.presentable_base().fields.write().parent = WeakHandle::default();
        value.set_combined_parent_transform(&Handle::default())?;
    }
    object
        .presentable_base()
        .fields
        .write()
        .children
        .remove(index);
    Ok(())
}

/// `PrsMgr_PresentableObject::Fill`: `Compute`, then the transformation, the clipping planes
/// and the persistence of the object on the presentation. Example: `prs_mgr_fill(&*object, &manager, &prs, 0)?`.
pub fn prs_mgr_fill<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    manager: &Handle<PrsMgrPresentationManager>,
    prs: &Handle<PrsMgrPresentation>,
    mode: i32,
) -> Result<(), Failure> {
    object.compute(manager, &prs.clone().cast(), mode)?;
    let fields = object.presentable_base().read();
    if let Some(structure) = prs.get() {
        structure.set_transformation(&fields.transformation);
        structure.set_clip_planes(&fields.clip_planes);
        structure.set_transform_persistence(&fields.transform_persistence);
    }
    Ok(())
}
