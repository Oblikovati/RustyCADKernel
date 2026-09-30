//! NCollection_ItemsView: the generic `Items()`/`IndexedItems()` view over a map.
//!
//! The native `View<MapType, RefType, Extractor, IsConst>` pairs a map reference with an
//! extractor that turns each map position into a key/value(/index) reference. Here the
//! extractor is a marker type (`KeyValueItems`, `KeyValueIndexItems`, `KeyIndexItems`) and
//! each map states what it yields through `ItemsSource<Extractor>`. Items are Rust tuples,
//! which destructure like the native structured bindings (`for (key, value) in view.begin()`).
mod sources;

use std::marker::PhantomData;

/// Extractor yielding `(key, value)` pairs (native `KeyValueRef`).
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyValueItems;
/// Extractor yielding `(key, value, index)` triples, index one-based (`KeyValueIndexRef`).
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyValueIndexItems;
/// Extractor yielding `(key, index)` pairs, index one-based (native `KeyIndexRef`).
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyIndexItems;

/// A map readable through extractor `E`; implemented by the NCollection maps.
pub trait ItemsSource<E> {
    /// The borrowed item tuple.
    type Item<'a>
    where
        Self: 'a;
    /// Items from the first map position (native `View::begin`).
    fn items_begin(&self) -> impl Iterator<Item = Self::Item<'_>>;
    /// The map's `Size()` (native `View::size`).
    fn items_size(&self) -> usize;
    /// The map's `IsEmpty()` (native `View::empty`).
    fn items_is_empty(&self) -> bool;
}

/// A map writable through extractor `E` (native `View<..., IsConst = false>`).
pub trait ItemsSourceMut<E>: ItemsSource<E> {
    /// The item tuple with a mutable value.
    type ItemMut<'a>
    where
        Self: 'a;
    /// Mutable items from the first map position.
    fn items_begin_mut(&mut self) -> impl Iterator<Item = Self::ItemMut<'_>>;
}

/// Read-only view (`IsConst = true`).
///
/// ```
/// use rustycad_tkernel::ncollection::{DataMap, ItemsView, KeyValueItems};
/// let mut map = DataMap::<i32, &str>::default();
/// map.bind(1, "one");
/// let view = ItemsView::<_, KeyValueItems>::new(&map);
/// assert_eq!((view.size(), view.is_empty()), (1, false));
/// assert_eq!(view.begin().collect::<Vec<_>>(), [(&1, &"one")]);
/// ```
pub struct ItemsView<'a, M: ?Sized, E> {
    map: &'a M,
    extractor: PhantomData<E>,
}

/// Mutable view (`IsConst = false`); `ItemsViewMut::<_, KeyValueItems>::new(&mut map)`.
pub struct ItemsViewMut<'a, M: ?Sized, E> {
    map: &'a mut M,
    extractor: PhantomData<E>,
}

impl<'a, M: ItemsSource<E> + ?Sized, E> ItemsView<'a, M, E> {
    /// Views `map` (native `View(MapRef)`); `ItemsView::<_, KeyValueItems>::new(&map)`.
    pub fn new(map: &'a M) -> Self {
        Self {
            map,
            extractor: PhantomData,
        }
    }

    /// Items from the first entry; exhaustion replaces the native `end()`; `view.begin()`.
    pub fn begin(&self) -> impl Iterator<Item = M::Item<'a>> + use<'a, M, E> {
        self.map.items_begin()
    }

    /// Number of mapped entries; `view.size()`.
    pub fn size(&self) -> usize {
        self.map.items_size()
    }

    /// True for an empty map; `view.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.map.items_is_empty()
    }
}

impl<'a, M: ItemsSourceMut<E> + ?Sized, E> ItemsViewMut<'a, M, E> {
    /// Views `map` mutably (native `View(MapRef)` with `IsConst = false`).
    pub fn new(map: &'a mut M) -> Self {
        Self {
            map,
            extractor: PhantomData,
        }
    }

    /// Items with writable values from the first entry; `view.begin()`.
    pub fn begin(&mut self) -> impl Iterator<Item = M::ItemMut<'_>> {
        self.map.items_begin_mut()
    }

    /// Number of mapped entries; `view.size()`.
    pub fn size(&self) -> usize {
        self.map.items_size()
    }

    /// True for an empty map; `view.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.map.items_is_empty()
    }
}
