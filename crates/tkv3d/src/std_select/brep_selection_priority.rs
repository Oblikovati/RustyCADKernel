//! Pinned native priorities are independent of shape kind for explicit leaf decomposition.
use super::StdSelectBrepSelectionTool;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
impl StdSelectBrepSelectionTool {
    /// Return native owner priority for the requested decomposition; `Self::get_standard_priority(&shape,ShapeEnum::Edge)?`.
    pub fn get_standard_priority(shape: &Shape, kind: ShapeEnum) -> Result<i32, Failure> {
        match kind {
            ShapeEnum::Vertex => Ok(8),
            ShapeEnum::Edge => Ok(7),
            ShapeEnum::Wire => Ok(6),
            ShapeEnum::Face => Ok(5),
            _ => Ok(default_brep_owner_priority(shape.shape_type()?)),
        }
    }
}
fn default_brep_owner_priority(kind: ShapeEnum) -> i32 {
    match kind {
        ShapeEnum::Vertex => 9,
        ShapeEnum::Edge => 8,
        ShapeEnum::Wire => 7,
        ShapeEnum::Face => 6,
        ShapeEnum::Shell => 5,
        _ => 4,
    }
}
