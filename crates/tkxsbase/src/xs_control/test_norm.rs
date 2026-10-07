//! A fake norm controller for the unit tests of XSControl: the fake work library and
//! protocol, a reading actor (for the entities named "s...", a new compound holding one
//! shared vertex; the entity itself else; a failure for "fail") and a writing actor (a new entity "written" for any
//! object).
use super::{XSControlController, XSControlControllerBase};
use crate::if_select::test_work_library::fake_work_library;
use crate::interface::InterfaceInterfaceModel;
use crate::interface::test_norm::{FakeEntity, fake_entity, fake_model, fake_protocol};
use crate::transfer::{
    TransferActor, TransferActorOfFinderProcess, TransferActorOfFinderProcessOps,
    TransferActorOfTransientProcess, TransferActorOfTransientProcessOps, TransferActorOps,
    TransferBinder, TransferFinder, TransferFinderKind, TransferFinderProcess, TransferProcessOps,
    TransferShapeProcessing, TransferTransientKind, TransferTransientProcess,
    transferring_by_finder_process, transferring_by_transient_process,
};
use crate::transfer_brep::TransferBRepShapeBinder;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Builder, Compound, Shape, Vertex};
use rustycad_tkernel::{
    impl_transient,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use std::sync::OnceLock;

/// The vertex every compound of the reading actor holds.
pub(super) fn shared_vertex() -> Shape {
    static VERTEX: OnceLock<Shape> = OnceLock::new();
    VERTEX
        .get_or_init(|| {
            let mut v = Vertex::default();
            BRepBuilder::default().make_vertex(&mut v);
            v.into_shape()
        })
        .clone()
}

/// A new compound (a distinct shape at each call) holding the shared vertex.
pub(super) fn vertex_compound() -> Shape {
    let mut c = Compound::default();
    Builder.make_compound(&mut c);
    let mut shape = c.into_shape();
    Builder
        .add(&mut shape, &shared_vertex())
        .expect("a compound holds a vertex");
    shape
}

/// The name of a fake entity (empty for another object).
fn entity_name(ent: &Handle<StandardTransient>) -> String {
    ent.cast::<FakeEntity>()
        .get()
        .map(|e| e.name.read().clone())
        .unwrap_or_default()
}

/// The reading actor of the fake norm.
#[derive(Default)]
pub(super) struct FakeReadActor {
    base: TransferActorOfTransientProcess,
}
impl_transient!(FakeReadActor, "Fake_ReadActor", TransferActorOfTransientProcess, base, interfaces: [dyn TransferActorOps<TransferTransientKind>, dyn TransferActorOfTransientProcessOps]);

impl TransferActorOps<TransferTransientKind> for FakeReadActor {
    fn actor(&self) -> &TransferActor<TransferTransientKind> {
        self.base.actor()
    }
    fn recognize(&self, start: &Handle<StandardTransient>) -> bool {
        start.cast::<FakeEntity>().is_some()
    }
    fn transferring(
        &self,
        start: &Handle<StandardTransient>,
        tp: &Handle<dyn TransferProcessOps<TransferTransientKind>>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        transferring_by_transient_process(self, start, tp, progress)
    }
}

impl TransferActorOfTransientProcessOps for FakeReadActor {
    fn shape_processing(&self) -> &TransferShapeProcessing {
        self.base.shape_processing()
    }
    fn transfer(
        &self,
        start: &Handle<StandardTransient>,
        _tp: &Handle<TransferTransientProcess>,
        _progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        let name = entity_name(start);
        if name == "fail" {
            return Err(Failure::new("fake read failure"));
        }
        if name.starts_with('s') {
            return Ok(
                Handle::new(TransferBRepShapeBinder::with_result(&vertex_compound())).cast(),
            );
        }
        Ok(self.transient_result(start).cast())
    }
}

/// The writing actor of the fake norm.
#[derive(Default)]
pub(super) struct FakeWriteActor {
    base: TransferActorOfFinderProcess,
}
impl_transient!(FakeWriteActor, "Fake_WriteActor", TransferActorOfFinderProcess, base, interfaces: [dyn TransferActorOps<TransferFinderKind>, dyn TransferActorOfFinderProcessOps]);

impl TransferActorOps<TransferFinderKind> for FakeWriteActor {
    fn actor(&self) -> &TransferActor<TransferFinderKind> {
        self.base.actor()
    }
    fn recognize(&self, _start: &Handle<dyn TransferFinder>) -> bool {
        true
    }
    fn transferring(
        &self,
        start: &Handle<dyn TransferFinder>,
        tp: &Handle<dyn TransferProcessOps<TransferFinderKind>>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        transferring_by_finder_process(self, start, tp, progress)
    }
}

impl TransferActorOfFinderProcessOps for FakeWriteActor {
    fn shape_processing(&self) -> &TransferShapeProcessing {
        self.base.shape_processing()
    }
    fn mode_trans_state(&self) -> &SharedState<i32> {
        self.base.mode_trans_state()
    }
    fn transfer(
        &self,
        _start: &Handle<dyn TransferFinder>,
        _fp: &Handle<TransferFinderProcess>,
        _progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        Ok(self.transient_result(&fake_entity("written", &[])).cast())
    }
}

/// The fake controller.
pub(super) struct FakeController {
    transient: StandardTransient,
    base: XSControlControllerBase,
}
impl_transient!(FakeController, "Fake_Controller", StandardTransient, transient, interfaces: [dyn XSControlController]);

impl XSControlController for FakeController {
    fn controller_base(&self) -> &XSControlControllerBase {
        &self.base
    }
    fn new_model(&self) -> Handle<dyn InterfaceInterfaceModel> {
        fake_model()
    }
}

/// A new fake controller named `long`/`short`, bound to its handle, with the fake
/// library, protocol and actors (not recorded).
pub(super) fn fake_controller(long: &str, short: &str) -> Handle<dyn XSControlController> {
    let base = XSControlControllerBase::new(long, short).expect("the standard statics are defined");
    base.set_adaptor_library(&fake_work_library());
    base.set_adaptor_protocol(&fake_protocol());
    base.set_adaptor_read(&Handle::new(FakeReadActor::default()).cast());
    base.set_adaptor_write(&Handle::new(FakeWriteActor::default()).cast());
    let ctl: Handle<dyn XSControlController> = Handle::new(FakeController {
        transient: StandardTransient::default(),
        base,
    })
    .cast();
    ctl.controller_base().bind_self(&ctl);
    ctl
}
