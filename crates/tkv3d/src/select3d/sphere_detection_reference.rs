//! A borrowed live view preserves native detection aliasing without holding a read lock.
use rustycad_tkernel::standard::SharedState;
use rustycad_tkmath::gp::Point3;

/// A live reference to one sphere's detected point, valid while that sphere is borrowed.
/// Example: `let point = sphere.last_detected_point_ref(); sphere.reset_last_detected_point(); point.point();`.
#[derive(Clone, Copy)]
pub struct Select3DSphereDetectionRef<'a> {
    point: &'a SharedState<Point3>,
}
impl<'a> Select3DSphereDetectionRef<'a> {
    pub(super) fn new(point: &'a SharedState<Point3>) -> Self {
        // Holding a lock here would deadlock later native-style mutation through the same handle.
        Self { point }
    }
    /// Reads the current point through this retained alias, including subsequent resets.
    /// Example: `let coordinates = sphere.last_detected_point_ref().point();`.
    pub fn point(&self) -> Point3 {
        // Only this observation holds a lock; the retained view never blocks selection updates.
        *self.point.read()
    }
}
