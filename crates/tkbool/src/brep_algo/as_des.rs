//! Bidirectional shape ancestry used while offset intersections are assembled.
use rustycad_tkbrep::{topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{DataMap, List},
    standard::{Failure, StandardTransient},
};
mod mutation;
use mutation::collect_common_descendants;
/// Stores descendant and ascendant shape associations; `BRepAlgoAsDes::new()`.
pub struct BRepAlgoAsDes {
    parent: StandardTransient,
    relations: Relations,
}
struct Relations {
    up: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    down: DataMap<Shape, List<Shape>, ShapeMapHasher>,
}
impl_transient!(BRepAlgoAsDes, "BRepAlgo_AsDes", StandardTransient, parent);
impl Default for BRepAlgoAsDes {
    fn default() -> Self {
        Self::new()
    }
}
impl BRepAlgoAsDes {
    /// Creates empty bidirectional associations; `BRepAlgoAsDes::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            relations: Relations {
                up: DataMap::default(),
                down: DataMap::default(),
            },
        }
    }
    /// Removes every ancestor and descendant relation; `associations.clear()`.
    pub fn clear(&mut self) {
        self.relations.up.clear(true);
        self.relations.down.clear(true);
    }
    /// Adds one descendant without deduplication; `associations.add(&face, &edge)`.
    pub fn add(&mut self, source: &Shape, descendant: &Shape) {
        self.relations
            .down
            .try_emplaced(source.clone(), List::default)
            .append(descendant.clone());
        self.relations
            .up
            .try_emplaced(descendant.clone(), List::default)
            .append(source.clone());
    }
    /// Adds each descendant in input order; `associations.add_many(&face, &edges)`.
    pub fn add_many(&mut self, source: &Shape, descendants: &List<Shape>) {
        for descendant in descendants.iter() {
            self.add(source, descendant);
        }
    }
    /// Reports an ancestor association; `associations.has_ascendant(&edge)`.
    pub fn has_ascendant(&self, shape: &Shape) -> bool {
        self.relations.up.is_bound(shape)
    }
    /// Reports a descendant association; `associations.has_descendant(&face)`.
    pub fn has_descendant(&self, shape: &Shape) -> bool {
        self.relations.down.is_bound(shape)
    }
    /// Borrows ancestors in insertion order; `associations.ascendant(&edge)`.
    pub fn ascendant(&self, shape: &Shape) -> impl Iterator<Item = &Shape> {
        self.relations
            .up
            .seek(shape)
            .into_iter()
            .flat_map(List::iter)
    }
    /// Borrows descendants in insertion order; `associations.descendant(&face)`.
    pub fn descendant(&self, shape: &Shape) -> impl Iterator<Item = &Shape> {
        self.relations
            .down
            .seek(shape)
            .into_iter()
            .flat_map(List::iter)
    }
    /// Borrows a descendant list when bound; absent keys return `None` safely.
    /// Example: `if let Some(edges) = associations.change_descendant(&face) { edges.clear(); }`.
    pub fn change_descendant(&mut self, shape: &Shape) -> Option<&mut List<Shape>> {
        self.relations.down.change_seek(shape)
    }
    /// Replaces an associated shape in both directions; `associations.replace(&old, &new)`.
    pub fn replace(&mut self, old: &Shape, new: &Shape) {
        self.relations.replace(old, new);
    }
    /// Removes a non-ancestor leaf; errors when the shape is itself a source or unbound.
    /// Example: `associations.remove(&edge)?`.
    pub fn remove(&mut self, shape: &Shape) -> Result<(), Failure> {
        self.relations.remove(shape)
    }
    /// Lists descendants common to both shapes and reports whether any were found.
    /// Example: `let found = associations.has_common_descendant(&face1, &face2, &mut common);`.
    pub fn has_common_descendant(
        &self,
        first: &Shape,
        second: &Shape,
        common: &mut List<Shape>,
    ) -> bool {
        common.clear();
        for shape in collect_common_descendants(&self.relations, first, second) {
            common.append(shape);
        }
        !common.is_empty()
    }
}
