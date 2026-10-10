//! A real interactive object builds retained groups and a selectable center point.
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions::program_error};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, select3d::Select3DSensitivePoint};
use std::sync::atomic::{AtomicUsize, Ordering};
pub struct AisTriangle {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    pub vertices: Handle<Graphic3dArrayOfPrimitives>,
    styles: [Handle<dyn Graphic3dAspectsSource>; 2],
    pub computations: AtomicUsize,
}
rustycad_tkernel::impl_transient!(AisTriangle,"Martensite_TestTriangle",dyn AisInteractiveObject,transient,interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisTriangle {
    pub fn new(
        vertices: Handle<Graphic3dArrayOfPrimitives>,
        styles: [Handle<dyn Graphic3dAspectsSource>; 2],
    ) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            vertices,
            styles,
            computations: AtomicUsize::new(0),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}
impl PrsMgrPresentableObject for AisTriangle {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        (0..=1).contains(&mode)
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        let style = self.styles.get(mode as usize).ok_or_else(|| {
            program_error(&format!("triangle display mode {mode}; expected 0 or 1"))
        })?;
        let group = presentation.new_group()?;
        group.set_group_primitives_aspect(style);
        group.add_primitive_array(&self.vertices, true);
        self.computations.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    rustycad_tkv3d::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisTriangle {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), mode);
        selection.add(
            &Handle::new(Select3DSensitivePoint::new(
                &owner.cast(),
                Point3::default(),
            ))
            .cast(),
        )
    }
}
impl AisInteractiveObject for AisTriangle {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}
