//! Application subclasses preserve AIS_RubberBand RTTI, protected caches and overridden hooks.
use rustycad_tkernel::standard::HandleReference;
#[allow(dead_code)]
#[path = "support/sensitive_display_driver.rs"]
mod display_driver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkservice::{aspect::AspectTypeOfLine, graphic3d::*};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, v3d::V3dViewer};
struct ApplicationRubberBand {
    transient: StandardTransient,
    base: AisRubberBandBase,
}
rustycad_tkernel::impl_transient!(ApplicationRubberBand,"ApplicationRubberBand",dyn AisRubberBandSource,transient,interfaces:[dyn AisRubberBandSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisRubberBandSource for ApplicationRubberBand {
    fn rubber_band_base(&self) -> &AisRubberBandBase {
        &self.base
    }
    fn fill_triangles(&self) -> Result<bool, Failure> {
        // Native Compute statically calls the nonvirtual fillTriangles body, ignoring a hidden method.
        Ok(false)
    }
}
impl AisInteractiveObject for ApplicationRubberBand {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
}
impl PrsMgrPresentableObject for ApplicationRubberBand {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.interactive.presentable
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        prs: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_rubber_band_compute(self, &prs.observe(), mode)
    }
}
impl SelectMgrSelectableObject for ApplicationRubberBand {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.interactive.selectable
    }
    fn compute_selection(
        &self,
        _: &dyn HandleReference<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        Ok(())
    }
}
#[test]
fn derived_cache_is_reused_and_compute_keeps_nonvirtual_triangulation() {
    let band = Handle::new(ApplicationRubberBand {
        transient: StandardTransient::default(),
        base: AisRubberBandBase::new(
            Color::named(NamedColor::WHITE),
            AspectTypeOfLine::Solid,
            1.,
            true,
        )
        .unwrap(),
    });
    prs_mgr_presentable_bind(&band.cast());
    ais_rubber_band_initialize_overlay(&*band).unwrap();
    let inherited = band.cast::<dyn AisRubberBandSource>();
    assert!(!inherited.is_null());
    inherited.set_rectangle(8, 8, 36, 36);
    inherited.set_filling(true);
    let array =
        Handle::new(Graphic3dArrayOfTriangles::with_options(6, 0, true, false, false).unwrap());
    for point in [
        [8., 8., 0.],
        [8., 36., 0.],
        [36., 8., 0.],
        [36., 8., 0.],
        [8., 36., 0.],
        [36., 36., 0.],
    ] {
        array.add_vertex_normal(point, [0., 1., 0.]).unwrap();
    }
    *band.base.fields.triangles.write() = array.clone();
    let (driver, _) = display_driver::SensitiveDisplayDriver::new_handle(0);
    let viewer = V3dViewer::new(&driver).unwrap();
    let manager = PrsMgrPresentationManager::new(&viewer.structure_manager());
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default()).unwrap();
    band.compute(&manager, &prs.cast(), 0).unwrap();
    assert_eq!(*band.base.fields.triangles.read(), array);
    assert_eq!(prs.groups().len(), 2);
    assert!(!band.fill_triangles().unwrap());
    assert_eq!(array.vertex_normal_xyz(1).unwrap(), [0., 1., 0.]);
    assert_eq!(band.z_layer(), GRAPHIC3D_ZLAYER_ID_TOP_OSD);
}
