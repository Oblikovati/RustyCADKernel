//! Selected-owner state, live cursors, virtual dispatch and callback boundaries.
#[path = "support/ais_selection_fixture.rs"]
mod ais_selection_fixture;
#[path = "support/boxed_selectable.rs"]
#[allow(dead_code)]
mod boxed_selectable;
use ais_selection_fixture::{AisStateFilter, AisStateOwner};
use boxed_selectable::BoxedSelectable;
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkv3d::{ais::*, select_mgr::*};
use std::sync::atomic::Ordering;
struct AisScene {
    selection: Handle<AisSelection>,
    object: Handle<dyn SelectMgrSelectableObject>,
    owners: Vec<Handle<AisStateOwner>>,
    filter: Handle<AisStateFilter>,
}
impl AisScene {
    fn new() -> Self {
        let selection = Handle::new(AisSelection::new());
        let object = BoxedSelectable::new(0., false).cast();
        let owners: Vec<_> = (0..4).map(|i| AisStateOwner::new(&object, i)).collect();
        for owner in &owners {
            *owner.selection.write() = selection.cast::<dyn AisSelectionSource>().downgrade();
        }
        let filter = AisStateFilter::new(&selection.cast(), &owners[3].cast());
        Self {
            selection,
            object,
            owners,
            filter,
        }
    }
    fn owner(&self, index: usize) -> AisSelectedOwner {
        self.owners[index].cast()
    }
}
#[test]
fn add_selection_keeps_list_order_and_actual_borrowed_handles() {
    let scene = AisScene::new();
    assert!(scene.selection.is_empty());
    for i in 0..3 {
        assert_eq!(
            scene.selection.add_select(&scene.owner(i)).unwrap(),
            AisSelectStatus::Added
        );
    }
    assert_eq!(
        scene.selection.add_select(&scene.owner(0)).unwrap(),
        AisSelectStatus::NotDone
    );
    scene.selection.init();
    let current = scene.selection.value().unwrap();
    let objects = scene.selection.objects();
    assert!(std::ptr::eq(&*current, objects.first().unwrap()));
    assert_eq!(
        objects.iter().map(|o| o.priority()).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!(
        (
            AisSelectStatus::Added as i32,
            AisSelectStatus::Removed as i32,
            AisSelectStatus::NotDone as i32
        ),
        (0, 1, 2)
    );
}
#[test]
fn remove_repairs_successor_positions_and_advances_public_iteration() {
    let scene = AisScene::new();
    for i in 0..3 {
        scene.selection.add_select(&scene.owner(i)).unwrap();
    }
    scene.selection.init();
    for i in 0..3 {
        assert_eq!(
            scene
                .selection
                .select(
                    &scene.owner(i),
                    &Handle::default(),
                    AisSelectionScheme::Add,
                    true
                )
                .unwrap(),
            AisSelectStatus::Removed
        );
        assert!(!scene.owners[i].is_selected());
        assert!(!scene.selection.is_selected(&scene.owner(i)));
    }
    assert!(!scene.selection.more());
    assert_eq!(scene.selection.extent(), 0);
    assert_eq!(
        scene.selection.value().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
    scene.selection.next();
    assert!(!scene.selection.more());
}
#[test]
fn forced_highlighting_observes_an_already_advanced_cursor() {
    let scene = AisScene::new();
    scene.selection.add_select(&scene.owner(0)).unwrap();
    scene.selection.add_select(&scene.owner(1)).unwrap();
    scene.selection.init();
    scene.owners[0].forced.store(true, Ordering::Release);
    assert_eq!(
        scene
            .selection
            .select(
                &scene.owner(0),
                &Handle::default(),
                AisSelectionScheme::Add,
                true
            )
            .unwrap(),
        AisSelectStatus::Added
    );
    assert_eq!(*scene.owners[0].observed.read(), [1]);
    assert_eq!(scene.selection.extent(), 2);
    assert_eq!(scene.selection.value().unwrap().priority(), 1);
}
#[test]
fn externally_cleared_owner_flag_preserves_native_duplicate_list_entries() {
    let scene = AisScene::new();
    scene.selection.add_select(&scene.owner(0)).unwrap();
    scene.owners[0].set_selected(false);
    scene
        .selection
        .select(
            &scene.owner(0),
            &Handle::default(),
            AisSelectionScheme::Add,
            true,
        )
        .unwrap();
    assert_eq!(scene.selection.extent(), 2);
    assert_eq!(
        scene
            .selection
            .selection_base()
            .fields
            .read()
            .positions
            .len(),
        1
    );
    scene.selection.clear().unwrap();
    assert!(!scene.owners[0].is_selected());
    assert!(scene.selection.is_empty());
}
#[test]
fn replace_extra_compares_count_and_membership_even_for_duplicate_picks() {
    let scene = AisScene::new();
    scene.selection.add_select(&scene.owner(0)).unwrap();
    scene.selection.add_select(&scene.owner(1)).unwrap();
    let owners = [scene.owner(0), scene.owner(0)];
    let picked = Array1::from_shared(&owners, -3);
    scene
        .selection
        .select_owners(
            &picked,
            AisSelectionScheme::ReplaceExtra,
            false,
            &Handle::default(),
        )
        .unwrap();
    assert!(scene.selection.is_empty());
    assert!(!scene.owners[1].is_selected());
}
#[test]
fn failing_and_reentrant_filters_observe_unlocked_pre_mutation_state() {
    let scene = AisScene::new();
    scene.filter.mode.store(3, Ordering::Release);
    assert!(
        scene
            .selection
            .select(
                &scene.owner(0),
                &scene.filter.cast(),
                AisSelectionScheme::Add,
                true
            )
            .is_err()
    );
    assert!(scene.selection.is_empty());
    scene
        .selection
        .select(
            &scene.owner(1),
            &scene.filter.cast(),
            AisSelectionScheme::Add,
            false,
        )
        .unwrap();
    assert_eq!(
        scene.filter.observed.read().len(),
        1,
        "undetected short-circuits filtering"
    );
    scene.filter.mode.store(4, Ordering::Release);
    scene
        .selection
        .select(
            &scene.owner(0),
            &scene.filter.cast(),
            AisSelectionScheme::Add,
            true,
        )
        .unwrap();
    assert_eq!(
        scene
            .selection
            .objects()
            .iter()
            .map(|o| o.priority())
            .collect::<Vec<_>>(),
        [1, 3, 0]
    );
}
#[test]
fn append_owner_validates_owner_before_filter_then_dispatches_add() {
    let scene = AisScene::new();
    assert_eq!(
        scene
            .selection
            .append_owner(&Handle::default(), &Handle::default())
            .unwrap(),
        AisSelectStatus::NotDone
    );
    scene.owners[0].set_selectable(&Handle::default());
    assert_eq!(
        scene
            .selection
            .append_owner(&scene.owner(0), &Handle::default())
            .unwrap(),
        AisSelectStatus::NotDone
    );
    scene.owners[0].set_selectable(&scene.object);
    assert_eq!(
        scene
            .selection
            .append_owner(&scene.owner(0), &Handle::default())
            .unwrap_err()
            .exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        scene
            .selection
            .append_owner(&scene.owner(0), &scene.filter.cast())
            .unwrap(),
        AisSelectStatus::Added
    );
}
struct DispatchSelection {
    transient: StandardTransient,
    base: AisSelectionBase,
    calls: SharedState<Vec<i32>>,
}
rustycad_tkernel::impl_transient!(DispatchSelection,"Test_DispatchSelection",dyn AisSelectionSource,transient,interfaces:[dyn AisSelectionSource]);
impl AisSelectionSource for DispatchSelection {
    fn selection_base(&self) -> &AisSelectionBase {
        &self.base
    }
    fn clear(&self) -> Result<(), Failure> {
        self.calls.write().push(1);
        ais_selection_clear(&self.base)
    }
    fn select(
        &self,
        owner: &AisSelectedOwner,
        filter: &Handle<dyn SelectMgrFilter>,
        scheme: AisSelectionScheme,
        detected: bool,
    ) -> Result<AisSelectStatus, Failure> {
        self.calls.write().push(2);
        ais_selection_select(&self.base, owner, filter, scheme, detected)
    }
    fn add_select(&self, owner: &AisSelectedOwner) -> Result<AisSelectStatus, Failure> {
        self.calls.write().push(3);
        ais_selection_add_select(&self.base, owner)
    }
}
#[test]
fn composite_virtual_members_dispatch_clear_select_and_add_to_subclasses() {
    let scene = AisScene::new();
    let derived = Handle::new(DispatchSelection {
        transient: StandardTransient::default(),
        base: AisSelectionBase::new(),
        calls: SharedState::default(),
    });
    let selection = derived.cast::<dyn AisSelectionSource>();
    selection
        .clear_and_select(&scene.owner(0), &Handle::default(), true)
        .unwrap();
    selection
        .append_owner(&scene.owner(1), &scene.filter.cast())
        .unwrap();
    let owners = [scene.owner(2)];
    let picked = Array1::from_shared(&owners, 1);
    selection
        .select_owners(&picked, AisSelectionScheme::Clear, true, &Handle::default())
        .unwrap();
    assert_eq!(*derived.calls.read(), [1, 2, 3, 1, 2]);
    assert!(selection.is_empty());
}
#[test]
fn composition_filters_preserve_short_circuit_and_failure_order() {
    let scene = AisScene::new();
    let failing = AisStateFilter::new(&scene.selection.cast(), &scene.owner(3));
    failing.mode.store(3, Ordering::Release);
    let and = SelectMgrAndFilter::default();
    let or = SelectMgrOrFilter::default();
    assert!(and.is_ok(&scene.owner(0)).unwrap());
    assert!(or.is_ok(&scene.owner(0)).unwrap());
    and.add(&scene.filter.cast());
    and.add(&failing.cast());
    or.add(&scene.filter.cast());
    or.add(&failing.cast());
    assert!(and.is_ok(&scene.owner(0)).is_err());
    assert!(or.is_ok(&scene.owner(0)).unwrap());
    scene.filter.mode.store(1, Ordering::Release);
    assert!(!and.is_ok(&scene.owner(0)).unwrap());
    assert!(or.is_ok(&scene.owner(0)).is_err());
    assert_eq!(failing.observed.read().len(), 2);
}
#[test]
fn disabled_selectables_skip_failing_filters_and_observe_shared_updates() {
    use rustycad_tkernel::ncollection::Shared;
    use std::collections::BTreeSet;
    let scene = AisScene::new();
    scene.filter.mode.store(3, Ordering::Release);
    let filter = SelectMgrAndOrFilter::new(SelectMgrFilterType::Or);
    filter.add(&scene.filter.cast());
    let disabled = Handle::new(Shared::new(SharedState::new(BTreeSet::new())));
    filter.set_disabled_objects(&disabled);
    disabled
        .write()
        .insert(scene.object.object_address() as usize);
    assert!(!filter.is_ok(&scene.owner(0)).unwrap());
    assert!(scene.filter.observed.read().is_empty());
    disabled.write().clear();
    assert!(filter.is_ok(&scene.owner(0)).is_err());
    filter.set_filter_type(SelectMgrFilterType::And);
    assert_eq!(filter.filter_type(), SelectMgrFilterType::And);
}
