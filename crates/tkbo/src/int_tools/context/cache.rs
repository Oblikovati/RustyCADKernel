use super::*;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{AllocatedBox, BaseAllocator, DataMap};
use std::sync::Arc;
/// A shared cached native service with allocator-owned storage; `context.fclass2d(&face)?.read()`.
pub type Cached<T> = Arc<AllocatedBox<SharedState<T>>>;
pub(super) struct ShapeCache<T> {
    allocator: BaseAllocator,
    entries: SharedState<DataMap<Shape, Cached<T>, ShapeMapHasher>>,
}
impl<T> ShapeCache<T> {
    pub(super) fn new(allocator: &BaseAllocator) -> Self {
        Self {
            allocator: allocator.clone(),
            entries: SharedState::new(DataMap::with_allocator(
                ShapeMapHasher,
                100,
                allocator.clone(),
            )),
        }
    }
    pub(super) fn get(
        &self,
        shape: &Shape,
        create: impl FnOnce() -> Result<T, Failure>,
    ) -> Result<Cached<T>, Failure> {
        let mut entries = self.entries.write();
        if let Some(service) = entries.seek(shape) {
            return Ok(service.clone());
        }
        let service = Arc::new(self.allocator.allocate_value(SharedState::new(create()?)));
        entries.bind(shape.clone(), service.clone());
        Ok(service)
    }
    pub(super) fn clear(&self) {
        self.entries.write().clear(true);
    }
}
