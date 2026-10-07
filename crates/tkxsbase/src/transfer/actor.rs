// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ActorOfProcessForTransient and
// Transfer_ActorOfProcessForFinder (.hxx, _0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The actors of the transfer processes: `Recognize` and `Transferring` are virtual, the
//! chain of actors (`SetNext`, `SetLast`) is common; xsbase-design.md section 3.
use super::{
    TransferBinder, TransferFinderKind, TransferProcessKind, TransferSimpleBinderOfTransient,
    TransferTransientKind,
};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    message::ProgressRange,
    standard::{Failure, Handle, SharedState, StandardTransient, Transient},
};

/// The private fields of an actor: the next actor of the chain and the "last" flag.
pub struct TransferActor<K: TransferProcessKind> {
    thenext: SharedState<Handle<K::Actor>>,
    thelast: SharedState<bool>,
}

impl<K: TransferProcessKind> Default for TransferActor<K> {
    /// The native constructor: no next actor, not last.
    fn default() -> Self {
        Self {
            thenext: SharedState::new(Handle::default()),
            thelast: SharedState::new(false),
        }
    }
}

/// The virtual members of `Transfer_ActorOfProcessForTransient` /
/// `Transfer_ActorOfProcessForFinder`; the non-virtual ones are provided.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferActorOfProcessForTransient, TransferActorOps};
/// use rustycad_tkernel::standard::Handle;
/// let first = Handle::new(TransferActorOfProcessForTransient::new());
/// let second = Handle::new(TransferActorOfProcessForTransient::new());
/// first.set_next(&second.clone().cast());
/// assert!(first.next().is_some() && !first.is_last());
/// ```
pub trait TransferActorOps<K: TransferProcessKind>: Transient {
    /// The embedded private fields.
    fn actor(&self) -> &TransferActor<K>;
    /// `Recognize(start)`: by default every object.
    fn recognize(&self, _start: &K::Start) -> bool {
        true
    }
    /// `Transferring(start, TP, theProgress)`: by default no result (a null binder).
    fn transferring(
        &self,
        _start: &K::Start,
        _tp: &Handle<K::Process>,
        _progress: &ProgressRange<'_>,
    ) -> Result<Handle<dyn TransferBinder>, Failure> {
        Ok(self.null_result())
    }
    /// `TransientResult(res)`: a simple binder of `res` (null for a null `res`).
    fn transient_result(
        &self,
        res: &Handle<StandardTransient>,
    ) -> Handle<TransferSimpleBinderOfTransient> {
        if res.is_null() {
            return Handle::default();
        }
        let binder = TransferSimpleBinderOfTransient::new();
        // a new binder is never "used": SetResult cannot fail
        let _ = binder.set_result(res);
        Handle::new(binder)
    }
    /// `NullResult`: a null binder.
    fn null_result(&self) -> Handle<dyn TransferBinder> {
        Handle::default()
    }
    /// `SetLast(mode = true)`: a last actor stays at the end of the chain.
    fn set_last(&self, mode: bool) {
        *self.actor().thelast.write() = mode;
    }
    /// `IsLast`.
    fn is_last(&self) -> bool {
        *self.actor().thelast.read()
    }
    /// `SetNext(next)`: inserts `next` in the chain, before the first "last" actor.
    fn set_next(&self, next: &Handle<K::Actor>) {
        let current = self.next();
        if current == *next {
            return;
        }
        let Some(live) = current.get() else {
            *self.actor().thenext.write() = next.clone();
            return;
        };
        if live.is_last() {
            next.set_next(&current);
            *self.actor().thenext.write() = next.clone();
            return;
        }
        live.set_next(next);
    }
    /// `Next`.
    fn next(&self) -> Handle<K::Actor> {
        self.actor().thenext.read().clone()
    }
}
impl_transient_interface!(
    dyn TransferActorOps<TransferTransientKind>,
    "Transfer_ActorOfProcessForTransient",
    StandardTransient
);
impl_transient_interface!(
    dyn TransferActorOps<TransferFinderKind>,
    "Transfer_ActorOfProcessForFinder",
    StandardTransient
);

/// `Transfer_ActorOfProcessForTransient` (concrete): recognizes everything, transfers
/// nothing.
#[derive(Default)]
pub struct TransferActorOfProcessForTransient {
    transient: StandardTransient,
    core: TransferActor<TransferTransientKind>,
}
impl_transient!(TransferActorOfProcessForTransient, "Transfer_ActorOfProcessForTransient", StandardTransient, transient, interfaces: [dyn TransferActorOps<TransferTransientKind>]);

impl TransferActorOfProcessForTransient {
    /// `Transfer_ActorOfProcessForTransient()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TransferActorOps<TransferTransientKind> for TransferActorOfProcessForTransient {
    fn actor(&self) -> &TransferActor<TransferTransientKind> {
        &self.core
    }
}

/// `Transfer_ActorOfProcessForFinder` (concrete): recognizes everything, transfers
/// nothing.
#[derive(Default)]
pub struct TransferActorOfProcessForFinder {
    transient: StandardTransient,
    core: TransferActor<TransferFinderKind>,
}
impl_transient!(TransferActorOfProcessForFinder, "Transfer_ActorOfProcessForFinder", StandardTransient, transient, interfaces: [dyn TransferActorOps<TransferFinderKind>]);

impl TransferActorOfProcessForFinder {
    /// `Transfer_ActorOfProcessForFinder()`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl TransferActorOps<TransferFinderKind> for TransferActorOfProcessForFinder {
    fn actor(&self) -> &TransferActor<TransferFinderKind> {
        &self.core
    }
}
