//! A presentation-only AIS subclass exercises real ViewCube defaults without inventing selection.
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions::program_error};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*};
pub struct ViewCubeCaptureAdapter {
    transient: StandardTransient,
    pub presentation: AisViewCubePresentationBase,
}
rustycad_tkernel::impl_transient!(ViewCubeCaptureAdapter,"ViewCubeCaptureAdapter",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisInteractiveObject,dyn PrsMgrPresentableObject,dyn SelectMgrSelectableObject]);
impl ViewCubeCaptureAdapter {
    /// Bind a presentation-only AIS owner; `ViewCubeCaptureAdapter::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let cube = Handle::new(Self {
            transient: StandardTransient::default(),
            presentation: AisViewCubePresentationBase::new()?,
        });
        prs_mgr_presentable_bind(&cube.cast());
        Ok(cube)
    }
}
impl AisInteractiveObject for ViewCubeCaptureAdapter {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.presentation.interactive
    }
}
impl AisViewCubePresentationSource for ViewCubeCaptureAdapter {
    fn view_cube_presentation_base(&self) -> &AisViewCubePresentationBase {
        &self.presentation
    }
}
impl PrsMgrPresentableObject for ViewCubeCaptureAdapter {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.presentation.interactive.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_view_cube_compute(self, presentation, mode)
    }
    rustycad_tkv3d::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for ViewCubeCaptureAdapter {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.presentation.interactive.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, mode: i32) -> Result<(), Failure> {
        Err(program_error(&format!(
            "ViewCubeCaptureAdapter selection mode={mode}; expected application-provided selection implementation"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presentation_adapter_preserves_defaults_and_rejects_selection() {
        let cube = ViewCubeCaptureAdapter::new().unwrap();
        assert_eq!(cube.presentation.settings.read().geometry.size, 70.);
        assert_eq!(
            cube.z_layer(),
            rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_TOPMOST
        );
        assert!(!cube.transform_persistence().is_null());
        let selection = Handle::new(SelectMgrSelection::new(0));
        assert!(
            cube.compute_selection(&selection, 0)
                .unwrap_err()
                .to_string()
                .contains("application-provided")
        );
    }
}
