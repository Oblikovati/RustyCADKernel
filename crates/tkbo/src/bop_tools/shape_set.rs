use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{Edge, Shape},
};
use rustycad_tkernel::{
    ncollection::{BaseAllocator, DynamicArray},
    standard::Failure,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
/// OCCT occurrence set; equality retains native asymmetric duplicate behavior.
#[derive(Clone)]
pub struct ShapeSet {
    allocator: BaseAllocator,
    occurrences: DynamicArray<Shape>,
    shape: Shape,
    sum: usize,
}
impl Default for ShapeSet {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl ShapeSet {
    /// Create an empty set; `let set = ShapeSet::new();`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Allocate occurrence blocks with the supplied resource; `ShapeSet::with_allocator(allocator)`.
    pub fn with_allocator(allocator: BaseAllocator) -> Self {
        Self {
            occurrences: DynamicArray::with_allocator(256, Some(allocator.clone())),
            allocator,
            shape: Shape::default(),
            sum: 0,
        }
    }
    /// Copy topology occurrences and allocator identity; `target.assign(&source);`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.allocator = source.allocator.clone();
        self.occurrences.assign(&source.occurrences, false);
        self.shape = source.shape.clone();
        self.sum = source.sum;
        self
    }
    /// Borrow the last source shape, even after clear; `set.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }
    /// Count occurrences, with each INTERNAL occurrence counted twice; `set.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.occurrences.length()
    }
    /// Return the process-local sum of normalized topology hashes; `set.get_sum()`.
    pub fn get_sum(&self) -> usize {
        self.sum
    }
    /// Empty occurrences while retaining source shape and block storage; `set.clear();`.
    pub fn clear(&mut self) {
        self.occurrences.clear(false);
        self.sum = 0;
    }
    /// Replace occurrences from a topology traversal; `set.add(&shape,ShapeEnum::Edge)?`.
    pub fn add(&mut self, shape: &Shape, kind: ShapeEnum) -> Result<(), Failure> {
        self.shape = shape.clone();
        self.clear();
        for occurrence in Explorer::new(shape, kind, ShapeEnum::Shape)? {
            if kind == ShapeEnum::Edge
                && BRepTool::degenerated(&Edge::unchecked(occurrence.clone()))?
            {
                continue;
            }
            self.append_occurrence(occurrence);
        }
        let count = self.occurrences.len();
        self.sum = self
            .occurrences
            .iter()
            .map(|shape| normalized_hash(shape.hash_code(), count))
            .fold(0, usize::wrapping_add);
        Ok(())
    }
    /// Compare native cardinality and one-way membership; `left.is_equal(&right)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.nb_shapes() == other.nb_shapes()
            && other.occurrences.iter().all(|candidate| {
                self.occurrences
                    .iter()
                    .any(|shape| shape.is_same(candidate))
            })
    }
    fn append_occurrence(&mut self, shape: Shape) {
        if shape.orientation() == Orientation::Internal {
            self.occurrences
                .append(shape.oriented(Orientation::Forward));
            self.occurrences
                .append(shape.oriented(Orientation::Reversed));
            return;
        }
        self.occurrences.append(shape);
    }
}
fn normalized_hash(hash: usize, count: usize) -> usize {
    let normalized = hash % 432123 + 1;
    let threshold = usize::MAX / count;
    if normalized > threshold {
        normalized % threshold
    } else {
        normalized
    }
}
