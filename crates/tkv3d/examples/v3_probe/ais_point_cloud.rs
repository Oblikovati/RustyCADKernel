//! Exact retained buffer, cached bounds and appearance comparisons against pinned OCCT.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use rustycad_tkernel::{
    ncollection::HArray1,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::{aspect::AspectTypeOfFacingModel, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*, select3d::*};
fn color(value: Color) {
    for component in [value.red(), value.green(), value.blue()] {
        real(component);
    }
}
fn snapshot(cloud: &AisPointCloud) -> Result<(), Failure> {
    int(cloud.display_mode());
    int(cloud.hilight_mode());
    flag(cloud.get_points().is_null());
    flag(cloud.get_bounding_box().is_void());
    if !cloud.get_bounding_box().is_void() {
        for bound in cloud.get_bounding_box().get()? {
            real(bound);
        }
    }
    let points = cloud.get_points();
    if points.is_null() {
        return Ok(());
    }
    int(points.vertex_number());
    flag(points.has_vertex_colors());
    flag(points.has_vertex_normals());
    for index in 1..=points.vertex_number() {
        pnt(points.vertice(index)?);
        if points.has_vertex_colors() {
            color(points.vertex_color(index)?);
        }
        if points.has_vertex_normals() {
            for n in points.vertex_normal_xyz(index)? {
                real(n);
            }
        }
    }
    Ok(())
}
fn appearance(cloud: &AisPointCloud) {
    flag(cloud.has_color());
    flag(cloud.has_material());
    color(cloud.attributes().color());
    real(cloud.transparency());
    let shading = cloud.attributes().shading_aspect();
    flag(cloud.attributes().has_own_shading_aspect());
    flag(shading.aspect().distinguish());
    for facing in [
        AspectTypeOfFacingModel::FrontSide,
        AspectTypeOfFacingModel::BackSide,
    ] {
        color(shading.color(facing));
        int(shading.material(facing).name() as i32);
        color(shading.material(facing).color());
        real(shading.transparency(facing));
    }
    color(shading.aspect().interior_color());
    color(shading.aspect().back_interior_color());
}
fn coordinate_arrays(cloud: &AisPointCloud, colors: i32, normals: i32) -> Result<(), Failure> {
    let coords = Handle::new(HArray1::with_value(-7, -5, Point3::new(0.1, 2., 3.)).unwrap());
    coords
        .change_array1()
        .set_value(-6, Point3::new(4., -5., 6.))
        .unwrap();
    coords
        .change_array1()
        .set_value(-5, Point3::new(-7., 8., -9.))
        .unwrap();
    let colors = if colors == 0 {
        Handle::default()
    } else {
        Handle::new(
            HArray1::with_value(
                8,
                8 + if colors == 1 { 2 } else { 0 },
                Color::named(NamedColor::RED),
            )
            .unwrap(),
        )
    };
    let normals = if normals == 0 {
        Handle::default()
    } else {
        Handle::new(
            HArray1::with_value(42, 42 + if normals == 1 { 2 } else { 0 }, Direction3::Z).unwrap(),
        )
    };
    cloud.set_points_arrays(&coords, &colors, &normals)
}
fn cloud_operation(cloud: &AisPointCloud, op: i32) -> Result<(), Failure> {
    match op {
        0 => cloud.set_color(Color::named(NamedColor::RED)),
        1 => cloud.unset_color(),
        2 => cloud.set_material(&Graphic3dMaterialAspect::new(Graphic3dNameOfMaterial::Gold)),
        3 => cloud.unset_material(),
        4 => cloud.set_transparency(0.25),
        5 => {
            cloud.set_current_facing_model(AspectTypeOfFacingModel::FrontSide);
            Ok(())
        }
        6 => {
            cloud.set_current_facing_model(AspectTypeOfFacingModel::BackSide);
            Ok(())
        }
        7 => cloud.set_color(Color::named(NamedColor::BLUE)),
        _ => Ok(()),
    }
}
fn selection(cloud: &AisPointCloud, mode: i32) -> Result<(), Failure> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    cloud.compute_selection(&selection, mode)?;
    int(selection.entities().len() as i32);
    for entity in selection.entities() {
        let sensitive = entity.base_sensitive();
        int(sensitive.owner_id().priority());
        flag(sensitive.owner_id().is_forced_hilight());
        flag(
            sensitive.owner_id().selectable().object_address()
                == cloud.presentable_base().this().object_address(),
        );
        int(sensitive.sensitivity_factor());
        let array = sensitive.cast::<Select3DSensitivePrimitiveArray>();
        flag(!array.is_null());
        if !array.is_null() {
            int(array.nb_sub_elements());
        }
    }
    Ok(())
}
pub fn cloud_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let linked = reader.integer();
    let colors = reader.integer();
    let normals = reader.integer();
    let count = reader.integer();
    let cloud = AisPointCloud::new().unwrap();
    if linked != 0 {
        let drawer = Handle::new(Prs3dDrawer::new());
        drawer.setup_own_defaults().unwrap();
        drawer
            .shading_aspect()
            .set_color(
                Color::named(NamedColor::GREEN),
                AspectTypeOfFacingModel::BothSide,
            )
            .unwrap();
        cloud.attributes().link_to(&drawer);
    }
    attempt(|| coordinate_arrays(&cloud, colors, normals));
    attempt(|| snapshot(&cloud));
    appearance(&cloud);
    for _ in 0..count {
        attempt(|| cloud_operation(&cloud, reader.integer()));
        appearance(&cloud);
    }
    for mode in [-1, 0, 1, 2, 99] {
        attempt(|| selection(&cloud, mode));
    }
    end();
}
pub fn alias_case(_: &[f64]) {
    let cloud = AisPointCloud::new().unwrap();
    let points = Handle::new(Graphic3dArrayOfPoints::new(3, 0).unwrap());
    points.add_vertex_pnt(&Point3::new(0.1, 2., 3.)).unwrap();
    points.add_vertex_pnt(&Point3::new(4., -5., 6.)).unwrap();
    attempt(|| cloud.set_points(&points));
    attempt(|| snapshot(&cloud));
    points
        .set_vertice_pnt(1, &Point3::new(-9., -8., -7.))
        .unwrap();
    attempt(|| snapshot(&cloud));
    attempt(|| cloud.set_points(&points));
    attempt(|| snapshot(&cloud));
    attempt(|| cloud.set_points(&Handle::default()));
    attempt(|| snapshot(&cloud));
    attempt(|| {
        cloud.set_points_arrays(
            &Handle::new(HArray1::<Point3>::default()),
            &Handle::default(),
            &Handle::default(),
        )
    });
    attempt(|| snapshot(&cloud));
    attempt(|| selection(&cloud, 0));
    end();
}
fn owner_snapshot(owner: &AisPointCloudOwner, cloud: &AisPointCloud) -> Result<(), Failure> {
    int(owner.priority());
    flag(owner.is_forced_hilight());
    for subset in [owner.selected_points(), owner.detected_points()] {
        int(subset.map().extent());
        for index in 0..3 {
            flag(subset.map().contains(index));
        }
    }
    for prs in [
        cloud.get_select_presentation(&Handle::default())?,
        cloud.get_hilight_presentation(&Handle::default())?,
    ] {
        flag(!prs.is_null());
        if !prs.is_null() {
            flag(prs.is_displayed());
            int(prs.get_z_layer());
            int(prs.groups().len() as i32);
        }
    }
    Ok(())
}
fn owner_detect(sensitive: &Select3DSensitivePrimitiveArray, x: f64) -> Result<(), Failure> {
    let mut volume = SelectMgrSelectingVolumeManager::new();
    volume.init_axis_selecting_volume(rustycad_tkmath::gp::Axis1::new(
        Point3::new(x, 0., 10.),
        Direction3::new(0., 0., -1.)?,
    ));
    volume.build_selecting_volume()?;
    flag(sensitive.matches(
        &volume,
        &mut rustycad_tkv3d::select_basics::SelectBasicsPickResult::default(),
    )?);
    Ok(())
}
pub fn owner_case(args: &[f64]) {
    let mut reader = Reader::new(args);
    let (driver, _) =
        super::sensitive_display::display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = rustycad_tkv3d::v3d::V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let cloud = AisPointCloud::new().unwrap();
    let points = Handle::new(Graphic3dArrayOfPoints::new(3, 0).unwrap());
    for index in 0..3 {
        points
            .add_vertex_pnt(&Point3::new(f64::from(index) * 2., 0., 0.))
            .unwrap();
    }
    cloud.set_points(&points).unwrap();
    let selection = Handle::new(SelectMgrSelection::new(1));
    cloud.add_selection(&selection, 1).unwrap();
    let sensitive = selection.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitivePrimitiveArray>();
    let owner = sensitive.owner_id().cast::<AisPointCloudOwner>();
    let style = Handle::new(Prs3dDrawer::new());
    style.setup_own_defaults().unwrap();
    style.set_z_layer(reader.integer());
    owner_snapshot(&owner, &cloud).unwrap();
    attempt(|| owner_detect(&sensitive, 2.));
    attempt(|| owner.hilight_with_color(&manager, &style, 99));
    owner_snapshot(&owner, &cloud).unwrap();
    owner.clear(&manager, 0);
    owner_snapshot(&owner, &cloud).unwrap();
    owner.unhilight(&manager, 0);
    owner_snapshot(&owner, &cloud).unwrap();
    manager.begin_immediate_draw();
    attempt(|| owner_detect(&sensitive, 4.));
    attempt(|| owner.hilight_with_color(&manager, &style, 0));
    owner_snapshot(&owner, &cloud).unwrap();
    end();
}
