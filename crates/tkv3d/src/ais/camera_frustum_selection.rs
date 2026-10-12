//! Native edge groups and aliased triangle sensitivity; selection is not invalidated by camera updates.
use super::AisCameraFrustum;
use crate::{prs_mgr::*, select_mgr::*, select3d::*};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::toploc::Location;
impl AisCameraFrustum {
    fn frustum_edge_selection(
        &self,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let group = Handle::new(Select3DSensitiveGroup::new(owner, true));
        let borders = self.fields.read().borders.clone();
        let borders = borders.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "AIS_CameraFrustum edges: borders=null; expected SetCameraFrustum before selection",
            )
        })?;
        for index in (1..=borders.edge_number()).step_by(2) {
            let start = borders.vertice(borders.edge(index)?)?;
            let end = borders.vertice(borders.edge(index + 1)?)?;
            group.add(&Handle::new(Select3DSensitiveSegment::new(owner, start, end)).cast());
        }
        Ok(group.cast())
    }
    fn frustum_volume_selection(
        &self,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    ) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let array = Handle::new(Select3DSensitivePrimitiveArray::new(owner)?);
        let triangles = self.fields.read().triangles.clone();
        let triangles = triangles.get().ok_or_else(|| rustycad_tkernel::standard::exceptions::null_object("AIS_CameraFrustum volume: triangles=null; expected SetCameraFrustum before selection"))?;
        array.init_triangulation(&Select3DPrimArrayInit::whole(
            &triangles.attributes(),
            &triangles.indices(),
            &Location::default(),
            true,
            1,
        ))?;
        Ok(array.cast())
    }
}
impl SelectMgrSelectableObject for AisCameraFrustum {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), 0).cast();
        let sensitive = match mode {
            0 => self.frustum_edge_selection(&owner)?,
            1 => self.frustum_volume_selection(&owner)?,
            _ => return Ok(()),
        };
        selection.observe().add(&sensitive)
    }
}
