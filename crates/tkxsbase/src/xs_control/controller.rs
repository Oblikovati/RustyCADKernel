// Rust port of DataExchange/TKXSBase/XSControl/XSControl_Controller (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XSControl_Controller`: the description of a norm for the exchange (its names,
//! protocol, work library, reading and writing actors, session items and parameters),
//! recorded by name for the work sessions. `Customise` is in `controller_customise`.
use super::XSControlWorkSession;
use crate::if_select::{IFSelectGeneralModifier, IFSelectReturnStatus, IFSelectWorkLibrary};
use crate::interface::{
    InterfaceInterfaceModel, InterfaceModelEntities, InterfaceProtocol, InterfaceStatic,
};
use crate::transfer::{
    TransferActorOfFinderProcessOps, TransferActorOfTransientProcessOps, TransferFinder,
    TransferFinderProcess, TransferSimpleBinderOfTransient, TransferTransientMapper,
};
use crate::transfer_brep::TransferBRepShapeMapper;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient_interface,
    message::ProgressRange,
    ncollection::DataMap,
    standard::{
        Failure, Handle, RuntimeTyped, SharedState, StandardTransient, Transient, WeakHandle,
        exceptions,
    },
    tcollection::{AsciiString, HAsciiString},
};
use std::sync::{OnceLock, PoisonError, RwLock, RwLockWriteGuard};

/// The native static `listad`: the controllers by recorded name.
fn recorded_controllers()
-> RwLockWriteGuard<'static, DataMap<AsciiString, Handle<dyn XSControlController>>> {
    static LISTAD: OnceLock<RwLock<DataMap<AsciiString, Handle<dyn XSControlController>>>> =
        OnceLock::new();
    LISTAD
        .get_or_init(|| RwLock::new(DataMap::default()))
        .write()
        .unwrap_or_else(PoisonError::into_inner)
}

/// The modes of the shape writing and their helps (`myModeWriteShapeN`): the lower bound
/// and the helps from it.
type ModeWriteHelps = Option<(i32, Vec<Handle<HAsciiString>>)>;

/// The fields of `XSControl_Controller` (the protected ones, set by the norms, through
/// the `set_adaptor_*` members).
pub struct XSControlControllerBase {
    names: SharedState<(AsciiString, AsciiString)>,
    adaptor_library: SharedState<Handle<dyn IFSelectWorkLibrary>>,
    adaptor_protocol: SharedState<Handle<dyn InterfaceProtocol>>,
    adaptor_read: SharedState<Handle<dyn TransferActorOfTransientProcessOps>>,
    adaptor_write: SharedState<Handle<dyn TransferActorOfFinderProcessOps>>,
    adaptor_session: SharedState<DataMap<AsciiString, Handle<StandardTransient>>>,
    adaptor_applied: SharedState<Vec<Handle<StandardTransient>>>,
    params: SharedState<(Vec<Handle<StandardTransient>>, Vec<i32>)>,
    mode_write_shape_n: SharedState<ModeWriteHelps>,
    this: OnceLock<WeakHandle<dyn XSControlController>>,
}

impl XSControlControllerBase {
    /// `XSControl_Controller(theLongName, theShortName)` (protected): the standard static
    /// parameters defined, the precision ones traced for reading (5) and writing (6).
    ///
    /// ```
    /// use rustycad_tkxsbase::xs_control::XSControlControllerBase;
    /// let base = XSControlControllerBase::new("long", "short").unwrap();
    /// assert_eq!(base.params().0.len(), 4);
    /// ```
    pub fn new(long_name: &str, short_name: &str) -> Result<Self, Failure> {
        let base = Self {
            names: SharedState::new((AsciiString::new(short_name), AsciiString::new(long_name))),
            adaptor_library: SharedState::default(),
            adaptor_protocol: SharedState::default(),
            adaptor_read: SharedState::default(),
            adaptor_write: SharedState::default(),
            adaptor_session: SharedState::default(),
            adaptor_applied: SharedState::default(),
            params: SharedState::default(),
            mode_write_shape_n: SharedState::default(),
            this: OnceLock::new(),
        };
        InterfaceStatic::standards()?;
        for (name, used) in [
            ("read.precision.mode", 5),
            ("read.precision.val", 5),
            ("write.precision.mode", 6),
            ("write.precision.val", 6),
        ] {
            base.trace_static(name, used);
        }
        Ok(base)
    }
    /// Records the handle of the controller (natively `this`), which `Record` registers.
    pub fn bind_self(&self, controller: &Handle<dyn XSControlController>) {
        let _ = self.this.set(controller.downgrade());
    }
    /// The handle `this` converts to natively; `Standard_NullObject` before `bind_self`.
    pub fn this(&self) -> Result<Handle<dyn XSControlController>, Failure> {
        self.this
            .get()
            .map(WeakHandle::upgrade)
            .filter(|h| h.is_some())
            .ok_or_else(|| {
                exceptions::null_object(
                    "XSControl_Controller: controller not bound to its handle; expected bind_self",
                )
            })
    }
    /// `TraceStatic(theName, theUse)` (protected): a static parameter traced for a use
    /// (nothing for an unknown one).
    pub fn trace_static(&self, name: &str, used: i32) {
        let val = InterfaceStatic::static_named(name);
        if val.is_null() {
            return;
        }
        let mut params = self.params.write();
        params.0.push(val.cast());
        params.1.push(used);
    }
    /// The traced parameters and their uses (`myParams`, `myParamUses`).
    pub fn params(&self) -> (Vec<Handle<StandardTransient>>, Vec<i32>) {
        self.params.read().clone()
    }
    /// The protected `myAdaptorLibrary` set.
    pub fn set_adaptor_library(&self, lib: &Handle<dyn IFSelectWorkLibrary>) {
        *self.adaptor_library.write() = lib.clone();
    }
    /// The protected `myAdaptorProtocol` set.
    pub fn set_adaptor_protocol(&self, protocol: &Handle<dyn InterfaceProtocol>) {
        *self.adaptor_protocol.write() = protocol.clone();
    }
    /// The protected `myAdaptorRead` set.
    pub fn set_adaptor_read(&self, actor: &Handle<dyn TransferActorOfTransientProcessOps>) {
        *self.adaptor_read.write() = actor.clone();
    }
    /// The protected `myAdaptorWrite` set.
    pub fn set_adaptor_write(&self, actor: &Handle<dyn TransferActorOfFinderProcessOps>) {
        *self.adaptor_write.write() = actor.clone();
    }
    /// The protected `myAdaptorRead`.
    pub fn adaptor_read(&self) -> Handle<dyn TransferActorOfTransientProcessOps> {
        self.adaptor_read.read().clone()
    }
    /// The protected `myAdaptorWrite`.
    pub fn adaptor_write(&self) -> Handle<dyn TransferActorOfFinderProcessOps> {
        self.adaptor_write.read().clone()
    }
    /// The session items (`myAdaptorSession`), in the native map order.
    pub(super) fn session_items(&self) -> Vec<(AsciiString, Handle<StandardTransient>)> {
        self.adaptor_session
            .read()
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    /// The session items to apply as modifiers (`myAdaptorApplied`).
    pub(super) fn adaptor_applied(&self) -> Vec<Handle<StandardTransient>> {
        self.adaptor_applied.read().clone()
    }
    /// `Recorded(name)`: the controller recorded under `name` (null when none).
    pub fn recorded(name: &str) -> Handle<dyn XSControlController> {
        recorded_controllers()
            .seek(&AsciiString::new(name))
            .cloned()
            .unwrap_or_default()
    }
}

/// The text of an `AsciiString`.
fn text(s: &AsciiString) -> String {
    String::from_utf8_lossy(s.as_bytes()).into_owned()
}

/// `TransferFinder(theActor, theMapper, theFP, theModel, theModeTrans, theProgress)`: the
/// mapper transferred by the actor in the finder process; each transient result added to
/// the model with what it references. Done when a result is added, fail otherwise; error
/// without actor or model.
fn transfer_finder(
    actor: &Handle<dyn TransferActorOfFinderProcessOps>,
    mapper: &Handle<dyn TransferFinder>,
    fp: &Handle<TransferFinderProcess>,
    model: &Handle<dyn InterfaceInterfaceModel>,
    modetrans: i32,
    progress: &ProgressRange<'_>,
) -> Result<IFSelectReturnStatus, Failure> {
    let (Some(live_actor), Some(live_model)) = (actor.get(), model.get()) else {
        return Ok(IFSelectReturnStatus::RetError);
    };
    *live_actor.mode_trans() = modetrans;
    let process = fp
        .get()
        .ok_or_else(|| exceptions::null_object("XSControl_Controller: null finder process"))?;
    process.set_model(model);
    process.set_actor(&actor.cast());
    process.transfer(mapper, progress)?;
    let mut stat = IFSelectReturnStatus::RetFail;
    let mut binder = process.find(mapper);
    while let Some(live) = binder.get() {
        if let Some(bindtr) = binder.cast::<TransferSimpleBinderOfTransient>().get() {
            let ent = bindtr.result();
            if ent.is_some() {
                stat = IFSelectReturnStatus::RetDone;
                live_model.add_with_refs(&ent, 0, false)?;
            }
        }
        let next = live.next_result();
        binder = next;
    }
    Ok(stat)
}

/// `XSControl_Controller`, abstract: `NewModel` is required; the protected constructor
/// is [`XSControlControllerBase::new`], whose `bind_self` an implementation calls once it
/// has its handle. The non-virtual members are default methods (do not override them).
pub trait XSControlController: Transient {
    /// The embedded fields.
    fn controller_base(&self) -> &XSControlControllerBase;
    /// `NewModel`: a new empty model of the norm.
    fn new_model(&self) -> Handle<dyn InterfaceInterfaceModel>;
    /// `ActorRead(model)`: the reading actor (`myAdaptorRead` by default).
    fn actor_read(
        &self,
        _model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Handle<dyn TransferActorOfTransientProcessOps> {
        self.controller_base().adaptor_read()
    }
    /// `ActorWrite`: the writing actor (`myAdaptorWrite` by default).
    fn actor_write(&self) -> Handle<dyn TransferActorOfFinderProcessOps> {
        self.controller_base().adaptor_write()
    }
    /// `RecognizeWriteTransient(obj, modetrans = 0)`: the writing actor recognizes the
    /// transient (false without actor).
    fn recognize_write_transient(&self, obj: &Handle<StandardTransient>, modetrans: i32) -> bool {
        let actor = self.controller_base().adaptor_write();
        let Some(live) = actor.get() else {
            return false;
        };
        *live.mode_trans() = modetrans;
        live.recognize(&Handle::new(TransferTransientMapper::new(obj)).cast())
    }
    /// `TransferWriteTransient(obj, FP, model, modetrans = 0, theProgress)`: the transient
    /// transferred by the writing actor (void for null).
    fn transfer_write_transient(
        &self,
        obj: &Handle<StandardTransient>,
        fp: &Handle<TransferFinderProcess>,
        model: &Handle<dyn InterfaceInterfaceModel>,
        modetrans: i32,
        progress: &ProgressRange<'_>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if obj.is_null() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        let mapper: Handle<dyn TransferFinder> =
            Handle::new(TransferTransientMapper::new(obj)).cast();
        transfer_finder(
            &self.controller_base().adaptor_write(),
            &mapper,
            fp,
            model,
            modetrans,
            progress,
        )
    }
    /// `RecognizeWriteShape(shape, modetrans = 0)`: the writing actor recognizes the
    /// shape (false without actor).
    fn recognize_write_shape(&self, shape: &Shape, modetrans: i32) -> bool {
        let actor = self.controller_base().adaptor_write();
        let Some(live) = actor.get() else {
            return false;
        };
        *live.mode_trans() = modetrans;
        live.recognize(&Handle::new(TransferBRepShapeMapper::new(shape)).cast())
    }
    /// `TransferWriteShape(shape, FP, model, modetrans = 0, theProgress)`: the shape
    /// transferred by the writing actor (void for a null shape).
    fn transfer_write_shape(
        &self,
        shape: &Shape,
        fp: &Handle<TransferFinderProcess>,
        model: &Handle<dyn InterfaceInterfaceModel>,
        modetrans: i32,
        progress: &ProgressRange<'_>,
    ) -> Result<IFSelectReturnStatus, Failure> {
        if shape.is_null() {
            return Ok(IFSelectReturnStatus::RetVoid);
        }
        let mapper: Handle<dyn TransferFinder> =
            Handle::new(TransferBRepShapeMapper::new(shape)).cast();
        transfer_finder(
            &self.controller_base().adaptor_write(),
            &mapper,
            fp,
            model,
            modetrans,
            progress,
        )
    }
    /// `Customise(WS)`: the session prepared for the norm (see
    /// [`super::controller_customise`]).
    fn customise(&self, ws: &Handle<XSControlWorkSession>) -> Result<(), Failure> {
        super::controller_customise::customise_base(self.controller_base(), ws)
    }
    /// `SetNames(theLongName, theShortName)`: the names given (the empty ones kept).
    fn set_names(&self, long_name: &str, short_name: &str) {
        let mut names = self.controller_base().names.write();
        if !long_name.is_empty() {
            names.1 = AsciiString::new(long_name);
        }
        if !short_name.is_empty() {
            names.0 = AsciiString::new(short_name);
        }
    }
    /// `AutoRecord`: recorded under its short and long names.
    fn auto_record(&self) -> Result<(), Failure> {
        self.record(&text(&self.name(true)))?;
        self.record(&text(&self.name(false)))
    }
    /// `Record(name)`: recorded under `name`, unless a controller of a derived type is;
    /// `Standard_DomainError` when another unrelated controller is.
    fn record(&self, name: &str) -> Result<(), Failure> {
        let this = self.controller_base().this()?;
        let key = AsciiString::new(name);
        let mut listad = recorded_controllers();
        if let Some(newadapt) = listad.seek(&key).cloned() {
            let this_type = this
                .dynamic_type()
                .ok_or_else(|| exceptions::null_object("XSControl_Controller::Record: no type"))?;
            if newadapt.is_kind(this_type) {
                return Ok(());
            }
            let new_type = newadapt
                .dynamic_type()
                .ok_or_else(|| exceptions::null_object("XSControl_Controller::Record: no type"))?;
            if !this.is_kind(new_type) && this != newadapt {
                return Err(exceptions::domain_error(&format!(
                    "XSControl_Controller : Record: {name} is recorded by another controller"
                )));
            }
        }
        listad.bind(key, this);
        Ok(())
    }
    /// `Name(rsc = false)`: the short (`rsc`) or long name.
    fn name(&self, rsc: bool) -> AsciiString {
        let names = self.controller_base().names.read();
        if rsc {
            names.0.clone()
        } else {
            names.1.clone()
        }
    }
    /// `Protocol`.
    fn protocol(&self) -> Handle<dyn InterfaceProtocol> {
        self.controller_base().adaptor_protocol.read().clone()
    }
    /// `WorkLibrary`.
    fn work_library(&self) -> Handle<dyn IFSelectWorkLibrary> {
        self.controller_base().adaptor_library.read().clone()
    }
    /// `SetModeWrite(modemin, modemax, shape = true)`: the modes of the shape writing (none
    /// for `modemin > modemax`).
    fn set_mode_write(&self, modemin: i32, modemax: i32, _shape: bool) {
        *self.controller_base().mode_write_shape_n.write() = (modemin <= modemax).then(|| {
            (
                modemin,
                vec![Handle::default(); (modemax - modemin + 1) as usize],
            )
        });
    }
    /// `SetModeWriteHelp(modetrans, help, shape = true)`: the help of a mode (nothing out
    /// of the modes).
    fn set_mode_write_help(&self, modetrans: i32, help: &str, _shape: bool) {
        let mut modes = self.controller_base().mode_write_shape_n.write();
        if let Some((lower, helps)) = modes.as_mut()
            && let Some(slot) = usize::try_from(modetrans - *lower)
                .ok()
                .and_then(|i| helps.get_mut(i))
        {
            *slot = Handle::new(HAsciiString::new(help));
        }
    }
    /// `ModeWriteBounds(modemin, modemax, shape = true)`: the bounds, `None` (natively
    /// false, the bounds 0) without modes.
    fn mode_write_bounds(&self, _shape: bool) -> Option<(i32, i32)> {
        self.controller_base()
            .mode_write_shape_n
            .read()
            .as_ref()
            .map(|(lower, helps)| (*lower, lower + helps.len() as i32 - 1))
    }
    /// `IsModeWrite(modetrans, shape = true)`: true without modes, else within them.
    fn is_mode_write(&self, modetrans: i32, shape: bool) -> bool {
        self.mode_write_bounds(shape)
            .is_none_or(|(min, max)| (min..=max).contains(&modetrans))
    }
    /// `ModeWriteHelp(modetrans, shape = true)`: the help of a mode (empty out of them).
    fn mode_write_help(&self, modetrans: i32, _shape: bool) -> AsciiString {
        let modes = self.controller_base().mode_write_shape_n.read();
        let Some((lower, helps)) = modes.as_ref() else {
            return AsciiString::default();
        };
        let help = usize::try_from(modetrans - lower)
            .ok()
            .and_then(|i| helps.get(i))
            .cloned()
            .unwrap_or_default();
        help.get().map(|h| h.string().clone()).unwrap_or_default()
    }
    /// `AddSessionItem(theItem, theName, toApply = false)`: an item for the sessions (a
    /// modifier applied when `toApply`); nothing for a null item or an empty name.
    fn add_session_item(&self, item: &Handle<StandardTransient>, name: &str, to_apply: bool) {
        if item.is_null() || name.is_empty() {
            return;
        }
        let base = self.controller_base();
        base.adaptor_session
            .write()
            .bind(AsciiString::new(name), item.clone());
        if to_apply && item.is_kind(<dyn IFSelectGeneralModifier>::static_type()) {
            base.adaptor_applied.write().push(item.clone());
        }
    }
    /// `SessionItem(theName)`: null without items; `Standard_NoSuchObject` for a name
    /// unknown among the items (the native `Find`).
    fn session_item(&self, name: &str) -> Result<Handle<StandardTransient>, Failure> {
        let session = self.controller_base().adaptor_session.read();
        if session.is_empty() {
            return Ok(Handle::default());
        }
        session.find(&AsciiString::new(name)).cloned()
    }
    /// `AdaptorSession`: the items for the sessions.
    fn adaptor_session(&self) -> DataMap<AsciiString, Handle<StandardTransient>> {
        let mut copy = DataMap::default();
        for (k, v) in self.controller_base().session_items() {
            copy.bind(k, v);
        }
        copy
    }
}
impl_transient_interface!(
    dyn XSControlController,
    "XSControl_Controller",
    StandardTransient
);
