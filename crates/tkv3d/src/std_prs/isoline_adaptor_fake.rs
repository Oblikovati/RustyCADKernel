//! Named application-derived adaptor owns the actual BRep core and injects a genuine virtual sample failure.
use super::{StdPrsIsolineSurfaceSource, isoline_face_fixture::isoline_fixture_face};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkernel::standard::{Failure, SharedState};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::{
    geom_abs::SurfaceType,
    gp::{Plane3, Point3},
};
pub(super) struct IsolineAdaptorFake {
    core: BRepAdaptorSurface,
    pub points: SharedState<i32>,
    pub fail: SharedState<bool>,
    pub unrelated_panic: SharedState<bool>,
    replacement: SharedState<Option<IsolineSourceReplacement>>,
}
rustycad_tkernel::impl_transient!(IsolineAdaptorFake,"Martensite_IsolineAdaptorFake",BRepAdaptorSurface,core,interfaces:[dyn Adaptor3dSurface,dyn StdPrsIsolineSurfaceSource]);
impl IsolineAdaptorFake {
    pub(super) fn new() -> Self {
        Self {
            core: BRepAdaptorSurface::with_face(&isoline_fixture_face(0), true).unwrap(),
            points: SharedState::new(0),
            fail: SharedState::new(true),
            unrelated_panic: SharedState::new(false),
            replacement: SharedState::new(None),
        }
    }
}
impl StdPrsIsolineSurfaceSource for IsolineAdaptorFake {
    fn isoline_surface_base(&self) -> &BRepAdaptorSurface {
        // Nonvirtual native Face reads the genuine parent core rather than an override-supplied face copy.
        &self.core
    }
}
impl Adaptor3dSurface for IsolineAdaptorFake {
    fn first_u_parameter(&self) -> Result<f64, Failure> {
        let core = &self.core;
        core.first_u_parameter()
    }
    fn last_u_parameter(&self) -> Result<f64, Failure> {
        let core = &self.core;
        core.last_u_parameter()
    }
    fn first_v_parameter(&self) -> Result<f64, Failure> {
        let core = &self.core;
        core.first_v_parameter()
    }
    fn last_v_parameter(&self) -> Result<f64, Failure> {
        let core = &self.core;
        core.last_v_parameter()
    }
    fn get_type(&self) -> Result<SurfaceType, Failure> {
        let count = self.next_surface_kind_call();
        self.replace_live_surface_slot();
        if *self.unrelated_panic.read() {
            std::panic::panic_any("Martensite application surface panic");
        }
        if *self.fail.read() && count == 3 {
            return Err(Failure::new(
                "isoline point callback=3; expected successful sample",
            ));
        }
        self.core.get_type()
    }
    fn plane(&self) -> Result<Plane3, Failure> {
        let core = &self.core;
        core.plane()
    }
    fn eval_d1(&self, u: f64, v: f64) -> Result<rustycad_tkg3d::geom::GeomSurfaceD1, Failure> {
        // Native final derivative evaluation delegates the unchanged parent surface.
        self.core.eval_d1(u, v)
    }
    fn u_continuity(&self) -> Result<rustycad_tkmath::geom_abs::Shape, Failure> {
        let core = &self.core;
        core.u_continuity()
    }
    fn v_continuity(&self) -> Result<rustycad_tkmath::geom_abs::Shape, Failure> {
        let core = &self.core;
        core.v_continuity()
    }
    fn is_u_periodic(&self) -> Result<bool, Failure> {
        let core = &self.core;
        core.is_u_periodic()
    }
    fn is_v_periodic(&self) -> Result<bool, Failure> {
        let core = &self.core;
        core.is_v_periodic()
    }
    fn is_u_closed(&self) -> Result<bool, Failure> {
        let core = &self.core;
        core.is_u_closed()
    }
    fn is_v_closed(&self) -> Result<bool, Failure> {
        let core = &self.core;
        core.is_v_closed()
    }
    fn u_period(&self) -> Result<f64, Failure> {
        let core = &self.core;
        core.u_period()
    }
    fn v_period(&self) -> Result<f64, Failure> {
        let core = &self.core;
        core.v_period()
    }
    fn eval_d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        // OCCT 3d097a0 marks BRep adaptor EvalD0 final; GetType is genuinely overridable.
        self.core.eval_d0(u, v)
    }
}
impl IsolineAdaptorFake {
    fn next_surface_kind_call(&self) -> i32 {
        let mut count = self.points.write();
        *count += 1;
        *count
    }
}

type IsolineSourceReplacement = (
    std::sync::Arc<SharedState<rustycad_tkernel::standard::Handle<dyn StdPrsIsolineSurfaceSource>>>,
    rustycad_tkernel::standard::Handle<dyn StdPrsIsolineSurfaceSource>,
);
impl IsolineAdaptorFake {
    pub(super) fn replace_source_on_kind_query(
        &self,
        slot: &std::sync::Arc<
            SharedState<rustycad_tkernel::standard::Handle<dyn StdPrsIsolineSurfaceSource>>,
        >,
        replacement: &rustycad_tkernel::standard::Handle<dyn StdPrsIsolineSurfaceSource>,
    ) {
        // Taking the armed hook during dispatch also breaks the deliberate temporary ownership cycle.
        *self.replacement.write() = Some((std::sync::Arc::clone(slot), replacement.clone()));
    }
    pub(super) fn reinitialize_core(&self, face: &rustycad_tkbrep::topods::Face) {
        let core = &self.core;
        core.initialize(face, true).unwrap();
    }
    fn replace_live_surface_slot(&self) {
        let replacement = self.replacement.write().take();
        if let Some((slot, source)) = replacement {
            *slot.write() = source;
        }
    }
}
