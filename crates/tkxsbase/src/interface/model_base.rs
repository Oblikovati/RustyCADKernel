// Rust port of DataExchange/TKXSBase/Interface/Interface_InterfaceModel.cxx (the private
// fields, the constructor, Destroy and the template statics)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The embedded base of every interface model (xsbase-design.md section 1): the indexed
//! map of entities, the report maps, the global checks and the GTool.
use super::{InterfaceCheck, InterfaceGTool, InterfaceInterfaceModel};
use rustycad_tkernel::{
    ncollection::{DataMap, IndexedMap},
    standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle, exceptions},
    tcollection::{AsciiString, HAsciiString},
};
use std::sync::{OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The private fields of `Interface_InterfaceModel`, in the native order.
pub(super) struct ModelState {
    pub(super) theentities: IndexedMap<Handle<StandardTransient>>,
    pub(super) thereports: DataMap<i32, Handle<StandardTransient>>,
    pub(super) therepch: DataMap<i32, Handle<StandardTransient>>,
    pub(super) thecheckstx: Handle<InterfaceCheck>,
    pub(super) thechecksem: Handle<InterfaceCheck>,
    pub(super) haschecksem: bool,
    pub(super) isdispatch: bool,
    pub(super) thecategory: Handle<HAsciiString>,
    pub(super) thegtool: Handle<InterfaceGTool>,
}

/// The embedded base of an interface model; `InterfaceModelBase::default()` is the native
/// protected constructor (no entity, two empty global checks).
pub struct InterfaceModelBase {
    pub(super) state: SharedState<ModelState>,
    this: OnceLock<WeakHandle<dyn InterfaceInterfaceModel>>,
}

impl Default for InterfaceModelBase {
    fn default() -> Self {
        Self {
            state: SharedState::new(ModelState {
                theentities: IndexedMap::default(),
                thereports: DataMap::default(),
                therepch: DataMap::default(),
                thecheckstx: Handle::new(InterfaceCheck::new()),
                thechecksem: Handle::new(InterfaceCheck::new()),
                haschecksem: false,
                isdispatch: false,
                thecategory: Handle::default(),
                thegtool: Handle::default(),
            }),
            this: OnceLock::new(),
        }
    }
}

/// The model templates (`atemp`), by name.
type TemplateMap = DataMap<AsciiString, Handle<dyn InterfaceInterfaceModel>>;

fn templates() -> &'static RwLock<TemplateMap> {
    static TEMPLATES: OnceLock<RwLock<TemplateMap>> = OnceLock::new();
    TEMPLATES.get_or_init(|| RwLock::new(TemplateMap::default()))
}

impl InterfaceModelBase {
    /// Records the owning handle of the model (natively `this` converts to it); the
    /// members that pass the model to other objects need it.
    /// `model.model_base().bind_self(&model)`.
    pub fn bind_self(&self, model: &Handle<dyn InterfaceInterfaceModel>) {
        let _ = self.this.set(model.downgrade());
    }
    /// The handle `this` converts to natively; null before `bind_self`.
    pub fn this(&self) -> Handle<dyn InterfaceInterfaceModel> {
        self.this.get().map(WeakHandle::upgrade).unwrap_or_default()
    }
    pub(super) fn read(&self) -> RwLockReadGuard<'_, ModelState> {
        self.state.read()
    }
    pub(super) fn write(&self) -> RwLockWriteGuard<'_, ModelState> {
        self.state.write()
    }
    /// The base `ClearEntities` (also the one the native destructor runs).
    pub(super) fn clear_entities_base(&self) {
        let gtool = {
            let mut state = self.write();
            state.thereports.clear(false);
            state.therepch.clear(false);
            state.haschecksem = false;
            state.isdispatch = false;
            state.theentities.clear(false);
            state.thegtool.clone()
        };
        // smh#14 FRA62479: the cache of the GTool is cleared with the entities.
        if let Some(gtool) = gtool.get() {
            gtool.clear_entities();
        }
    }
    /// `HasTemplate(name)`.
    pub fn has_template(name: &str) -> bool {
        Self::template_map().is_bound(&AsciiString::new(name))
    }
    /// `Template(name)`: a new empty model filled from the template (null when none).
    pub fn template(name: &str) -> Result<Handle<dyn InterfaceInterfaceModel>, Failure> {
        let found = Self::template_map().seek(&AsciiString::new(name)).cloned();
        let Some(model) = found.filter(|m| m.is_some()) else {
            return Ok(Handle::default());
        };
        let newmod = model.new_empty_model();
        let live = newmod.get().ok_or_else(|| {
            exceptions::null_object(
                "Interface_InterfaceModel::Template: NewEmptyModel returned null; expected a model",
            )
        })?;
        live.get_from_another(&model)?;
        Ok(newmod)
    }
    /// `SetTemplate(name, model)`: true when the name is new.
    pub fn set_template(name: &str, model: &Handle<dyn InterfaceInterfaceModel>) -> bool {
        let mut map = templates().write().unwrap_or_else(PoisonError::into_inner);
        map.bind(AsciiString::new(name), model.clone())
    }
    /// `ListTemplates`: the names of the templates, in the native map order.
    pub fn list_templates() -> Vec<Handle<HAsciiString>> {
        let map = Self::template_map();
        map.items()
            .map(|(name, _)| Handle::new(HAsciiString::from_string(name)))
            .collect()
    }
    fn template_map() -> RwLockReadGuard<'static, TemplateMap> {
        templates().read().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Drop for InterfaceModelBase {
    /// `~Interface_InterfaceModel`: `Destroy` with the base `ClearEntities`, as in a native
    /// destructor (virtual calls resolve to the base class there).
    fn drop(&mut self) {
        self.clear_entities_base();
        let state = self.read();
        for check in [&state.thecheckstx, &state.thechecksem] {
            if let Some(live) = check.get() {
                live.clear();
            }
        }
    }
}
