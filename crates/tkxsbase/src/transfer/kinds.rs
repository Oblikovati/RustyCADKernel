//! The two instances of the native transfer templates (process, actor and iterator): by
//! transient (`...ForTransient`) and by finder (`...ForFinder`); xsbase-design.md section 3.
use super::{TransferActorOps, TransferFindHasher, TransferFinder, TransferProcessOps};
use rustycad_tkernel::{
    ncollection::{DefaultHashPolicy, HashPolicy},
    standard::{Handle, RuntimeTyped, StandardTransient},
};
use std::fmt::Debug;
use std::io::{self, Write};

/// One instance of the transfer templates: its starting objects and their hashing.
pub trait TransferProcessKind: Sized + Send + Sync + 'static {
    /// `TheStart` (`occ::handle<Standard_Transient>` or `occ::handle<Transfer_Finder>`).
    type Start: Clone + Default + PartialEq + Debug + Send + Sync + 'static;
    /// `TheMapHasher` (`std::hash<occ::handle<Standard_Transient>>` or `Transfer_FindHasher`).
    type Hasher: HashPolicy<Self::Start> + Default + Send + Sync + 'static;
    /// The actor class seen through its virtual members (`dyn TransferActorOps<Self>`).
    type Actor: ?Sized + TransferActorOps<Self> + RuntimeTyped;
    /// The process class seen through its virtual members (`dyn TransferProcessOps<Self>`).
    type Process: ?Sized + TransferProcessOps<Self> + RuntimeTyped;
    /// The native name of the iterator class (in its messages).
    const ITERATOR_NAME: &'static str;
    /// Whether a starting object is null.
    fn is_null(start: &Self::Start) -> bool;
    /// The starting object as a transient (for `Interface_Check::SetEntity`).
    fn as_transient(start: &Self::Start) -> Handle<StandardTransient>;
    /// The base `PrintTrace(start, S)`: " Type:" and the dynamic type name (nothing for
    /// null).
    fn print_start(start: &Self::Start, s: &mut dyn Write) -> io::Result<()>;
}

/// The transient instance (`...ForTransient`): starts are transients hashed by address.
#[derive(Debug)]
pub struct TransferTransientKind;

impl TransferProcessKind for TransferTransientKind {
    type Start = Handle<StandardTransient>;
    type Hasher = DefaultHashPolicy;
    type Actor = dyn TransferActorOps<Self>;
    type Process = dyn TransferProcessOps<Self>;
    const ITERATOR_NAME: &'static str = "Transfer_IteratorOfProcessForTransient";
    fn is_null(start: &Self::Start) -> bool {
        start.is_null()
    }
    fn as_transient(start: &Self::Start) -> Handle<StandardTransient> {
        start.clone()
    }
    fn print_start(start: &Self::Start, s: &mut dyn Write) -> io::Result<()> {
        match start.dynamic_type() {
            Some(kind) => write!(s, " Type:{}", kind.name()),
            None => Ok(()),
        }
    }
}

/// The finder instance (`...ForFinder`): starts are finders hashed by `Transfer_FindHasher`.
#[derive(Debug)]
pub struct TransferFinderKind;

impl TransferProcessKind for TransferFinderKind {
    type Start = Handle<dyn TransferFinder>;
    type Hasher = TransferFindHasher;
    type Actor = dyn TransferActorOps<Self>;
    type Process = dyn TransferProcessOps<Self>;
    const ITERATOR_NAME: &'static str = "Transfer_IteratorOfProcessForFinder";
    fn is_null(start: &Self::Start) -> bool {
        start.is_null()
    }
    fn as_transient(start: &Self::Start) -> Handle<StandardTransient> {
        start.cast()
    }
    fn print_start(start: &Self::Start, s: &mut dyn Write) -> io::Result<()> {
        match start.get() {
            Some(finder) => write!(s, " Type:{}", finder.dynamic_type().name()),
            None => Ok(()),
        }
    }
}
