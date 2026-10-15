// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Assistant.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_Assistant`: the process-wide data used while models are saved or loaded (the
//! native function-local statics; tobj-design.md section 5).
use super::{TObjModel, TObjModelState};
use rustycad_tkernel::{
    standard::{Failure, Handle, RuntimeType},
    tcollection::ExtendedString,
};
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

/// One key of the type map: a type, or a `TObj_Assistant_UnknownType` (a fresh object
/// for each null type, so never equal to another key).
#[derive(Clone, Copy)]
enum TypeKey {
    Known(&'static RuntimeType),
    Unknown,
}

/// The statics `getModels`, `getTypes`, `getCurrentModel` and `getVersion`.
struct AssistantState {
    models: Vec<Handle<dyn TObjModel>>,
    types: Vec<TypeKey>,
    current_model: Handle<dyn TObjModel>,
    version: i32,
}

static STATE: LazyLock<Mutex<AssistantState>> = LazyLock::new(|| {
    Mutex::new(AssistantState {
        models: Vec::new(),
        types: Vec::new(),
        current_model: Handle::default(),
        version: 0,
    })
});

fn state() -> MutexGuard<'static, AssistantState> {
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// `TObj_Assistant`: a class of static members.
pub struct TObjAssistant;

impl TObjAssistant {
    /// `FindModel(theName)`: the last bound model of that name (UTF-8), null when none.
    pub fn find_model(name: &str) -> Result<Handle<dyn TObjModel>, Failure> {
        let wanted = ExtendedString::from_c_string(name.as_bytes(), true);
        let models = state().models.clone();
        for model in models.iter().rev() {
            let live = rustycad_tkcdf::live(model, "TObj_Assistant::FindModel")?;
            if wanted.is_equal(&*live.get_model_name().string()) {
                return Ok(model.clone());
            }
        }
        Ok(Handle::default())
    }
    /// `BindModel(theModel)`.
    pub fn bind_model(model: &Handle<dyn TObjModel>) {
        state().models.push(model.clone());
    }
    /// `ClearModelMap`.
    pub fn clear_model_map() {
        state().models.clear();
    }
    /// `FindType(theTypeIndex)`: the type of a 1-based index, `None` when out of range or
    /// bound for a null type.
    pub fn find_type(index: i32) -> Option<&'static RuntimeType> {
        let types = &state().types;
        let position = usize::try_from(index).ok()?.checked_sub(1)?;
        match types.get(position)? {
            TypeKey::Known(kind) => Some(*kind),
            TypeKey::Unknown => None,
        }
    }
    /// `FindTypeIndex(theType)`: the 1-based index of the type, 0 when not bound.
    pub fn find_type_index(kind: Option<&'static RuntimeType>) -> i32 {
        let Some(kind) = kind else {
            return 0;
        };
        state()
            .types
            .iter()
            .position(|key| matches!(key, TypeKey::Known(known) if std::ptr::eq(*known, kind)))
            .map_or(0, |position| position as i32 + 1)
    }
    /// `BindType(theType)`: the index of the type, added when absent; a null type always
    /// takes a new index.
    pub fn bind_type(kind: Option<&'static RuntimeType>) -> i32 {
        let existing = Self::find_type_index(kind);
        if existing != 0 {
            return existing;
        }
        let mut state = state();
        state
            .types
            .push(kind.map_or(TypeKey::Unknown, TypeKey::Known));
        state.types.len() as i32
    }
    /// `ClearTypeMap`.
    pub fn clear_type_map() {
        state().types.clear();
    }
    /// `SetCurrentModel(theModel)`: the version is reset to unknown (-1).
    pub fn set_current_model(model: &Handle<dyn TObjModel>) {
        let mut state = state();
        state.current_model = model.clone();
        state.version = -1;
    }
    /// `GetCurrentModel`.
    pub fn get_current_model() -> Handle<dyn TObjModel> {
        state().current_model.clone()
    }
    /// `UnSetCurrentModel`.
    pub fn unset_current_model() {
        state().current_model = Handle::default();
    }
    /// `GetAppVersion`: the format version of the current model, read once per model;
    /// 0 before any model was set.
    pub fn get_app_version() -> Result<i32, Failure> {
        let (version, model) = {
            let state = state();
            (state.version, state.current_model.clone())
        };
        if version >= 0 {
            return Ok(version);
        }
        let Some(model) = model.get() else {
            return Ok(version);
        };
        let version = model.get_format_version()?;
        state().version = version;
        Ok(version)
    }
}
