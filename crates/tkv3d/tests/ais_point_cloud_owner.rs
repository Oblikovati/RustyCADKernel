//! Real ray detection drives forced point-cloud subsets and highlight lifecycle.
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis1, Direction3, Point3},
};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{
    ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_basics::SelectBasicsPickResult, select_mgr::*,
    select3d::*, v3d::V3dViewer,
};
fn cloud() -> Handle<AisPointCloud> {
    let cloud = AisPointCloud::new().unwrap();
    let points = Handle::new(Graphic3dArrayOfPoints::new(3, 0).unwrap());
    for point in [
        Point3::new(0., 0., 0.),
        Point3::new(2., 0., 0.),
        Point3::new(4., 0., 0.),
    ] {
        points.add_vertex_pnt(&point).unwrap();
    }
    cloud.set_points(&points).unwrap();
    cloud
}
fn detect(sensitive: &Handle<Select3DSensitivePrimitiveArray>, x: f64) {
    let mut volume = SelectMgrSelectingVolumeManager::new();
    volume.init_axis_selecting_volume(Axis1::new(
        Point3::new(x, 0., 10.),
        Direction3::new(0., 0., -1.).unwrap(),
    ));
    volume.build_selecting_volume().unwrap();
    assert!(
        sensitive
            .matches(&volume, &mut SelectBasicsPickResult::default())
            .unwrap()
    );
}
#[test]
fn forced_owner_keeps_independent_zero_based_maps_and_highlight_presentations() {
    let cloud = cloud();
    let selection = Handle::new(SelectMgrSelection::new(1));
    cloud.add_selection(&selection, 1).unwrap();
    let sensitive = selection.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitivePrimitiveArray>();
    let owner = sensitive.owner_id().cast::<AisPointCloudOwner>();
    assert!(owner.is_forced_hilight());
    assert_ne!(owner.selected_points(), owner.detected_points());
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let style = Handle::new(Prs3dDrawer::new());
    style.setup_own_defaults().unwrap();
    style.set_color(Color::named(NamedColor::RED));
    detect(&sensitive, 2.);
    owner.hilight_with_color(&manager, &style, 99).unwrap();
    assert!(owner.selected_points().map().contains(1));
    assert_eq!(owner.selected_points().map().extent(), 1);
    assert!(owner.detected_points().map().is_empty());
    let prs = cloud.get_select_presentation(&Handle::default()).unwrap();
    assert!(prs.is_displayed());
    assert_eq!(prs.groups().len(), 1);
    owner.clear(&manager, 0);
    assert!(prs.is_displayed());
    owner.unhilight(&manager, 0);
    assert!(!prs.is_displayed());
    style.set_z_layer(-1);
    manager.begin_immediate_draw();
    detect(&sensitive, 4.);
    owner.hilight_with_color(&manager, &style, 0).unwrap();
    assert!(owner.detected_points().map().contains(2));
    assert!(owner.selected_points().map().contains(1));
    let dynamic = cloud.get_hilight_presentation(&Handle::default()).unwrap();
    assert_eq!(dynamic.get_z_layer(), GRAPHIC3D_ZLAYER_ID_TOP);
    style.set_z_layer(0);
    owner.hilight_with_color(&manager, &style, 0).unwrap();
    assert_eq!(dynamic.get_z_layer(), 0);
}
#[test]
fn owner_rejects_wrong_selectable_and_clears_empty_subset_geometry() {
    let cloud = cloud();
    let owner = AisPointCloudOwner::new(&cloud.cast());
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let style = Handle::new(Prs3dDrawer::new());
    style.setup_own_defaults().unwrap();
    owner.hilight_with_color(&manager, &style, 0).unwrap();
    assert!(
        cloud
            .get_select_presentation(&Handle::default())
            .unwrap()
            .groups()
            .is_empty()
    );
    owner.set_selectable(&Handle::default());
    assert_eq!(
        owner
            .hilight_with_color(&manager, &style, 0)
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
    owner.unhilight(&Handle::default(), 0);
}
struct DynamicPointCloud {
    transient: StandardTransient,
    base: AisPointCloudBase,
    points: Handle<Graphic3dArrayOfPoints>,
    bounds: BoundingBox3,
    point_reads: SharedState<usize>,
}
rustycad_tkernel::impl_transient!(DynamicPointCloud,"DynamicPointCloud",dyn AisPointCloudSource,transient,
 interfaces:[dyn AisPointCloudSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisPointCloudSource for DynamicPointCloud {
    fn point_cloud_base(&self) -> &AisPointCloudBase {
        &self.base
    }
    fn set_points(&self, points: &Handle<Graphic3dArrayOfPoints>) -> Result<(), Failure> {
        self.point_cloud_base().set_points(points)
    }
    fn get_points(&self) -> Handle<Graphic3dArrayOfPoints> {
        *self.point_reads.write() += 1;
        self.points.clone()
    }
    fn get_bounding_box(&self) -> BoundingBox3 {
        self.bounds.clone()
    }
}
impl AisInteractiveObject for DynamicPointCloud {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
}
impl PrsMgrPresentableObject for DynamicPointCloud {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_point_cloud_compute(self, prs, mode)
    }
}
impl SelectMgrSelectableObject for DynamicPointCloud {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_point_cloud_compute_selection(self, selection, mode)
    }
}
#[test]
fn derived_virtual_provider_supplies_points_and_bounds_to_native_hooks() {
    let cloud = cloud();
    let derived = Handle::new(DynamicPointCloud {
        transient: StandardTransient::default(),
        base: AisPointCloudBase::new().unwrap(),
        points: cloud.get_points(),
        point_reads: SharedState::new(0),
        bounds: BoundingBox3::new(Point3::new(-9., -8., -7.), Point3::new(9., 8., 7.)),
    });
    prs_mgr_presentable_bind(&derived.cast());
    assert!(derived.point_cloud_base().fields.read().points.is_null());
    let replacement = cloud.get_points();
    derived.set_points(&replacement).unwrap();
    assert_eq!(derived.point_cloud_base().fields.read().points, replacement);
    assert_eq!(derived.get_points(), replacement);
    *derived.point_reads.write() = 0;
    let selection = Handle::new(SelectMgrSelection::new(2));
    derived.compute_selection(&selection, 2).unwrap();
    assert_eq!(*derived.point_reads.read(), 0);
    let box_entity = selection.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitiveBox>();
    assert_eq!(
        box_entity.bnd_box().get().unwrap(),
        derived.bounds.get().unwrap()
    );
    let points = Handle::new(SelectMgrSelection::new(1));
    derived.compute_selection(&points, 1).unwrap();
    assert!(
        !points.entities()[0]
            .base_sensitive()
            .owner_id()
            .cast::<AisPointCloudOwner>()
            .is_null()
    );
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    derived.compute(&Handle::default(), &prs.cast(), 0).unwrap();
    assert_eq!(prs.groups().len(), 1);
}
struct DerivedSubsetOwner {
    transient: StandardTransient,
    base: AisPointCloudOwnerBase,
}
rustycad_tkernel::impl_transient!(DerivedSubsetOwner,"DerivedSubsetOwner",dyn AisPointCloudOwnerSource,transient,
 interfaces:[dyn AisPointCloudOwnerSource,dyn SelectMgrEntityOwnerSource]);
impl AisPointCloudOwnerSource for DerivedSubsetOwner {
    fn point_cloud_owner_base(&self) -> &AisPointCloudOwnerBase {
        &self.base
    }
}
impl SelectMgrEntityOwnerSource for DerivedSubsetOwner {
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
}
#[test]
fn derived_subset_owner_embeds_native_protected_maps_and_shared_highlight_body() {
    let cloud = cloud();
    let owner = Handle::new(DerivedSubsetOwner {
        transient: StandardTransient::default(),
        base: AisPointCloudOwnerBase::new(&cloud.cast()),
    });
    select_mgr_entity_owner_bind(&owner.cast());
    let selection = Handle::new(SelectMgrSelection::new(1));
    cloud.add_selection(&selection, 1).unwrap();
    let sensitive = selection.entities()[0]
        .base_sensitive()
        .cast::<Select3DSensitivePrimitiveArray>();
    sensitive.set(&owner.cast());
    detect(&sensitive, 2.);
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let style = Handle::new(Prs3dDrawer::new());
    style.setup_own_defaults().unwrap();
    owner.hilight_with_color(&manager, &style, 0).unwrap();
    assert_eq!(owner.selected_points(), owner.base.fields.read().selected);
    assert!(owner.base.fields.read().selected.map().contains(1));
    assert_eq!(owner.detected_points(), owner.base.fields.read().detected);
    assert!(owner.base.fields.read().detected.map().is_empty());
    let replacement = Handle::new(rustycad_tkernel::tcol_std::HPackedMapOfInteger::default());
    replacement.change_map().add(2);
    owner.base.fields.write().selected = replacement.clone();
    assert_eq!(owner.selected_points(), replacement);
    let selected = owner.selected_points();
    drop(owner);
    assert!(selected.map().contains(2));
}
