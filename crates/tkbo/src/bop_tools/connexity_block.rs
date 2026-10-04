use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::{BaseAllocator, List};
/// Shape block, loops and regularity; `ConnexityBlock::default()`.
pub struct ConnexityBlock {
    _allocator: BaseAllocator,
    regular: bool,
    shapes: List<Shape>,
    loops: List<Shape>,
}
impl Default for ConnexityBlock {
    fn default() -> Self {
        Self::with_allocator(BaseAllocator::common())
    }
}
impl ConnexityBlock {
    /// Use one allocator for both lists; `ConnexityBlock::with_allocator(allocator)`.
    pub fn with_allocator(allocator: BaseAllocator) -> Self {
        Self {
            shapes: List::with_allocator(allocator.clone()),
            loops: List::with_allocator(allocator.clone()),
            _allocator: allocator,
            regular: true,
        }
    }
    /// Read block elements; `block.shapes()`.
    pub fn shapes(&self) -> &List<Shape> {
        &self.shapes
    }
    /// Mutate block elements; `block.change_shapes().append(shape)`.
    pub fn change_shapes(&mut self) -> &mut List<Shape> {
        &mut self.shapes
    }
    /// Read constructed loops; `block.loops()`.
    pub fn loops(&self) -> &List<Shape> {
        &self.loops
    }
    /// Mutate constructed loops; `block.change_loops().append(loop_shape)`.
    pub fn change_loops(&mut self) -> &mut List<Shape> {
        &mut self.loops
    }
    /// Assign regularity; `block.set_regular(false)`.
    pub fn set_regular(&mut self, regular: bool) {
        self.regular = regular;
    }
    /// Read regularity; `block.is_regular()`.
    pub fn is_regular(&self) -> bool {
        self.regular
    }
}
