//! Native point arrays and cached bounding boxes feed presentations and selection.
use super::{AisPointCloud, AisPointCloudOwner, AisPointCloudSource};
use crate::{prs_mgr::*, prs3d::Prs3dBndBox, select_mgr::*, select3d::*};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::toploc::Location;
use rustycad_tkservice::graphic3d::*;
/// Compute using the virtual geometry provider; `ais_point_cloud_compute(&*cloud, &prs, 0)?`.
pub fn ais_point_cloud_compute<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    mode: i32,
) -> Result<(), Failure> {
    if mode == 2 {
        return Prs3dBndBox::add_box(prs, &cloud.get_bounding_box(), &cloud.attributes());
    }
    if mode != 0 {
        return Ok(());
    }
    let points = cloud.get_points();
    if points.is_null() {
        return Ok(());
    }
    let group = prs.new_group()?;
    group.set_group_primitives_aspect(&cloud.attributes().shading_aspect().aspect().cast());
    group.add_primitive_array(&points.cast(), true);
    Ok(())
}
/// Build native point/subset/box selection without clearing prior entities; `ais_point_cloud_compute_selection(&*cloud, &selection, 1)?`.
pub fn ais_point_cloud_compute_selection<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    if !(0..=2).contains(&mode) {
        return Ok(());
    }
    let selectable = cloud.presentable_base().this().cast();
    let owner = SelectMgrEntityOwner::with_selectable(&selectable, 0).cast();
    let points = if mode == 2 {
        Handle::default()
    } else {
        cloud.get_points()
    };
    if mode != 2 && points.get().is_some_and(|p| !p.attributes().is_null()) {
        return add_cloud_point_selection(selection, &points, &selectable, &owner, mode == 1);
    }
    let bounds = cloud.get_bounding_box();
    if bounds.is_void() {
        return Ok(());
    }
    selection.add(&Handle::new(Select3DSensitiveBox::new(&owner, &bounds)?).cast())
}
fn add_cloud_point_selection(
    selection: &Handle<SelectMgrSelection>,
    points: &Handle<Graphic3dArrayOfPoints>,
    selectable: &Handle<dyn SelectMgrSelectableObject>,
    whole_owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    subset: bool,
) -> Result<(), Failure> {
    let owner = if subset {
        AisPointCloudOwner::new(&selectable.cast()).cast()
    } else {
        whole_owner.clone()
    };
    let sensitive = Handle::new(Select3DSensitivePrimitiveArray::new(&owner)?);
    sensitive.set_detect_elements(true);
    sensitive.set_detect_element_map(subset);
    sensitive.set_sensitivity_factor(8)?;
    let attributes = points.attributes();
    let groups = point_cloud_selection_groups(attributes.nb_elements());
    sensitive.init_points(&Select3DPrimArrayInit::whole(
        &attributes,
        &points.indices(),
        &Location::default(),
        true,
        groups,
    ))?;
    sensitive.bvh()?;
    selection.add(&sensitive.cast())
}
impl PrsMgrPresentableObject for AisPointCloud {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.point_cloud_base().interactive.presentable
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_point_cloud_compute(self, &prs.observe(), mode)
    }
    fn set_color(&self, color: rustycad_tkernel::quantity::Color) -> Result<(), Failure> {
        super::ais_point_cloud_set_color(self, color)
    }
    fn unset_color(&self) -> Result<(), Failure> {
        super::ais_point_cloud_unset_color(self)
    }
    fn set_material(&self, material: &Graphic3dMaterialAspect) -> Result<(), Failure> {
        super::ais_point_cloud_set_material(self, material)
    }
    fn unset_material(&self) -> Result<(), Failure> {
        super::ais_point_cloud_unset_material(self)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisPointCloud {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.point_cloud_base().interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_point_cloud_compute_selection(self, &selection.observe(), mode)
    }
}

fn point_cloud_selection_groups(count: i32) -> i32 {
    // OCCT 3d097a0 splits only above 500000 attribute elements, not used indices.
    if count > 500_000 {
        return 8;
    }
    1
}
#[cfg(test)]
mod tests {
    use super::point_cloud_selection_groups;
    #[test]
    fn native_group_threshold_is_strictly_above_half_a_million_attributes() {
        assert_eq!(point_cloud_selection_groups(0), 1);
        assert_eq!(point_cloud_selection_groups(500_000), 1);
        assert_eq!(point_cloud_selection_groups(500_001), 8);
    }
}
