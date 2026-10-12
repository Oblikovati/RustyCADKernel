//! OpenGl rendering consumes myCamera directly; virtual Camera is a separate public reference contract.
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{Graphic3dCView, Graphic3dCamera, graphic3d_cview_camera_ref};

pub(crate) fn owned_render_camera(view: &dyn Graphic3dCView) -> Handle<Graphic3dCamera> {
    // OCCT 3d097a0 OpenGl_View.cxx:SetCamera uses myCamera, not an overridable Camera().
    graphic3d_cview_camera_ref(view.cview_base()).clone_handle()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkservice::{graphic3d::*, null_driver::NullGraphicDriver};
    #[test]
    fn renderer_reads_owned_camera_field_and_observes_its_replacement() {
        let driver = NullGraphicDriver::new_handle();
        let owner = Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
        let view = Graphic3dRetainedView::new(&owner, &[]).unwrap();
        let next = Handle::new(Graphic3dCamera::default());
        assert_ne!(
            owned_render_camera(&view).object_address(),
            next.object_address()
        );
        view.set_camera(&next);
        assert_eq!(
            owned_render_camera(&view).object_address(),
            next.object_address()
        );
        assert_eq!(next.ref_count(), 2);
    }
}
