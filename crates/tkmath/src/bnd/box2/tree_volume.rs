//! Bnd_Box2d bounds used by the native mesh segment UBTree/EBTree.
use super::BoundingBox2;
use rustycad_tkernel::ncollection::BoundingVolume;
impl BoundingVolume for BoundingBox2 {
    fn add(&mut self, other: &Self) {
        self.add_box(other);
    }
    fn is_out(&self, other: &Self) -> bool {
        self.is_out_box(other)
    }
    fn square_extent(&self) -> f64 {
        BoundingBox2::square_extent(self)
    }
}
