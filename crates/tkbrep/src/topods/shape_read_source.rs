//! Martensite's live topology read protocol preserves native const-field access across external callbacks.
use super::Shape;
use rustycad_tkernel::standard::Failure;
/// Read current shape fields for one native operation; `source.read_current_shape(|shape|Ok(shape.nb_children()))?`.
/// Collectors may explicitly clone at native value-copy boundaries; presentation callbacks happen outside the read frame.
pub trait TopoDSShapeReadSource {
    /// Lend the current field for a bounded operation; `source.read_current_shape(|shape|Ok(shape.is_null()))?`.
    fn read_current_shape<R>(
        &self,
        project: impl FnOnce(&Shape) -> Result<R, Failure>,
    ) -> Result<R, Failure>;
}
impl TopoDSShapeReadSource for Shape {
    fn read_current_shape<R>(
        &self,
        project: impl FnOnce(&Shape) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let shape = self;
        project(shape)
    }
}
