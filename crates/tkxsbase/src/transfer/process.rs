// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ProcessForTransient and
// Transfer_ProcessForFinder (.hxx, _0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871:
// the state, the constructors and the map of binders.

//! `TransferProcess<K>`: the common body of `Transfer_ProcessForTransient` and
//! `Transfer_ProcessForFinder` (the two sources are one template instantiated twice);
//! xsbase-design.md section 3. The members are split over `process_*.rs`.
use super::{
    TransferActorOps, TransferBinder, TransferProcessKind, TransferStatusResult,
    TransferVoidBinder, transfer_failure,
};
use rustycad_tkernel::{
    message::{Message, Messenger},
    ncollection::{IndexedDataMap, IndexedMap},
    standard::{Failure, Handle, RuntimeTyped, SharedState, WeakHandle},
};
use std::sync::{Arc, OnceLock, RwLockReadGuard, RwLockWriteGuard};

/// The map of the starting objects to their binders.
pub(super) type BinderMap<K> = IndexedDataMap<
    <K as TransferProcessKind>::Start,
    Handle<dyn TransferBinder>,
    <K as TransferProcessKind>::Hasher,
>;

/// The private fields of the native process.
pub(super) struct ProcessState<K: TransferProcessKind> {
    pub(super) theerrh: bool,
    pub(super) thetrace: i32,
    pub(super) themessenger: Arc<Messenger>,
    pub(super) thelevel: i32,
    pub(super) therootl: i32,
    pub(super) therootm: bool,
    pub(super) theroots: IndexedMap<i32>,
    pub(super) thelastobj: K::Start,
    pub(super) thelastbnd: Handle<dyn TransferBinder>,
    pub(super) theindex: i32,
    pub(super) theactor: Handle<K::Actor>,
    pub(super) themap: BinderMap<K>,
}

/// The common body of the transfer processes, embedded by
/// [`TransferProcessForTransient`](super::TransferProcessForTransient) and
/// [`TransferProcessForFinder`](super::TransferProcessForFinder) which dereference to it.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferBinder, TransferProcessForTransient};
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let tp = TransferProcessForTransient::new(100);
/// let ent = Handle::new(StandardTransient::default());
/// tp.bind_transient(&ent, &Handle::new(StandardTransient::default())).unwrap();
/// assert!(tp.is_bound(&ent) && tp.nb_mapped() == 1);
/// ```
pub struct TransferProcess<K: TransferProcessKind> {
    state: SharedState<ProcessState<K>>,
    this: OnceLock<WeakHandle<K::Process>>,
}

impl<K: TransferProcessKind> TransferProcess<K> {
    /// `Transfer_ProcessFor...(nb = 10000)`: the default messenger, trace level 0.
    pub fn new(nb: i32) -> Self {
        Self::build(Message::default_messenger().clone(), 0, nb)
    }
    /// `Transfer_ProcessFor...(printer, nb = 10000)`: trace level 1, the default messenger
    /// for `None`.
    pub fn with_messenger(printer: Option<Arc<Messenger>>, nb: i32) -> Self {
        let messenger = printer.unwrap_or_else(|| Message::default_messenger().clone());
        Self::build(messenger, 1, nb)
    }
    fn build(themessenger: Arc<Messenger>, thetrace: i32, nb: i32) -> Self {
        let state = ProcessState {
            theerrh: true,
            thetrace,
            themessenger,
            thelevel: 0,
            therootl: 0,
            therootm: false,
            theroots: IndexedMap::default(),
            thelastobj: K::Start::default(),
            thelastbnd: Handle::default(),
            theindex: 0,
            theactor: Handle::default(),
            themap: BinderMap::<K>::new(nb.max(1) as usize),
        };
        Self {
            state: SharedState::new(state),
            this: OnceLock::new(),
        }
    }
    /// Records the owning handle (natively `this`): the actors receive it and the virtual
    /// `PrintTrace` / `CheckNum` dispatch through it. The constructors of the concrete
    /// processes call it; `tp.bind_self(&handle)`.
    pub fn bind_self(&self, process: &Handle<K::Process>) {
        let _ = self.this.set(process.downgrade());
    }
    /// The handle `this` converts to natively; null before `bind_self`.
    pub fn this(&self) -> Handle<K::Process> {
        self.this.get().map(WeakHandle::upgrade).unwrap_or_default()
    }
    pub(super) fn read(&self) -> RwLockReadGuard<'_, ProcessState<K>> {
        self.state.read()
    }
    pub(super) fn write(&self) -> RwLockWriteGuard<'_, ProcessState<K>> {
        self.state.write()
    }
    /// `Clear`: no binder, no root, level 0.
    pub fn clear(&self) {
        let mut st = self.write();
        st.thelevel = 0;
        st.therootl = 0;
        st.theroots.clear(true);
        st.themap.clear(true);
        st.theindex = 0;
        st.thelastobj = K::Start::default();
        st.thelastbnd.nullify();
    }
    /// `Clean`: removes the null binders and renumbers the roots.
    pub fn clean(&self) {
        let mut st = self.write();
        let (newmap, unbs) = without_null_binders::<K>(&st.themap);
        if unbs.is_empty() {
            return;
        }
        st.themap = newmap;
        let mut roots = IndexedMap::default();
        for root in st.theroots.iter() {
            let k = usize::try_from(*root)
                .ok()
                .and_then(|j| unbs.get(j))
                .copied()
                .unwrap_or(0);
            if k != 0 {
                roots.add(k);
            }
        }
        st.theroots = roots;
        st.thelastobj = K::Start::default();
        st.thelastbnd.nullify();
        st.theindex = 0;
    }
    /// `Resize(nb)`: more buckets when `nb` exceeds their count.
    pub fn resize(&self, nb: i32) {
        let mut st = self.write();
        if nb > 0 && nb as usize > st.themap.bucket_count() {
            st.themap.resize(nb as usize);
        }
    }
    /// `SetActor(actor)`: the first actor, or chained before the first "last" one.
    pub fn set_actor(&self, actor: &Handle<K::Actor>) {
        let current = self.actor();
        if current == *actor {
            return;
        }
        let Some(live) = current.get().filter(|_| actor.is_some()) else {
            self.write().theactor = actor.clone();
            return;
        };
        if live.is_last() {
            actor.set_next(&current);
            self.write().theactor = actor.clone();
            return;
        }
        live.set_next(actor);
    }
    /// `Actor`.
    pub fn actor(&self) -> Handle<K::Actor> {
        self.read().theactor.clone()
    }
    /// `Find(start)`: the binder of `start` (null when not mapped).
    pub fn find(&self, start: &K::Start) -> Handle<dyn TransferBinder> {
        let st = self.read();
        if st.thelastobj == *start && st.theindex > 0 {
            return st.thelastbnd.clone();
        }
        let index = st.themap.find_index(start);
        if index == 0 {
            return Handle::default();
        }
        st.themap[index].clone()
    }
    /// `IsBound(start)`: mapped with a result.
    pub fn is_bound(&self, start: &K::Start) -> bool {
        self.find(start).get().is_some_and(|b| b.has_result())
    }
    /// `IsAlreadyUsed(start)`: `Transfer_TransferFailure` when `start` is not mapped.
    pub fn is_already_used(&self, start: &K::Start) -> Result<bool, Failure> {
        let binder = self.find(start);
        let Some(live) = binder.get() else {
            let level = self.nesting_level();
            self.start_trace(&binder, start, level, 4);
            return Err(transfer_failure(
                "TransferProcess : IsAlreadyUsed, transfer not done cannot be used...",
            ));
        };
        Ok(live.status() == TransferStatusResult::StatusUsed)
    }
    /// `FindAndMask(start)` (private): `Find` that records `start` as the last object.
    pub(super) fn find_and_mask(&self, start: &K::Start) -> Handle<dyn TransferBinder> {
        let mut st = self.write();
        if st.thelastobj == *start && st.theindex > 0 {
            return st.thelastbnd.clone();
        }
        st.thelastobj = start.clone();
        st.theindex = st.themap.find_index(start) as i32;
        st.thelastbnd = if st.theindex > 0 {
            st.themap[st.theindex as usize].clone()
        } else {
            Handle::default()
        };
        st.thelastbnd.clone()
    }
    /// `Bind(start, binder)`: a former void binder gives its check; a former used one is
    /// `Transfer_TransferFailure`; another former one gives its messages.
    pub fn bind(
        &self,
        start: &K::Start,
        binder: &Handle<dyn TransferBinder>,
    ) -> Result<(), Failure> {
        let Some(live) = binder.get() else {
            return Ok(());
        };
        let former = self.find_and_mask(start);
        if let Some(old) = former.get() {
            if std::ptr::eq(old.dynamic_type(), TransferVoidBinder::static_type()) {
                live.merge(&former);
            } else if old.status() == TransferStatusResult::StatusUsed {
                let level = self.nesting_level();
                self.start_trace(&former, start, level, 4);
                return Err(transfer_failure("TransferProcess : Bind, already Bound"));
            } else {
                if self.trace_level() > 2 {
                    let level = self.nesting_level();
                    self.start_trace(&former, start, level, 5);
                }
                live.c_check().get_messages(&old.check());
            }
        }
        self.store_binder(start, binder);
        Ok(())
    }
    /// The end of `Bind`: adds `start` (not masked) or replaces the masked binder.
    fn store_binder(&self, start: &K::Start, binder: &Handle<dyn TransferBinder>) {
        let mut st = self.write();
        if st.theindex == 0 {
            st.theindex = st.themap.add(start.clone(), binder.clone()) as i32;
        } else {
            let index = st.theindex as usize;
            st.themap[index] = binder.clone();
        }
        st.thelastbnd = binder.clone();
    }
    /// `Rebind(start, binder)`: the same as `Bind`.
    pub fn rebind(
        &self,
        start: &K::Start,
        binder: &Handle<dyn TransferBinder>,
    ) -> Result<(), Failure> {
        self.bind(start, binder)
    }
    /// `Unbind(start)`: forgets `start` as a root. Native quirk kept: the binder stays in
    /// the map (it is replaced by itself), and a void binder is not unbound at all.
    pub fn unbind(&self, start: &K::Start) -> bool {
        let former = self.find_and_mask(start);
        let mut st = self.write();
        if st.theindex == 0 || former.is_null() {
            return false;
        }
        if former
            .get()
            .is_some_and(|f| std::ptr::eq(f.dynamic_type(), TransferVoidBinder::static_type()))
        {
            return true;
        }
        let index = st.theindex;
        st.themap[index as usize] = st.thelastbnd.clone();
        if st.theroots.contains(&index) {
            let kept: Vec<i32> = st
                .theroots
                .iter()
                .copied()
                .filter(|r| *r != index)
                .collect();
            st.theroots.clear(true);
            kept.into_iter().for_each(|r| {
                st.theroots.add(r);
            });
        }
        st.thelastobj = K::Start::default();
        st.thelastbnd.nullify();
        st.theindex = 0;
        true
    }
    /// `FindElseBind(start)`: the binder of `start`, else a new void binder bound to it.
    pub fn find_else_bind(&self, start: &K::Start) -> Handle<dyn TransferBinder> {
        let binder = self.find_and_mask(start);
        if binder.is_some() {
            return binder;
        }
        self.bind_void(start)
    }
    /// Binds a new void binder to an unmapped (masked) `start`: `Bind` cannot fail then.
    pub(super) fn bind_void(&self, start: &K::Start) -> Handle<dyn TransferBinder> {
        let binder: Handle<dyn TransferBinder> = Handle::new(TransferVoidBinder::new()).cast();
        self.store_binder(start, &binder);
        binder
    }
    /// `SetMessenger(messenger)`: the default messenger for `None`.
    pub fn set_messenger(&self, messenger: Option<Arc<Messenger>>) {
        self.write().themessenger =
            messenger.unwrap_or_else(|| Message::default_messenger().clone());
    }
    /// `Messenger`.
    pub fn messenger(&self) -> Arc<Messenger> {
        self.read().themessenger.clone()
    }
    /// `SetTraceLevel(tracelev)`.
    pub fn set_trace_level(&self, tracelev: i32) {
        self.write().thetrace = tracelev;
    }
    /// `TraceLevel`.
    pub fn trace_level(&self) -> i32 {
        self.read().thetrace
    }
    /// `SetErrorHandle(err)`: whether `Transferring` catches the failures of the actors.
    pub fn set_error_handle(&self, err: bool) {
        self.write().theerrh = err;
    }
    /// `ErrorHandle`.
    pub fn error_handle(&self) -> bool {
        self.read().theerrh
    }
}

/// The map without its null binders and the new index of each old one (from 1, 0 for a
/// removed one); empty offsets when there is no null binder.
fn without_null_binders<K: TransferProcessKind>(map: &BinderMap<K>) -> (BinderMap<K>, Vec<i32>) {
    let nb = map.extent();
    let mut newmap = BinderMap::<K>::new(nb * 2);
    if map.iter().all(|binder| binder.is_some()) {
        return (newmap, Vec::new());
    }
    let mut unbs = vec![0_i32];
    for (ent, binder) in
        (1..=nb).filter_map(|i| Some((map.find_key(i).ok()?, map.find_from_index(i).ok()?)))
    {
        unbs.push(if binder.is_some() {
            newmap.add(ent.clone(), binder.clone()) as i32
        } else {
            0
        });
    }
    (newmap, unbs)
}
