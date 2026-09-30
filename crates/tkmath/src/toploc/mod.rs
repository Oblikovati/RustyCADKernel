// Rust translation of FoundationClasses/TKMath/TopLoc at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Shared datum chains with eagerly computed transformations (OCCT TopLoc, OCC25545).
//!
//! A [`Location`] is an immutable, shared list of `(datum, power)` items. Each list node
//! caches the product of its own powered datum and every item behind it, so reading the
//! transformation is a field read that is safe on many threads at once.
mod datum3d;
mod item_location;
mod location;
mod location_ops;
mod slist;
pub use datum3d::Datum3;
pub use item_location::ItemLocation;
pub use location::Location;
use rustycad_tkernel::ncollection::{IndexedMap, Map};
pub use slist::{SListNodeOfItemLocation, SListOfItemLocation};
/// `TopLoc_MapOfLocation`: locations hashed by datum identity and power; `MapOfLocation::default()`.
pub type MapOfLocation = Map<Location>;
/// `TopLoc_IndexedMapOfLocation`, keyed like [`MapOfLocation`]; `IndexedMapOfLocation::new(1)`.
pub type IndexedMapOfLocation = IndexedMap<Location>;
