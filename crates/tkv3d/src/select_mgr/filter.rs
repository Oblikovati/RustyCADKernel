//! `SelectMgr_Filter` and the composition filters (`SelectMgr_CompositionFilter`,
//! `SelectMgr_AndFilter`, `SelectMgr_OrFilter`, `SelectMgr_AndOrFilter`).
use super::{SelectMgrEntityOwnerSource, SelectMgrFilterType};
use rustycad_tkernel::ncollection::Shared;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, Transient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::collections::BTreeSet;

/// A filter of the detected owners.
pub trait SelectMgrFilter: Transient {
    /// `IsOk` (pure virtual): whether the owner may be detected; `filter.is_ok(&owner)`.
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> bool;
    /// `ActsOn`: false by default; `filter.acts_on(ShapeEnum::Face)`.
    fn acts_on(&self, standard_mode: ShapeEnum) -> bool {
        let _ = standard_mode;
        false
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn SelectMgrFilter,
    "SelectMgr_Filter",
    StandardTransient
);

/// The filters of a composition filter (`myFilters`).
#[derive(Default)]
pub struct SelectMgrCompositionFilterBase {
    /// `myFilters`.
    pub filters: SharedState<Vec<Handle<dyn SelectMgrFilter>>>,
}

/// A filter combining other filters.
pub trait SelectMgrCompositionFilter: SelectMgrFilter {
    /// The embedded filter list; `filter.composition_base()`.
    fn composition_base(&self) -> &SelectMgrCompositionFilterBase;
    /// `Add`; `composition.add(&filter)`.
    fn add(&self, filter: &Handle<dyn SelectMgrFilter>) {
        self.composition_base().filters.write().push(filter.clone());
    }
    /// `Remove`: the first occurrence of the filter; `composition.remove(&filter)`.
    fn remove(&self, filter: &Handle<dyn SelectMgrFilter>) {
        let mut filters = self.composition_base().filters.write();
        if let Some(index) = filters.iter().position(|f| f == filter) {
            filters.remove(index);
        }
    }
    /// `IsEmpty`; `composition.is_empty()`.
    fn is_empty(&self) -> bool {
        self.composition_base().filters.read().is_empty()
    }
    /// `IsIn`; `composition.is_in(&filter)`.
    fn is_in(&self, filter: &Handle<dyn SelectMgrFilter>) -> bool {
        self.composition_base()
            .filters
            .read()
            .iter()
            .any(|f| f == filter)
    }
    /// `StoredFilters`; `composition.stored_filters()`.
    fn stored_filters(&self) -> Vec<Handle<dyn SelectMgrFilter>> {
        self.composition_base().filters.read().clone()
    }
    /// `Clear`; `composition.clear()`.
    fn clear(&self) {
        self.composition_base().filters.write().clear();
    }
}

/// `SelectMgr_CompositionFilter::ActsOn`: one stored filter acts on the mode.
/// Example: `select_mgr_composition_acts_on(&*filter, ShapeEnum::Edge)`.
pub fn select_mgr_composition_acts_on<F: SelectMgrCompositionFilter + ?Sized>(
    filter: &F,
    standard_mode: ShapeEnum,
) -> bool {
    filter
        .stored_filters()
        .iter()
        .filter_map(Handle::get)
        .any(|f| f.acts_on(standard_mode))
}

macro_rules! composition_filter {
    ($name:ident, $native:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Default)]
        pub struct $name {
            transient: StandardTransient,
            base: SelectMgrCompositionFilterBase,
        }
        rustycad_tkernel::impl_transient!($name, $native, StandardTransient, transient,
            interfaces: [dyn SelectMgrFilter]);
        impl SelectMgrCompositionFilter for $name {
            fn composition_base(&self) -> &SelectMgrCompositionFilterBase {
                &self.base
            }
        }
    };
}

composition_filter!(
    SelectMgrAndFilter,
    "SelectMgr_AndFilter",
    "`SelectMgr_AndFilter`: every stored filter must accept; `SelectMgrAndFilter::default()`."
);
composition_filter!(
    SelectMgrOrFilter,
    "SelectMgr_OrFilter",
    "`SelectMgr_OrFilter`: one stored filter (or none stored) must accept; `SelectMgrOrFilter::default()`."
);

impl SelectMgrFilter for SelectMgrAndFilter {
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> bool {
        self.stored_filters()
            .iter()
            .filter_map(Handle::get)
            .all(|f| f.is_ok(owner))
    }
    fn acts_on(&self, standard_mode: ShapeEnum) -> bool {
        select_mgr_composition_acts_on(self, standard_mode)
    }
}

impl SelectMgrFilter for SelectMgrOrFilter {
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> bool {
        let filters = self.stored_filters();
        filters.is_empty()
            || filters
                .iter()
                .filter_map(Handle::get)
                .any(|f| f.is_ok(owner))
    }
    fn acts_on(&self, standard_mode: ShapeEnum) -> bool {
        select_mgr_composition_acts_on(self, standard_mode)
    }
}

/// The disabled objects of an `SelectMgr_AndOrFilter`, by object address
/// (`NCollection_Shared<NCollection_Map<const Standard_Transient*>>`).
pub type SelectMgrDisabledObjects = Shared<SharedState<BTreeSet<usize>>>;

/// `SelectMgr_AndOrFilter`: the stored filters combined by AND or OR, refusing the disabled
/// objects. Example: `SelectMgrAndOrFilter::new(SelectMgrFilterType::Or)`.
pub struct SelectMgrAndOrFilter {
    transient: StandardTransient,
    base: SelectMgrCompositionFilterBase,
    disabled_objects: SharedState<Handle<SelectMgrDisabledObjects>>,
    filter_type: SharedState<SelectMgrFilterType>,
}
rustycad_tkernel::impl_transient!(SelectMgrAndOrFilter, "SelectMgr_AndOrFilter", StandardTransient, transient,
    interfaces: [dyn SelectMgrFilter]);

impl SelectMgrAndOrFilter {
    /// A filter of a type without disabled objects; `SelectMgrAndOrFilter::new(SelectMgrFilterType::And)`.
    pub fn new(filter_type: SelectMgrFilterType) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: SelectMgrCompositionFilterBase::default(),
            disabled_objects: SharedState::new(Handle::default()),
            filter_type: SharedState::new(filter_type),
        }
    }
    /// `SetDisabledObjects`; `filter.set_disabled_objects(&objects)`.
    pub fn set_disabled_objects(&self, objects: &Handle<SelectMgrDisabledObjects>) {
        *self.disabled_objects.write() = objects.clone();
    }
    /// `FilterType`; `filter.filter_type()`.
    pub fn filter_type(&self) -> SelectMgrFilterType {
        *self.filter_type.read()
    }
    /// `SetFilterType`; `filter.set_filter_type(SelectMgrFilterType::Or)`.
    pub fn set_filter_type(&self, filter_type: SelectMgrFilterType) {
        *self.filter_type.write() = filter_type;
    }
    /// Whether the selectable object of the owner is disabled.
    fn is_disabled(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> bool {
        let address = owner
            .get()
            .map_or(std::ptr::null(), |o| o.selectable().object_address())
            as usize;
        self.disabled_objects
            .read()
            .get()
            .is_some_and(|set| set.read().contains(&address))
    }
}

impl SelectMgrCompositionFilter for SelectMgrAndOrFilter {
    fn composition_base(&self) -> &SelectMgrCompositionFilterBase {
        &self.base
    }
}

impl SelectMgrFilter for SelectMgrAndOrFilter {
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> bool {
        if self.is_disabled(owner) {
            return false;
        }
        let filter_type = self.filter_type();
        let filters = self.stored_filters();
        for filter in filters.iter().filter_map(Handle::get) {
            let is_ok = filter.is_ok(owner);
            if is_ok && filter_type == SelectMgrFilterType::Or {
                return true;
            }
            if !is_ok && filter_type == SelectMgrFilterType::And {
                return false;
            }
        }
        filter_type != SelectMgrFilterType::Or || filters.is_empty()
    }
    fn acts_on(&self, standard_mode: ShapeEnum) -> bool {
        select_mgr_composition_acts_on(self, standard_mode)
    }
}
