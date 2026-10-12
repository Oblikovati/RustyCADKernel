//! Selection consumes the last computed arrays, deliberately excluding arrows.
use super::{
    AisTrihedron, AisTrihedronOwner, AisTrihedronSource,
    trihedron::{TRI_AXIS_PARTS, TRI_PLANE_PARTS},
};
use crate::{prs_mgr::*, prs3d::*, select_mgr::*, select3d::*};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::*;
/// Construct native sensitivity for one cached part; `ais_trihedron_sensitive(&*trihedron, part, &owner)?`.
pub fn ais_trihedron_sensitive<O: AisTrihedronSource + ?Sized>(
    object: &O,
    part: Prs3dDatumParts,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let array = object.array_of_primitives(part);
    if array.is_null() {
        return Ok(Handle::default());
    }
    if TRI_PLANE_PARTS.contains(&part) {
        return Ok(Handle::new(Select3DSensitiveTriangle::new(
            owner,
            array.vertice(1)?,
            array.vertice(2)?,
            array.vertice(3)?,
            Select3DTypeOfSensitivity::Interior,
        ))
        .cast());
    }
    if object.datum_display_mode() == Prs3dDatumMode::Shaded {
        let sensitive = Handle::new(Select3DSensitivePrimitiveArray::new(owner)?);
        sensitive.init_triangulation(&Select3DPrimArrayInit::whole(
            &array.attributes(),
            &array.indices(),
            &Location::default(),
            true,
            1,
        ))?;
        return Ok(sensitive.cast());
    }
    if !array.cast::<Graphic3dArrayOfPoints>().is_null() {
        return Ok(Handle::new(Select3DSensitivePoint::new(owner, array.vertice(1)?)).cast());
    }
    if !array.cast::<Graphic3dArrayOfSegments>().is_null() {
        return Ok(Handle::new(Select3DSensitiveSegment::new(
            owner,
            array.vertice(1)?,
            array.vertice(2)?,
        ))
        .cast());
    }
    Ok(Handle::default())
}
/// Append native whole/part owners using the current draw flags; `ais_trihedron_compute_selection(&*trihedron, &selection, 2)?`.
pub fn ais_trihedron_compute_selection<O: AisTrihedronSource + ?Sized>(
    object: &O,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    let parts: Vec<Prs3dDatumParts> = match mode {
        0 => {
            if object.datum_display_mode() == Prs3dDatumMode::Shaded {
                vec![
                    Prs3dDatumParts::Origin,
                    TRI_AXIS_PARTS[0],
                    TRI_AXIS_PARTS[1],
                    TRI_AXIS_PARTS[2],
                ]
            } else {
                TRI_AXIS_PARTS.to_vec()
            }
        }
        1 => vec![Prs3dDatumParts::Origin],
        2 => TRI_AXIS_PARTS.to_vec(),
        3 => TRI_PLANE_PARTS.to_vec(),
        _ => return Ok(()),
    };
    let selectable = object.presentable_base().this().cast();
    let whole: Handle<dyn SelectMgrEntityOwnerSource> = SelectMgrEntityOwner::with_selectable(
        &selectable,
        object.selection_priority(Prs3dDatumParts::None),
    )
    .cast();
    for part in parts {
        if !object.attributes().datum_aspect().draw_datum_part(part) {
            continue;
        }
        let owner = if mode == 0 {
            whole.clone()
        } else {
            AisTrihedronOwner::new(&selectable, part, object.selection_priority(part)).cast()
        };
        selection.add(&ais_trihedron_sensitive(object, part, &owner)?)?;
    }
    Ok(())
}
impl SelectMgrSelectableObject for AisTrihedron {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.trihedron_base().interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_trihedron_compute_selection(self, &selection.observe(), mode)
    }
    fn hilight_owner_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<(), Failure> {
        super::ais_trihedron_hilight_owner(self, manager, style, owner)
    }
    fn hilight_selected(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        owners: &[Handle<dyn SelectMgrEntityOwnerSource>],
    ) -> Result<(), Failure> {
        super::ais_trihedron_hilight_selected(self, manager, owners)
    }
    fn clear_selected(&self) {
        super::ais_trihedron_clear_selected(self);
    }
}
