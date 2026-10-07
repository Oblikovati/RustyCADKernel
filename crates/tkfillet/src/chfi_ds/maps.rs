// Rust translation of ChFiDS_Map.hxx/.cxx and ChFiDS_StripeMap.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The ancestor map of the fillet builders and the map from vertices to stripes.
use super::stripe::StripeHandle;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

/// The sub-shapes of one type of a shape, each with the list of its ancestors of another
/// type (`ChFiDS_Map`, an IndexedDataMapOfShapeListOfShape).
///
/// ```
/// use rustycad_tkbrep::topods::{Builder, Compound};
/// use rustycad_tkfillet::chfi_ds::Map;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let mut compound = Compound::default();
/// Builder.make_compound(&mut compound);
/// let mut map = Map::new();
/// map.fill(&compound, ShapeEnum::Compound, ShapeEnum::Solid)?;
/// assert!(map.contains(&compound));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct Map {
    map: IndexedDataMapOfShapeListOfShape,
}

impl Map {
    /// An empty map; `Map::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Fill(S, T1, T2)`: adds the sub-shapes of type `sub_kind` of `shape` with their
    /// ancestors of type `ancestor_kind` (`TopExp::MapShapesAndAncestors`);
    /// `map.fill(&solid, ShapeEnum::Edge, ShapeEnum::Face)?`.
    pub fn fill(
        &mut self,
        shape: &Shape,
        sub_kind: ShapeEnum,
        ancestor_kind: ShapeEnum,
    ) -> Result<(), Failure> {
        topexp::map_shapes_and_ancestors(shape, sub_kind, ancestor_kind, &mut self.map)
    }

    /// `Contains(S)`; `map.contains(&edge)`.
    pub fn contains(&self, shape: &Shape) -> bool {
        self.map.contains(shape)
    }

    /// `FindFromKey(S)` (and `operator()`): Standard_NoSuchObject for an unbound shape;
    /// `map.find_from_key(&edge)?`.
    pub fn find_from_key(&self, shape: &Shape) -> Result<&List<Shape>, Failure> {
        self.map.find_from_key(shape)
    }

    /// `FindFromIndex(I)` (and `operator()`): Standard_OutOfRange outside `[1, Extent]`;
    /// `map.find_from_index(1)?`.
    pub fn find_from_index(&self, index: i32) -> Result<&List<Shape>, Failure> {
        self.map.find_from_index(index_of(index)?)
    }
}

/// The vertices of the fillet ends with the stripes that meet there (`ChFiDS_StripeMap`,
/// an IndexedDataMap from vertex to the list of stripes).
///
/// ```
/// use rustycad_tkbrep::topods::Vertex;
/// use rustycad_tkfillet::chfi_ds::{Stripe, StripeMap, shared};
/// let mut map = StripeMap::new();
/// map.add(&Vertex::default(), &shared(Stripe::new()));
/// assert_eq!(map.extent(), 1);
/// ```
#[derive(Default)]
pub struct StripeMap {
    map: IndexedDataMap<Shape, Vec<StripeHandle>, ShapeMapHasher>,
}

impl StripeMap {
    /// An empty map; `StripeMap::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Add(V, F)`: appends `stripe` to the list of `vertex`, binding it first when new;
    /// `map.add(&vertex, &stripe)`.
    pub fn add(&mut self, vertex: &Vertex, stripe: &StripeHandle) {
        let key: &Shape = vertex;
        let mut index = self.map.find_index(key);
        if index == 0 {
            index = self.map.add(key.clone(), Vec::new());
        }
        self.map[index].push(stripe.clone());
    }

    /// `Extent()`; `map.extent()`.
    pub fn extent(&self) -> i32 {
        self.map.extent() as i32
    }

    /// `FindFromKey(V)` (and `operator()`): Standard_NoSuchObject for an unbound vertex;
    /// `map.find_from_key(&vertex)?`.
    pub fn find_from_key(&self, vertex: &Vertex) -> Result<&[StripeHandle], Failure> {
        Ok(self.map.find_from_key(vertex)?)
    }

    /// `FindFromIndex(I)` (and `operator()`): Standard_OutOfRange outside `[1, Extent]`;
    /// `map.find_from_index(1)?`.
    pub fn find_from_index(&self, index: i32) -> Result<&[StripeHandle], Failure> {
        Ok(self.map.find_from_index(index_of(index)?)?)
    }

    /// `FindKey(I)`: the `index`-th vertex; Standard_OutOfRange outside `[1, Extent]`;
    /// `map.find_key(1)?`.
    pub fn find_key(&self, index: i32) -> Result<Vertex, Failure> {
        topods::vertex(self.map.find_key(index_of(index)?)?)
    }

    /// `Clear()`; `map.clear()`.
    pub fn clear(&mut self) {
        self.map.clear(true);
    }
}

/// A 1-based map index; a negative index is out of range.
fn index_of(index: i32) -> Result<usize, Failure> {
    usize::try_from(index).map_err(|_| {
        Failure::out_of_range(&format!("ChFiDS map index {index}; expected an index >= 1"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chfi_ds::{Stripe, same_object, shared};

    #[test]
    fn stripes_accumulate_per_vertex() {
        let mut map = StripeMap::new();
        let vertex = Vertex::default();
        let (first, second) = (shared(Stripe::new()), shared(Stripe::new()));
        map.add(&vertex, &first);
        map.add(&vertex, &second);
        assert_eq!(map.extent(), 1);
        let stripes = map.find_from_index(1).unwrap();
        assert!(same_object(&stripes[0], &first) && same_object(&stripes[1], &second));
        assert_eq!(map.find_from_key(&vertex).unwrap().len(), 2);
        assert!(map.find_key(1).unwrap().is_same(&vertex));
        map.clear();
        assert_eq!(map.extent(), 0);
        assert!(map.find_from_index(1).is_err());
    }

    #[test]
    fn unbound_keys_and_indices_raise() {
        let map = Map::new();
        assert!(map.find_from_key(&Shape::default()).is_err());
        assert!(map.find_from_index(-1).is_err());
        assert!(!map.contains(&Shape::default()));
    }
}
