//! Named application source has real parent state and a deliberately hidden nonvirtual Face member.
use super::StdPrsIsolineSurfaceSource;
use super::isoline_face_fixture::isoline_fixture_face;
use rustycad_tkbrep::{brep_adaptor::BRepAdaptorSurface, topods::Face};
use rustycad_tkernel::standard::SharedState;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
pub(super) struct RFaceLifetimeSourceFake {
    core: BRepAdaptorSurface,
    decoy: Face,
    pub hidden_face_calls: SharedState<i32>,
}
rustycad_tkernel::impl_transient!(RFaceLifetimeSourceFake,"Martensite_RFaceLifetimeSourceFake",BRepAdaptorSurface,core,interfaces:[dyn StdPrsIsolineSurfaceSource,dyn Adaptor3dSurface]);
impl RFaceLifetimeSourceFake {
    pub(super) fn new() -> Self {
        Self {
            core: BRepAdaptorSurface::with_face(&isoline_fixture_face(0), true).unwrap(),
            decoy: isoline_fixture_face(1),
            hidden_face_calls: SharedState::new(0),
        }
    }
    pub(super) fn face(&self) -> Face {
        *self.hidden_face_calls.write() += 1;
        self.decoy.clone()
    }
}
impl StdPrsIsolineSurfaceSource for RFaceLifetimeSourceFake {
    fn isoline_surface_base(&self) -> &BRepAdaptorSurface {
        // Constructor projections must read the real parent face, not same-name application members.
        &self.core
    }
}
impl Adaptor3dSurface for RFaceLifetimeSourceFake {
    fn get_type(
        &self,
    ) -> Result<rustycad_tkmath::geom_abs::SurfaceType, rustycad_tkernel::standard::Failure> {
        let core = &self.core;
        core.get_type()
    }
}
