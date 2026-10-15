//! Shared steps of the relation presentations: the label position along a leader, the
//! drawer aspects and the text-box selection.
use super::{PrsDim, PrsDimRelationMembers, PrsDimRelationSource};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::prs3d::{Prs3dArrowAspect, Prs3dDimensionAspect};
use crate::select_mgr::{SelectMgrEntityOwner, SelectMgrSelection};
use crate::select3d::{Select3DSensitiveBox, Select3DSensitiveSegment};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};

/// `Standard_NullObject` for a null handle the native code dereferences.
pub(super) fn null_object() -> Failure {
    Failure::with_message("Standard_NullObject", "PrsDim_Relation: null object")
}

/// The dimension aspect of the drawer.
pub(super) fn relation_dimension_aspect<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
) -> Result<Handle<Prs3dDimensionAspect>, Failure> {
    let aspect = relation.attributes().dimension_aspect();
    if aspect.is_null() {
        return Err(null_object());
    }
    Ok(aspect)
}

/// The arrow aspect of the drawer dimension aspect.
pub(super) fn relation_arrow<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
) -> Result<Handle<Prs3dArrowAspect>, Failure> {
    let aspect = relation_dimension_aspect(relation)?;
    Ok(aspect.get().ok_or_else(null_object)?.arrow_aspect())
}

/// The label position on a leader from an attach point: at the value distance (moved to the
/// bounding box) when automatic, else the position projected on the leader, at least
/// `min_distance` away; stored as the position.
pub(super) fn automatic_or_projected<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    attach: &Point3,
    direction: &Direction3,
    min_distance: f64,
) -> Result<Point3, Failure> {
    let f = relation.relation_base().fields.read().clone();
    let position = if f.automatic_position {
        let position = attach.translated(Vector3::from(*direction) * f.val);
        if f.is_set_bnd_box {
            PrsDim::translate_point_to_bound(&position, direction, &f.bnd_box)?
        } else {
            position
        }
    } else {
        let line = Line3::new(*attach, *direction);
        let position = elclib::value(elclib::parameter(&line, f.position)?, &line);
        if position.distance(attach) < min_distance {
            attach.translated(Vector3::from(*direction) * min_distance)
        } else {
            position
        }
    };
    relation.relation_base().fields.write().position = position;
    Ok(position)
}

/// An owner of the relation with a priority (`new SelectMgr_EntityOwner(this, priority)`).
pub(super) fn relation_owner<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    priority: i32,
) -> Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource> {
    SelectMgrEntityOwner::with_selectable(&relation.presentable_base().this().cast(), priority)
        .cast()
}

/// An optional sensitive segment and a sensitive box of a size at the position.
pub(super) fn text_box_selection<R: PrsDimRelationSource + ?Sized>(
    relation: &R,
    selection: &Handle<SelectMgrSelection>,
    priority: i32,
    segment: Option<(&Point3, &Point3)>,
    size: f64,
) -> Result<(), Failure> {
    let owner = relation_owner(relation, priority);
    let selection = selection.get().ok_or_else(null_object)?;
    if let Some((first, last)) = segment {
        selection.add(&Handle::new(Select3DSensitiveSegment::new(&owner, *first, *last)).cast())?;
    }
    let p = relation.position();
    let bounds = Select3DSensitiveBox::from_bounds(
        &owner,
        p.x(),
        p.y(),
        p.z(),
        p.x() + size,
        p.y() + size,
        p.z() + size,
    );
    selection.add(&Handle::new(bounds).cast())
}
