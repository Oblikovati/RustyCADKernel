// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_CacheMesh.hxx and
// BRepGraph_CacheMesh.cxx (the service, its slots and drivers) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_CacheMesh`: the registry-owned runtime mesh cache of a graph.
//!
//! Triangulations and polygons produced by meshing drivers live in numbered slots;
//! persistent meshes stay in the topology definitions. The entry getters return
//! copies (natively pointers into the slot), and the `Change*` members lend the entry
//! to a closure under the cache lock, so a closure must not call the cache again.
use crate::brep_graph::BRepGraph;
use crate::brep_graph::cache::{BRepGraphCache, CacheBase, CacheType};
use crate::brep_graph::copy_remap::CopyRemap;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{
    Failure, Guid, Handle, SharedState, StandardTransient, Transient,
};
use rustycad_tkernel::tcollection::AsciiString;
use std::sync::OnceLock;

mod access;
mod copy;
mod dirty;
mod entries;
mod fresh_writes;
mod freshness;

use entries::Slot;
pub use entries::{CoEdgeMeshEntry, DirtySet, EdgeMeshEntry, EntryStamp, FaceMeshEntry, SlotState};

/// `BRepGraph_CacheMesh::SlotId`.
pub type SlotId = u32;

/// `DefaultDisplaySlot`.
pub const DEFAULT_DISPLAY_SLOT: SlotId = 0;

/// `BRepGraph_CacheMesh::Driver`: a mesh recomputation driver a meshing toolkit
/// registers for a slot. `Fill` receives the graph and the dirty set of `Ensure`; it
/// may raise, as natively.
pub trait CacheMeshDriver: Transient {
    /// `ID()`: the driver identity.
    fn id(&self) -> Guid;

    /// `RecipeHash()`: the hash of the meshing parameters; a change clears the slot.
    fn recipe_hash(&self) -> u64;

    /// `Fill(theGraph, theSlot, theDirtySet, theRange)`: recompute the dirty entries.
    fn fill(
        &self,
        graph: &mut BRepGraph,
        slot: SlotId,
        dirty: &DirtySet,
        range: &ProgressRange<'_>,
    ) -> Result<bool, Failure>;
}
impl_transient_interface!(
    dyn CacheMeshDriver,
    "BRepGraph_CacheMesh::Driver",
    StandardTransient
);

// `mySlots` and `myActiveSlot`, behind the cache lock.
pub(super) struct MeshSlots {
    slots: Vec<Slot>,
    active_slot: SlotId,
}

impl Default for MeshSlots {
    // `BRepGraph_CacheMesh()`: the default display slot exists from the start.
    fn default() -> Self {
        let mut slots = Self {
            slots: Vec::new(),
            active_slot: DEFAULT_DISPLAY_SLOT,
        };
        slots.change_slot(DEFAULT_DISPLAY_SLOT);
        slots
    }
}

impl MeshSlots {
    /// `changeSlot(theSlot)`: the slot, created (with every slot below it) when absent.
    fn change_slot(&mut self, slot: SlotId) -> &mut Slot {
        while self.slots.len() <= slot as usize {
            let id = self.slots.len() as SlotId;
            self.slots.push(Slot::new(id));
        }
        let found = &mut self.slots[slot as usize];
        found.id = slot;
        found
    }

    /// `findSlot(theSlot)`.
    fn find_slot(&self, slot: SlotId) -> Option<&Slot> {
        self.slots.get(slot as usize)
    }

    fn find_slot_mut(&mut self, slot: SlotId) -> Option<&mut Slot> {
        self.slots.get_mut(slot as usize)
    }
}

/// `BRepGraph_CacheMesh`: per slot, the cached triangulations of faces, polygons of
/// edges and coedge polygons, each stamped with the slot recipe and generation and
/// bound to its entity's SubtreeGen. It is not copied with the topology except
/// through `CopyFreshTo`, nor transformed or serialized.
///
/// ```
/// use rustycad_tkbrep::brep_graph::cache_mesh::{CacheMesh, DEFAULT_DISPLAY_SLOT};
/// let cache = CacheMesh::new();
/// assert_eq!(cache.active_display_slot(), DEFAULT_DISPLAY_SLOT);
/// assert_eq!(cache.state(DEFAULT_DISPLAY_SLOT).generation, 1);
/// assert!(cache.driver_of(3).is_null());
/// ```
#[derive(Default)]
pub struct CacheMesh {
    transient: StandardTransient,
    base: CacheBase,
    slots: SharedState<MeshSlots>,
}
impl_transient!(CacheMesh, "BRepGraph_CacheMesh", dyn BRepGraphCache, transient,
    interfaces: [dyn BRepGraphCache]);

/// `GetID()`: d706ee7b-3c3a-4994-8fdc-c6902bd81450.
const CACHE_MESH_ID: Guid = Guid::from_components(
    0xd706ee7b_u32 as i32,
    0x3c3a,
    0x4994,
    0x8fdc,
    0xc6,
    0x90,
    0x2b,
    0xd8,
    0x14,
    0x50,
);

impl CacheType for CacheMesh {
    fn get_id() -> Guid {
        CACHE_MESH_ID
    }
}

impl CacheMesh {
    /// `BRepGraph_CacheMesh()`: detached, with the default display slot.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ActiveDisplaySlot()`: the slot the effective mesh view reads.
    pub fn active_display_slot(&self) -> SlotId {
        self.slots.read().active_slot
    }

    /// `SetActiveDisplaySlot(theSlot)`.
    pub fn set_active_display_slot(&self, slot: SlotId) {
        self.slots.write().active_slot = slot;
    }

    /// `RegisterDriver(theSlot, theDriver)`: the same driver (same ID and recipe) is
    /// only replaced; another one also takes its recipe and clears the slot.
    pub fn register_driver(&self, slot: SlotId, driver: &Handle<dyn CacheMeshDriver>) {
        let mut slots = self.slots.write();
        let found = slots.change_slot(slot);
        if is_same_driver(&found.driver, driver) {
            found.driver = driver.clone();
            return;
        }
        found.driver = driver.clone();
        found.recipe_hash = driver.get().map_or(0, CacheMeshDriver::recipe_hash);
        found.clear();
    }

    /// `UnregisterDriver(theSlot)`: drops the driver and clears an existing slot.
    pub fn unregister_driver(&self, slot: SlotId) {
        let mut slots = self.slots.write();
        let Some(found) = slots.find_slot_mut(slot) else {
            return;
        };
        found.driver = Handle::default();
        found.recipe_hash = 0;
        found.clear();
    }

    /// `DriverOf(theSlot)`: the slot's driver, a null handle for a missing slot.
    pub fn driver_of(&self, slot: SlotId) -> Handle<dyn CacheMeshDriver> {
        let slots = self.slots.read();
        slots
            .find_slot(slot)
            .map(|found| found.driver.clone())
            .unwrap_or_default()
    }

    /// `State(theSlot)`: the recipe, generation and driver presence of a slot (the
    /// default state for a missing slot).
    pub fn state(&self, slot: SlotId) -> SlotState {
        let slots = self.slots.read();
        let mut state = SlotState {
            slot,
            ..SlotState::default()
        };
        if let Some(found) = slots.find_slot(slot) {
            state.recipe_hash = found.recipe_hash;
            state.generation = found.generation;
            state.has_driver = found.driver.is_some();
        }
        state
    }
}

// `isSameDriver`: two null drivers, or two drivers with the same ID and recipe.
fn is_same_driver(left: &Handle<dyn CacheMeshDriver>, right: &Handle<dyn CacheMeshDriver>) -> bool {
    match (left.get(), right.get()) {
        (Some(left), Some(right)) => {
            left.id() == right.id() && left.recipe_hash() == right.recipe_hash()
        }
        (None, None) => true,
        _ => false,
    }
}

impl BRepGraphCache for CacheMesh {
    fn cache_base(&self) -> &CacheBase {
        &self.base
    }

    fn id(&self) -> Guid {
        CACHE_MESH_ID
    }

    fn name(&self) -> &AsciiString {
        static NAME: OnceLock<AsciiString> = OnceLock::new();
        NAME.get_or_init(|| AsciiString::new("BRepGraph.CacheMesh"))
    }

    /// `Clear()`: every slot is cleared (its generation bumped); the drivers stay.
    fn clear(&self) {
        let mut slots = self.slots.write();
        slots.slots.iter_mut().for_each(Slot::clear);
    }

    fn copy_fresh_to(&self, copy: &CopyRemap<'_>) -> Result<(), Failure> {
        self.copy_slots_to(copy)
    }
}
