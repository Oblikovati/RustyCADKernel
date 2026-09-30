// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerRegistry.hxx and
// BRepGraph_LayerRegistry.cxx (registration and lookup) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerRegistry`: the dense, GUID-keyed registry of a graph's layers.
use crate::brep_graph::layer::{BRepGraphLayer, LayerType, as_layer, attach_graph, detach_context};
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

mod copy;
mod dispatch;

/// The slot vector and the GUID-to-slot map (`myLayers`, `myGuidToSlot`), behind the
/// registry lock (`myMutex`).
#[derive(Default)]
struct RegistrySlots {
    layers: Vec<Handle<dyn BRepGraphLayer>>,
    guid_to_slot: HashMap<Guid, u32>,
}

/// `BRepGraph_LayerRegistry`: registered layers in a dense slot vector with a GUID
/// lookup, the OR of their subscription masks, and the graph they are attached to
/// (its attachment token; 0 for a standalone registry, whose layers stay detached).
/// Every member takes `&self`, the registry locking itself as natively.
///
/// ```
/// use rustycad_tkbrep::brep_graph::layer_history::LayerHistory;
/// use rustycad_tkbrep::brep_graph::layer_registry::LayerRegistry;
/// let registry = LayerRegistry::new();
/// assert_eq!(registry.nb_layers(), 0);
/// let history = registry.ensure::<LayerHistory>().unwrap();
/// assert_eq!(registry.nb_layers(), 1);
/// assert!(registry.find::<LayerHistory>() == history);
/// ```
pub struct LayerRegistry {
    slots: SharedState<RegistrySlots>,
    subscribed_kinds: AtomicU32,
    subscribed_ref_kinds: AtomicU32,
    graph_token: u64,
}

impl Default for LayerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LayerRegistry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LayerRegistry")
            .field("nb_layers", &self.nb_layers())
            .field("graph_token", &self.graph_token)
            .finish()
    }
}

impl LayerRegistry {
    /// `BRepGraph_LayerRegistry()`: an empty standalone registry (no graph).
    pub fn new() -> Self {
        Self::for_graph(0)
    }

    /// The registry a graph owns (`Attach(this)`), attaching its layers to the graph's
    /// attachment token.
    pub(crate) fn for_graph(graph_token: u64) -> Self {
        Self {
            slots: SharedState::new(RegistrySlots::default()),
            subscribed_kinds: AtomicU32::new(0),
            subscribed_ref_kinds: AtomicU32::new(0),
            graph_token,
        }
    }

    /// The attachment token of the owning graph, 0 for a standalone registry.
    pub(crate) fn graph_token(&self) -> u64 {
        self.graph_token
    }

    /// `RegisterLayer(theLayer)`: the slot of the layer, replacing (and detaching) a
    /// registered layer with the same GUID; Standard_ProgramError for a null layer.
    pub fn register_layer(&self, layer: &Handle<dyn BRepGraphLayer>) -> Result<u32, Failure> {
        let mut slots = self.slots.write();
        self.register_layer_locked(&mut slots, layer)
    }

    // `registerLayerLocked`: replace in place or append, then update the masks.
    fn register_layer_locked(
        &self,
        slots: &mut RegistrySlots,
        layer: &Handle<dyn BRepGraphLayer>,
    ) -> Result<u32, Failure> {
        let Some(object) = layer.get() else {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "BRepGraph_LayerRegistry::RegisterLayer() - null layer (expected a non-null handle)",
                "",
            ));
        };
        if let Some(&slot) = slots.guid_to_slot.get(&object.id()) {
            self.replace_layer(slots, slot, layer);
            return Ok(slot);
        }
        let new_slot =
            u32::try_from(slots.layers.len()).map_err(|_| too_many(slots.layers.len()))?;
        attach_graph(object, self.graph_token);
        slots.layers.push(layer.clone());
        slots.guid_to_slot.insert(object.id(), new_slot);
        self.or_masks(object);
        Ok(new_slot)
    }

    // The same-GUID case: detach a different previous layer, attach the new one in its
    // slot, recompute the masks.
    fn replace_layer(
        &self,
        slots: &mut RegistrySlots,
        slot: u32,
        layer: &Handle<dyn BRepGraphLayer>,
    ) {
        let previous = &slots.layers[slot as usize];
        if let Some(object) = previous.get()
            && previous.object_address() != layer.object_address()
        {
            detach_context(object);
        }
        if let Some(object) = layer.get() {
            attach_graph(object, self.graph_token);
        }
        slots.layers[slot as usize] = layer.clone();
        self.recompute_subscribed_kinds_mask(slots);
    }

    fn or_masks(&self, layer: &dyn BRepGraphLayer) {
        let kinds = self.subscribed_kinds.load(Ordering::Relaxed) | layer.subscribed_kinds() as u32;
        self.subscribed_kinds.store(kinds, Ordering::Release);
        let refs =
            self.subscribed_ref_kinds.load(Ordering::Relaxed) | layer.subscribed_ref_kinds() as u32;
        self.subscribed_ref_kinds.store(refs, Ordering::Release);
    }

    /// `UnregisterLayer(theGUID)`: moves the last layer into the freed slot, then
    /// detaches the removed layer; an unknown GUID is ignored.
    pub fn unregister_layer(&self, guid: &Guid) {
        let mut slots = self.slots.write();
        let Some(slot) = slots.guid_to_slot.get(guid).copied() else {
            return;
        };
        let removed = slots.layers.swap_remove(slot as usize);
        if let Some(moved) = slots.layers.get(slot as usize).and_then(Handle::get) {
            let moved_id = moved.id();
            slots.guid_to_slot.insert(moved_id, slot);
        }
        slots.guid_to_slot.remove(guid);
        self.recompute_subscribed_kinds_mask(&slots);
        if let Some(layer) = removed.get() {
            detach_context(layer);
        }
    }

    /// `FindLayer(theGUID)`: the registered layer, a null handle when absent.
    pub fn find_layer(&self, guid: &Guid) -> Handle<dyn BRepGraphLayer> {
        let slots = self.slots.read();
        find_layer_locked(&slots, guid)
    }

    /// `Find<T>()` and `FindLayer<T>()`: the layer registered under `T::GetID()` when it
    /// is a `T`, a null handle otherwise.
    pub fn find<T: LayerType>(&self) -> Handle<T> {
        self.find_layer(&T::get_id()).cast::<T>()
    }

    /// `Ensure<T>()`: the layer registered under `T::GetID()`, registering a new `T`
    /// first when there is none (the down cast is null when another class holds the
    /// GUID, as natively).
    pub fn ensure<T: LayerType>(&self) -> Result<Handle<T>, Failure> {
        let layer = self.ensure_layer(&T::get_id(), || as_layer(&Handle::new(T::default())))?;
        Ok(layer.cast::<T>())
    }

    /// `ensureLayer(theGUID, theFactory)` (private natively): the registered layer, or
    /// the factory's layer registered under the lock.
    pub(crate) fn ensure_layer(
        &self,
        guid: &Guid,
        factory: impl FnOnce() -> Handle<dyn BRepGraphLayer>,
    ) -> Result<Handle<dyn BRepGraphLayer>, Failure> {
        let found = self.find_layer(guid);
        if found.is_some() {
            return Ok(found);
        }
        let mut slots = self.slots.write();
        let found = find_layer_locked(&slots, guid);
        if found.is_some() {
            return Ok(found);
        }
        let layer = factory();
        self.register_layer_locked(&mut slots, &layer)?;
        Ok(layer)
    }

    /// `FindSlot(theGUID, theSlot)`: the slot of a registered GUID.
    pub fn find_slot(&self, guid: &Guid) -> Option<u32> {
        self.slots.read().guid_to_slot.get(guid).copied()
    }

    /// `Layer(theSlot)`: the layer in a slot, a null handle past the end.
    pub fn layer(&self, slot: u32) -> Handle<dyn BRepGraphLayer> {
        self.layer_at(slot)
    }

    /// `layerAt(theSlot)` (private natively): the dispatch loops' slot read.
    fn layer_at(&self, slot: u32) -> Handle<dyn BRepGraphLayer> {
        let slots = self.slots.read();
        slots.layers.get(slot as usize).cloned().unwrap_or_default()
    }

    /// `NbLayers()`: the number of registered layers.
    pub fn nb_layers(&self) -> u32 {
        self.slots.read().layers.len() as u32
    }

    /// `HasModificationSubscribers()`: a layer subscribes to node modifications.
    pub fn has_modification_subscribers(&self) -> bool {
        self.subscribed_kinds.load(Ordering::Acquire) != 0
    }

    /// `SubscribedKindsMask()`: the OR of the node subscription masks.
    pub fn subscribed_kinds_mask(&self) -> i32 {
        self.subscribed_kinds.load(Ordering::Acquire) as i32
    }

    /// `HasRefModificationSubscribers()`: a layer subscribes to reference modifications.
    pub fn has_ref_modification_subscribers(&self) -> bool {
        self.subscribed_ref_kinds.load(Ordering::Acquire) != 0
    }

    /// `SubscribedRefKindsMask()`: the OR of the reference subscription masks.
    pub fn subscribed_ref_kinds_mask(&self) -> i32 {
        self.subscribed_ref_kinds.load(Ordering::Acquire) as i32
    }

    /// `recomputeSubscribedKindsMask()`: the OR over every registered layer.
    fn recompute_subscribed_kinds_mask(&self, slots: &RegistrySlots) {
        let (mut kinds, mut refs) = (0_u32, 0_u32);
        for layer in slots.layers.iter().filter_map(Handle::get) {
            kinds |= layer.subscribed_kinds() as u32;
            refs |= layer.subscribed_ref_kinds() as u32;
        }
        self.subscribed_kinds.store(kinds, Ordering::Release);
        self.subscribed_ref_kinds.store(refs, Ordering::Release);
    }

    /// `detachAllLocked()`: detach every registered layer (the graph destructor's
    /// `Detach`).
    fn detach_all(&self) {
        for layer in self.slots.read().layers.iter().filter_map(Handle::get) {
            detach_context(layer);
        }
    }

    /// `Attach(theGraph)` with the graph the layers are already attached to (the
    /// compaction restores its saved registry): `attachGraph` on every layer, which
    /// detaches (`OnDetached`) and attaches (`OnAttached`) it again.
    pub(crate) fn reattach(&self) {
        for layer in self.slots.read().layers.iter().filter_map(Handle::get) {
            attach_graph(layer, self.graph_token);
        }
    }
}

impl Drop for LayerRegistry {
    // `~BRepGraph` calls `Detach()`: the layers that outlive the graph through their
    // handles are detached.
    fn drop(&mut self) {
        self.detach_all();
    }
}

fn find_layer_locked(slots: &RegistrySlots, guid: &Guid) -> Handle<dyn BRepGraphLayer> {
    slots
        .guid_to_slot
        .get(guid)
        .map(|&slot| slots.layers[slot as usize].clone())
        .unwrap_or_default()
}

fn too_many(count: usize) -> Failure {
    Failure::with_type(
        "Standard_OutOfRange",
        &format!(
            "BRepGraph_LayerRegistry - too many registered layers ({count}, expected at most u32::MAX)"
        ),
        "",
    )
}
