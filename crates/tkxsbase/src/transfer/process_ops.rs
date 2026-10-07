// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ProcessForTransient.hxx and
// Transfer_ProcessForFinder.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871: the classes
// and their virtual members.

//! The virtual members of the processes ([`TransferProcessOps`]) and the two concrete
//! process classes; xsbase-design.md section 3.
use super::{TransferFinderKind, TransferProcess, TransferProcessKind, TransferTransientKind};
use rustycad_tkernel::{
    impl_transient, impl_transient_interface,
    message::Messenger,
    standard::{Handle, RuntimeTyped, StandardTransient, Transient},
};
use std::io;
use std::ops::Deref;
use std::sync::Arc;

/// The virtual members of a process (`PrintTrace`, `CheckNum`) and its common body;
/// a derived process overrides them, and the process dispatches its own calls to them.
pub trait TransferProcessOps<K: TransferProcessKind>: Transient {
    /// The common body.
    fn process(&self) -> &TransferProcess<K>;
    /// `PrintTrace(start, S)`: by default " Type:" and the dynamic type name.
    fn print_trace(&self, start: &K::Start, s: &mut dyn io::Write) -> io::Result<()> {
        K::print_start(start, s)
    }
    /// `CheckNum(start)`: the number of `start` in the check lists, by default 0 (the map
    /// index is used then).
    fn check_num(&self, _start: &K::Start) -> i32 {
        0
    }
}
impl_transient_interface!(
    dyn TransferProcessOps<TransferTransientKind>,
    "Transfer_ProcessForTransient",
    StandardTransient
);
impl_transient_interface!(
    dyn TransferProcessOps<TransferFinderKind>,
    "Transfer_ProcessForFinder",
    StandardTransient
);

/// `Transfer_ProcessForTransient`: a process over transients; dereferences to its
/// [`TransferProcess`].
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferProcessForTransient;
/// let tp = TransferProcessForTransient::new(10000);
/// assert_eq!((tp.trace_level(), tp.nb_mapped(), tp.error_handle()), (0, 0, true));
/// ```
pub struct TransferProcessForTransient {
    transient: StandardTransient,
    core: TransferProcess<TransferTransientKind>,
}
impl_transient!(TransferProcessForTransient, "Transfer_ProcessForTransient", StandardTransient, transient, interfaces: [dyn TransferProcessOps<TransferTransientKind>]);

impl TransferProcessForTransient {
    /// The unbound object, for the derived processes (they bind their own handle).
    pub fn unbound(core: TransferProcess<TransferTransientKind>) -> Self {
        Self {
            transient: StandardTransient::default(),
            core,
        }
    }
    /// `Transfer_ProcessForTransient(nb = 10000)`.
    pub fn new(nb: i32) -> Handle<Self> {
        bind_new(Self::unbound(TransferProcess::new(nb)))
    }
    /// `Transfer_ProcessForTransient(printer, nb = 10000)`: trace level 1.
    pub fn with_messenger(printer: Option<Arc<Messenger>>, nb: i32) -> Handle<Self> {
        bind_new(Self::unbound(TransferProcess::with_messenger(printer, nb)))
    }
}

impl Deref for TransferProcessForTransient {
    type Target = TransferProcess<TransferTransientKind>;
    fn deref(&self) -> &TransferProcess<TransferTransientKind> {
        &self.core
    }
}

impl TransferProcessOps<TransferTransientKind> for TransferProcessForTransient {
    fn process(&self) -> &TransferProcess<TransferTransientKind> {
        &self.core
    }
}

/// `Transfer_ProcessForFinder`: a process over finders; dereferences to its
/// [`TransferProcess`].
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferProcessForFinder;
/// let fp = TransferProcessForFinder::with_messenger(None, 100);
/// assert_eq!(fp.trace_level(), 1);
/// ```
pub struct TransferProcessForFinder {
    transient: StandardTransient,
    core: TransferProcess<TransferFinderKind>,
}
impl_transient!(TransferProcessForFinder, "Transfer_ProcessForFinder", StandardTransient, transient, interfaces: [dyn TransferProcessOps<TransferFinderKind>]);

impl TransferProcessForFinder {
    /// The unbound object, for the derived processes (they bind their own handle).
    pub fn unbound(core: TransferProcess<TransferFinderKind>) -> Self {
        Self {
            transient: StandardTransient::default(),
            core,
        }
    }
    /// `Transfer_ProcessForFinder(nb = 10000)`.
    pub fn new(nb: i32) -> Handle<Self> {
        bind_new(Self::unbound(TransferProcess::new(nb)))
    }
    /// `Transfer_ProcessForFinder(printer, nb = 10000)`: trace level 1.
    pub fn with_messenger(printer: Option<Arc<Messenger>>, nb: i32) -> Handle<Self> {
        bind_new(Self::unbound(TransferProcess::with_messenger(printer, nb)))
    }
}

impl Deref for TransferProcessForFinder {
    type Target = TransferProcess<TransferFinderKind>;
    fn deref(&self) -> &TransferProcess<TransferFinderKind> {
        &self.core
    }
}

impl TransferProcessOps<TransferFinderKind> for TransferProcessForFinder {
    fn process(&self) -> &TransferProcess<TransferFinderKind> {
        &self.core
    }
}

/// Wraps a process in its handle and binds the handle as its `this`;
/// `bind_new(TransferProcessForTransient::unbound(core))`.
pub fn bind_new<K, P>(process: P) -> Handle<P>
where
    K: TransferProcessKind,
    P: TransferProcessOps<K> + RuntimeTyped + 'static,
{
    let handle = Handle::new(process);
    let ops: Handle<K::Process> = handle.cast();
    handle.process().bind_self(&ops);
    handle
}
