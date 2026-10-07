// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ActorOfTransientProcess and
// Transfer_ActorOfFinderProcess (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The actors of the transient and finder processes: `Transferring` calls the virtual
//! `Transfer` with the derived process, which by default wraps `TransferTransient`.
use super::{
    TransferActor, TransferActorOfProcessForFinder, TransferActorOfProcessForTransient,
    TransferActorOps, TransferBinder, TransferFinder, TransferFinderKind, TransferFinderProcess,
    TransferProcessKind, TransferTransientKind, TransferTransientMapper, TransferTransientProcess,
};
use crate::xs_algo::{XSAlgoParameterMap, XSAlgoProcessingFlags};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkshhealing::shape_process::ShapeProcessOperationsFlags;

/// The shape processing fields of the actors of the transient and finder processes.
#[derive(Default)]
pub struct TransferShapeProcessing {
    params: SharedState<XSAlgoParameterMap>,
    flags: SharedState<XSAlgoProcessingFlags>,
}

impl TransferShapeProcessing {
    /// `SetShapeFixParameters(theParameters)`.
    pub fn set_shape_fix_parameters(&self, parameters: XSAlgoParameterMap) {
        *self.params.write() = parameters;
    }
    /// `SetShapeFixParameters(theParameters, theAdditionalParameters = {})`: the shape
    /// fix parameters (replacing), then the additional ones (not replacing).
    pub fn set_shape_fix_parameters_from(
        &self,
        parameters: &crate::xs_algo::DEShapeFixParameters,
        additional: &XSAlgoParameterMap,
    ) {
        crate::xs_algo::XSAlgoShapeProcessor::set_shape_fix_parameters(
            parameters,
            additional,
            &mut self.params.write(),
        );
    }
    /// `GetShapeFixParameters`: a copy of the parameters.
    pub fn shape_fix_parameters(&self) -> XSAlgoParameterMap {
        self.params.read().clone()
    }
    /// `SetProcessingFlags(theFlags)` / `SetShapeProcessFlags(theFlags)`: the flags, set.
    pub fn set_flags(&self, flags: ShapeProcessOperationsFlags) {
        *self.flags.write() = (flags, true);
    }
    /// `GetProcessingFlags` / `GetShapeProcessFlags`.
    pub fn flags(&self) -> XSAlgoProcessingFlags {
        *self.flags.read()
    }
}

/// The virtual members of `Transfer_ActorOfTransientProcess` (`Transfer`,
/// `TransferTransient`) and its shape processing fields.
pub trait TransferActorOfTransientProcessOps: TransferActorOps<TransferTransientKind> {
    /// The shape processing fields.
    fn shape_processing(&self) -> &TransferShapeProcessing;
    /// `Transfer(start, TP, theProgress)`: a simple binder of `TransferTransient`, else
    /// null.
    fn transfer(
        &self,
        start: &Handle<StandardTransient>,
        tp: &Handle<TransferTransientProcess>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        let res = self.transfer_transient(start, tp, progress)?;
        if res.is_null() {
            return Ok(self.null_result());
        }
        Ok(self.transient_result(&res).cast())
    }
    /// `TransferTransient(start, TP, theProgress)`: by default null.
    fn transfer_transient(
        &self,
        _start: &Handle<StandardTransient>,
        _tp: &Handle<TransferTransientProcess>,
        _progress: &ProgressRange<'_>,
    ) -> Result<Handle<StandardTransient>, Failure> {
        Ok(Handle::default())
    }
}
impl_transient_interface!(
    dyn TransferActorOfTransientProcessOps,
    "Transfer_ActorOfTransientProcess",
    StandardTransient
);

/// `Transfer_ActorOfTransientProcess::Transferring`: `Transfer` with the process cast to
/// a transient process (null when it is not one); a derived actor's `transferring` calls it.
pub fn transferring_by_transient_process<A: TransferActorOfTransientProcessOps + ?Sized>(
    actor: &A,
    start: &Handle<StandardTransient>,
    tp: &Handle<<TransferTransientKind as TransferProcessKind>::Process>,
    progress: &ProgressRange<'_>,
) -> Result<Handle<dyn TransferBinder>, Failure> {
    actor.transfer(start, &tp.cast(), progress)
}

/// `Transfer_ActorOfTransientProcess` (concrete): transfers nothing by default; embeds
/// its `Transfer_ActorOfProcessForTransient` (registered as its RTTI parent).
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferActorOfTransientProcess, TransferActorOfTransientProcessOps};
/// let actor = TransferActorOfTransientProcess::new();
/// assert!(!actor.shape_processing().flags().1);
/// ```
#[derive(Default)]
pub struct TransferActorOfTransientProcess {
    base: TransferActorOfProcessForTransient,
    processing: TransferShapeProcessing,
}
impl_transient!(TransferActorOfTransientProcess, "Transfer_ActorOfTransientProcess", TransferActorOfProcessForTransient, base, interfaces: [dyn TransferActorOps<TransferTransientKind>, dyn TransferActorOfTransientProcessOps]);

impl TransferActorOfTransientProcess {
    /// `Transfer_ActorOfTransientProcess()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TransferActorOps<TransferTransientKind> for TransferActorOfTransientProcess {
    fn actor(&self) -> &TransferActor<TransferTransientKind> {
        self.base.actor()
    }
    fn transferring(
        &self,
        start: &Handle<StandardTransient>,
        tp: &Handle<<TransferTransientKind as TransferProcessKind>::Process>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        transferring_by_transient_process(self, start, tp, progress)
    }
}

impl TransferActorOfTransientProcessOps for TransferActorOfTransientProcess {
    fn shape_processing(&self) -> &TransferShapeProcessing {
        &self.processing
    }
}

/// The virtual members of `Transfer_ActorOfFinderProcess` (`Transfer`,
/// `TransferTransient`), its mode and its shape processing fields.
pub trait TransferActorOfFinderProcessOps: TransferActorOps<TransferFinderKind> {
    /// The shape processing fields.
    fn shape_processing(&self) -> &TransferShapeProcessing;
    /// The `themodetrans` field.
    fn mode_trans_state(&self) -> &SharedState<i32>;
    /// `ModeTrans`: the mode (natively a modifiable reference: the write guard).
    fn mode_trans(&self) -> std::sync::RwLockWriteGuard<'_, i32> {
        self.mode_trans_state().write()
    }
    /// `Transfer(start, FP, theProgress)`: for a transient mapper, a simple binder of
    /// `TransferTransient` of its value, else null.
    fn transfer(
        &self,
        start: &Handle<dyn TransferFinder>,
        fp: &Handle<TransferFinderProcess>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        let mapper = start.cast::<TransferTransientMapper>();
        let Some(live) = mapper.get() else {
            return Ok(self.null_result());
        };
        let res = self.transfer_transient(live.value(), fp, progress)?;
        if res.is_null() {
            return Ok(self.null_result());
        }
        Ok(self.transient_result(&res).cast())
    }
    /// `TransferTransient(start, FP, theProgress)`: by default null.
    fn transfer_transient(
        &self,
        _start: &Handle<StandardTransient>,
        _fp: &Handle<TransferFinderProcess>,
        _progress: &ProgressRange<'_>,
    ) -> Result<Handle<StandardTransient>, Failure> {
        Ok(Handle::default())
    }
}
impl_transient_interface!(
    dyn TransferActorOfFinderProcessOps,
    "Transfer_ActorOfFinderProcess",
    StandardTransient
);

/// `Transfer_ActorOfFinderProcess::Transferring`: `Transfer` with the process cast to a
/// finder process (null when it is not one); a derived actor's `transferring` calls it.
pub fn transferring_by_finder_process<A: TransferActorOfFinderProcessOps + ?Sized>(
    actor: &A,
    start: &Handle<dyn TransferFinder>,
    tp: &Handle<<TransferFinderKind as TransferProcessKind>::Process>,
    progress: &ProgressRange<'_>,
) -> Result<Handle<dyn TransferBinder>, Failure> {
    actor.transfer(start, &tp.cast(), progress)
}

/// `Transfer_ActorOfFinderProcess` (concrete): transfers nothing by default, mode 0;
/// embeds its `Transfer_ActorOfProcessForFinder` (registered as its RTTI parent).
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferActorOfFinderProcess, TransferActorOfFinderProcessOps};
/// let actor = TransferActorOfFinderProcess::new();
/// *actor.mode_trans() = 2;
/// assert_eq!(*actor.mode_trans(), 2);
/// ```
#[derive(Default)]
pub struct TransferActorOfFinderProcess {
    base: TransferActorOfProcessForFinder,
    themodetrans: SharedState<i32>,
    processing: TransferShapeProcessing,
}
impl_transient!(TransferActorOfFinderProcess, "Transfer_ActorOfFinderProcess", TransferActorOfProcessForFinder, base, interfaces: [dyn TransferActorOps<TransferFinderKind>, dyn TransferActorOfFinderProcessOps]);

impl TransferActorOfFinderProcess {
    /// `Transfer_ActorOfFinderProcess()`: mode 0.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TransferActorOps<TransferFinderKind> for TransferActorOfFinderProcess {
    fn actor(&self) -> &TransferActor<TransferFinderKind> {
        self.base.actor()
    }
    fn transferring(
        &self,
        start: &Handle<dyn TransferFinder>,
        tp: &Handle<<TransferFinderKind as TransferProcessKind>::Process>,
        progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        transferring_by_finder_process(self, start, tp, progress)
    }
}

impl TransferActorOfFinderProcessOps for TransferActorOfFinderProcess {
    fn shape_processing(&self) -> &TransferShapeProcessing {
        &self.processing
    }
    fn mode_trans_state(&self) -> &SharedState<i32> {
        &self.themodetrans
    }
}
