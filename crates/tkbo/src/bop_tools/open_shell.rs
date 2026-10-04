use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{Edge, Shell},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
/// Detect an edge with exactly one manifold face occurrence; `is_open_shell(&shell)?`.
pub fn is_open_shell(shell: &Shell) -> Result<bool, Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shell, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    for index in 1..=ancestors.extent() {
        let edge = Edge::unchecked(ancestors.find_key(index)?.clone());
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        let count = ancestors
            .find_from_index(index)?
            .iter()
            .filter(|face| {
                !matches!(
                    face.orientation(),
                    Orientation::Internal | Orientation::External
                )
            })
            .count();
        if count == 1 {
            return Ok(true);
        }
    }
    Ok(false)
}
