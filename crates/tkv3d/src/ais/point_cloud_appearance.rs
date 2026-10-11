//! Preserve point cloud color/material ownership and OCCT 3d097a0 restoration order.
use super::AisPointCloudSource;
use crate::prs_mgr::*;
use rustycad_tkernel::{quantity::Color, standard::Failure};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
/// Set the drawer and shading colors and synchronize live groups; `ais_point_cloud_set_color(&*cloud, red)?`.
pub fn ais_point_cloud_set_color<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
    color: Color,
) -> Result<(), Failure> {
    cloud.presentable_base().fields.write().has_own_color = true;
    let drawer = cloud.attributes();
    drawer.set_color(color);
    drawer
        .shading_aspect()
        .set_color(color, AspectTypeOfFacingModel::BothSide)?;
    cloud.synchronize_aspects();
    Ok(())
}
fn restored_cloud_material<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
) -> (Graphic3dMaterialAspect, Color) {
    let drawer = cloud.attributes();
    let mut material = Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Brass);
    let color = if drawer.has_link() {
        drawer
            .link()
            .shading_aspect()
            .color(cloud.current_facing_model())
    } else {
        material.color()
    };
    if cloud.has_material() || drawer.has_link() {
        // AIS_GraphicTool::GetMaterial always reads the front material, independently of facing.
        let source = if cloud.has_material() {
            drawer
        } else {
            drawer.link()
        };
        material = source
            .shading_aspect()
            .material(AspectTypeOfFacingModel::FrontSide);
    }
    (material, color)
}
/// Restore inherited/brass color while preserving own material and transparency; `ais_point_cloud_unset_color(&*cloud)?`.
pub fn ais_point_cloud_unset_color<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
) -> Result<(), Failure> {
    if !cloud.has_color() {
        return Ok(());
    }
    cloud.presentable_base().fields.write().has_own_color = false;
    let (mut material, color) = restored_cloud_material(cloud);
    if cloud.has_material() {
        material.set_color(color)?;
    }
    let shading = cloud.attributes().shading_aspect();
    if cloud.is_transparent() {
        material.set_transparency(shading.transparency(cloud.current_facing_model()) as f32)?;
    }
    shading.set_material(&material, cloud.current_facing_model());
    shading.aspect().set_interior_color(color);
    cloud.synchronize_aspects();
    Ok(())
}
/// Assign material to the current facing model and reapply drawer overrides; `ais_point_cloud_set_material(&*cloud, &gold)?`.
pub fn ais_point_cloud_set_material<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
    material: &Graphic3dMaterialAspect,
) -> Result<(), Failure> {
    cloud.presentable_base().fields.write().has_own_material = true;
    cloud
        .attributes()
        .shading_aspect()
        .set_material(material, cloud.current_facing_model());
    reapply_cloud_appearance(cloud, true)?;
    cloud.synchronize_aspects();
    Ok(())
}
fn reapply_cloud_appearance<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
    always_transparency: bool,
) -> Result<(), Failure> {
    let drawer = cloud.attributes();
    let shading = drawer.shading_aspect();
    if cloud.has_color() {
        shading.set_color(drawer.color(), cloud.current_facing_model())?;
    }
    if cloud.has_color() || always_transparency {
        shading.set_transparency(
            f64::from(drawer.transparency()),
            cloud.current_facing_model(),
        )?;
    }
    Ok(())
}
/// Restore linked/brass material, retaining own color and native alpha policy; `ais_point_cloud_unset_material(&*cloud)?`.
pub fn ais_point_cloud_unset_material<O: AisPointCloudSource + ?Sized>(
    cloud: &O,
) -> Result<(), Failure> {
    if !cloud.has_material() {
        return Ok(());
    }
    let drawer = cloud.attributes();
    let material = if drawer.has_link() {
        drawer
            .link()
            .shading_aspect()
            .material(cloud.current_facing_model())
    } else {
        Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Brass)
    };
    drawer
        .shading_aspect()
        .set_material(&material, cloud.current_facing_model());
    reapply_cloud_appearance(cloud, false)?;
    cloud.presentable_base().fields.write().has_own_material = false;
    cloud.synchronize_aspects();
    Ok(())
}
