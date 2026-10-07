// Rust port of DataExchange/TKXSBase/Interface/Interface_GTool (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_GTool`: the general library of a protocol, with the signature of entity
//! types and a cache of the selected module and case number by entity type.
use super::{
    InterfaceGeneralLib, InterfaceGeneralModule, InterfaceInterfaceModel, InterfaceProtocol,
    InterfaceSignType, InterfaceSignTypeBase,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{DataMap, IndexedDataMap},
    standard::{Handle, RuntimeType, SharedState, StandardTransient},
};

/// The cache key of an entity type: the address of its descriptor (natively the
/// `occ::handle<Standard_Type>`, hashed by address).
type TypeKey = usize;

fn type_key(kind: &RuntimeType) -> TypeKey {
    kind as *const RuntimeType as usize
}

/// The fields of a GTool.
#[derive(Default)]
struct GToolState {
    theproto: Handle<dyn InterfaceProtocol>,
    thesign: Handle<dyn InterfaceSignType>,
    thelib: InterfaceGeneralLib,
    thentnum: DataMap<TypeKey, i32>,
    thentmod: IndexedDataMap<TypeKey, Handle<dyn InterfaceGeneralModule>>,
}

/// `Interface_GTool`.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceGTool;
/// let gtool = InterfaceGTool::new();
/// assert_eq!(gtool.sign_name(), "Class Name");
/// ```
#[derive(Default)]
pub struct InterfaceGTool {
    transient: StandardTransient,
    state: SharedState<GToolState>,
}
impl_transient!(
    InterfaceGTool,
    "Interface_GTool",
    StandardTransient,
    transient
);

impl InterfaceGTool {
    /// `Interface_GTool()`: no protocol.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Interface_GTool(proto, nbent = 0)`: the library of `proto`, with room for `nbent`
    /// entity types.
    pub fn with_protocol(proto: &Handle<dyn InterfaceProtocol>, nbent: i32) -> Self {
        let gtool = Self::default();
        {
            let mut state = gtool.state.write();
            state.theproto = proto.clone();
            state.thelib = InterfaceGeneralLib::new(proto);
            if nbent > 0 {
                state.thentnum.resize(nbent as usize);
                state.thentmod.resize(nbent as usize);
            }
        }
        gtool
    }
    /// `SetSignType(sign)`.
    pub fn set_sign_type(&self, sign: &Handle<dyn InterfaceSignType>) {
        self.state.write().thesign = sign.clone();
    }
    /// `SignType`.
    pub fn sign_type(&self) -> Handle<dyn InterfaceSignType> {
        self.state.read().thesign.clone()
    }
    /// `SignValue(ent, model)`: the signature of `ent`, by default its class name.
    pub fn sign_value(
        &self,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> String {
        let Some(kind) = ent.dynamic_type() else {
            return String::new();
        };
        let sign = self.sign_type();
        match sign.get() {
            None => InterfaceSignTypeBase::class_name(kind.name()).to_owned(),
            Some(signature) => signature.value(ent, model),
        }
    }
    /// `SignName`: "Class Name" without signature.
    pub fn sign_name(&self) -> String {
        let sign = self.sign_type();
        sign.get()
            .map_or_else(|| String::from("Class Name"), |s| s.name())
    }
    /// `SetProtocol(proto, enforce = false)`: rebuilds the library (unless the protocol
    /// is the same and not enforced).
    pub fn set_protocol(&self, proto: &Handle<dyn InterfaceProtocol>, enforce: bool) {
        let mut state = self.state.write();
        if state.theproto == *proto && !enforce {
            return;
        }
        state.theproto = proto.clone();
        state.thelib.clear();
        state.thelib.add_protocol(&proto.cast());
    }
    /// `Protocol`.
    pub fn protocol(&self) -> Handle<dyn InterfaceProtocol> {
        self.state.read().theproto.clone()
    }
    /// `Lib`: a copy of the library (it shares the selected nodes).
    pub fn lib(&self) -> InterfaceGeneralLib {
        self.state.read().thelib.clone()
    }
    /// `Lib` as a modifiable reference: edits the library under the lock.
    pub fn change_lib<R>(&self, edit: impl FnOnce(&mut InterfaceGeneralLib) -> R) -> R {
        edit(&mut self.state.write().thelib)
    }
    /// `Reservate(nb, enforce = false)`. Native quirk kept: the cache grows only when it
    /// already has at least `nb` buckets, or when enforced.
    pub fn reservate(&self, nb: i32, enforce: bool) {
        let mut state = self.state.write();
        if (state.thentnum.bucket_count() as i64) < i64::from(nb) && !enforce {
            return;
        }
        let size = usize::try_from(nb).unwrap_or(0);
        state.thentnum.resize(size);
        state.thentmod.resize(size);
    }
    /// `ClearEntities`: empties the cache.
    pub fn clear_entities(&self) {
        let mut state = self.state.write();
        state.thentnum.clear(false);
        state.thentmod.clear(false);
    }
    /// `Select(ent, gmod, CN, enforce = false)`: the module and case number of the type
    /// of `ent`, from the cache or (when not cached, or enforced) from the library.
    pub fn select(
        &self,
        ent: &Handle<StandardTransient>,
        enforce: bool,
    ) -> Option<(Handle<dyn InterfaceGeneralModule>, i32)> {
        let key = type_key(ent.dynamic_type()?);
        let lib = {
            let state = self.state.read();
            if state.thentmod.find_index(&key) != 0 && !enforce {
                let module = state.thentmod.find_from_key(&key).ok()?.clone();
                let cn = *state.thentnum.find(&key).ok()?;
                return Some((module, cn));
            }
            state.thelib.clone()
        };
        let (module, cn) = lib.select(ent)?;
        let mut state = self.state.write();
        state.thentmod.add(key, module.clone());
        state.thentnum.bind(key, cn);
        Some((module, cn))
    }
}
