//! Native ResetStyles invokes real derived unset hooks before restoring geometry.
use super::{view_cube_interaction_fixture::widget_rows, *};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use std::sync::atomic::{AtomicUsize, Ordering};
struct CubeResetOverrides {
    transient: StandardTransient,
    base: AisViewCubeInteractionBase,
    attributes: AtomicUsize,
    highlights: AtomicUsize,
}
rustycad_tkernel::impl_transient!(CubeResetOverrides,"CubeResetOverrides",dyn AisInteractiveObject,transient,interfaces:[dyn AisInteractiveObject,dyn AisViewCubeSource,dyn PrsMgrPresentableObject,dyn SelectMgrSelectableObject]);
impl AisInteractiveObject for CubeResetOverrides {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.presentation.interactive
    }
}
impl AisViewCubePresentationSource for CubeResetOverrides {
    fn view_cube_presentation_base(&self) -> &AisViewCubePresentationBase {
        &self.base.presentation
    }
}
impl AisViewCubeSource for CubeResetOverrides {
    fn interaction_base(&self) -> &AisViewCubeInteractionBase {
        &self.base
    }
}
impl SelectMgrSelectableObject for CubeResetOverrides {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.presentation.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute_selection(self, selection, mode)
    }
}
impl PrsMgrPresentableObject for CubeResetOverrides {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentation.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute(self, prs, mode)
    }
    fn unset_attributes(&self) {
        self.attributes.fetch_add(1, Ordering::SeqCst);
        ais_view_cube_unset_attributes(self).unwrap();
    }
    fn unset_hilight_attributes(&self) {
        self.highlights.fetch_add(1, Ordering::SeqCst);
        ais_view_cube_unset_hilight_attributes(self).unwrap();
    }
}
#[test]
fn native_reset_styles_dispatches_both_real_application_unset_overrides() {
    let cube = Handle::new(CubeResetOverrides {
        transient: StandardTransient::default(),
        base: AisViewCubeInteractionBase::new().unwrap(),
        attributes: AtomicUsize::new(0),
        highlights: AtomicUsize::new(0),
    });
    prs_mgr_presentable_bind(&cube.cast());
    cube.reset_styles().unwrap();
    let native = include_str!("../../tests/fixtures/view-cube-appearance-aliases.txt");
    let row = widget_rows(native, "reset-dispatch").next().unwrap();
    assert_eq!(
        cube.attributes.load(Ordering::SeqCst),
        row[1].parse().unwrap()
    );
    assert_eq!(
        cube.highlights.load(Ordering::SeqCst),
        row[2].parse().unwrap()
    );
}
