// Rust translation of ModelingData/TKBRep/BRepGraph/BRepGraph_LayerIterator.hxx and
// BRepGraph_CacheIterator.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGraph_LayerIterator` and `BRepGraph_CacheIterator`: the slots of a registry.
//!
//! Both native iterators count the slots at construction and read each slot from the
//! registry when asked; they share this one generic body.
use crate::brep_graph::cache::BRepGraphCache;
use crate::brep_graph::cache_registry::CacheRegistry;
use crate::brep_graph::layer::BRepGraphLayer;
use crate::brep_graph::layer_registry::LayerRegistry;
use rustycad_tkernel::standard::Handle;

/// A registry whose slots an iterator walks.
pub trait SlotRegistry {
    /// The handle a slot holds.
    type Slot;
    /// The current number of slots.
    fn slot_count(&self) -> u32;
    /// The handle in a slot (null past the end).
    fn slot_value(&self, slot: u32) -> Self::Slot;
}

impl SlotRegistry for LayerRegistry {
    type Slot = Handle<dyn BRepGraphLayer>;
    fn slot_count(&self) -> u32 {
        self.nb_layers()
    }
    fn slot_value(&self, slot: u32) -> Self::Slot {
        self.layer(slot)
    }
}

impl SlotRegistry for CacheRegistry {
    type Slot = Handle<dyn BRepGraphCache>;
    fn slot_count(&self) -> u32 {
        self.nb_caches()
    }
    fn slot_value(&self, slot: u32) -> Self::Slot {
        self.cache(slot)
    }
}

/// The shared iterator body: `More`/`Next`/`Value`/`Slot` over the slot count taken at
/// construction, and a Rust `Iterator` (the native `begin`/`end` range).
#[derive(Debug)]
pub struct SlotIterator<'r, R: SlotRegistry> {
    registry: &'r R,
    count: u32,
    current: u32,
}

impl<R: SlotRegistry> Clone for SlotIterator<'_, R> {
    fn clone(&self) -> Self {
        Self {
            registry: self.registry,
            count: self.count,
            current: self.current,
        }
    }
}

/// `BRepGraph_LayerIterator`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::LayerIterator;
/// use rustycad_tkbrep::brep_graph::BRepGraph;
/// let graph = BRepGraph::new();
/// let iterator = LayerIterator::new(graph.layer_registry());
/// assert_eq!(iterator.nb_layers(), 1);
/// assert_eq!(iterator.count(), 1);
/// ```
pub type LayerIterator<'r> = SlotIterator<'r, LayerRegistry>;

/// `BRepGraph_CacheIterator`.
///
/// ```
/// use rustycad_tkbrep::brep_graph::cache_registry::CacheRegistry;
/// let registry = CacheRegistry::new();
/// let iterator = registry.cache_iter();
/// assert!(!iterator.more() && iterator.nb_caches() == 0);
/// ```
pub type CacheIterator<'r> = SlotIterator<'r, CacheRegistry>;

impl<'r, R: SlotRegistry> SlotIterator<'r, R> {
    /// `BRepGraph_LayerIterator(theRegistry)` / `BRepGraph_CacheIterator(theRegistry)`:
    /// from slot 0 over the current slot count.
    pub fn new(registry: &'r R) -> Self {
        Self {
            registry,
            count: registry.slot_count(),
            current: 0,
        }
    }

    /// `More()`: a current slot remains.
    pub fn more(&self) -> bool {
        self.current < self.count
    }

    /// `Next()`: the next slot.
    pub fn next_slot(&mut self) {
        self.current += 1;
    }

    /// `Value()`: the handle in the current slot, read from the registry now.
    pub fn value(&self) -> R::Slot {
        self.registry.slot_value(self.current)
    }

    /// `Slot()`: the current slot index.
    pub fn slot(&self) -> u32 {
        self.current
    }
}

impl SlotIterator<'_, LayerRegistry> {
    /// `NbLayers()`: the count taken at construction.
    pub fn nb_layers(&self) -> u32 {
        self.count
    }
}

impl SlotIterator<'_, CacheRegistry> {
    /// `NbCaches()`: the registry's service count, read now (natively too, unlike the
    /// iteration bound).
    pub fn nb_caches(&self) -> u32 {
        self.registry.nb_caches()
    }
}

impl<R: SlotRegistry> Iterator for SlotIterator<'_, R> {
    type Item = R::Slot;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.more() {
            return None;
        }
        let value = self.value();
        self.next_slot();
        Some(value)
    }
}
