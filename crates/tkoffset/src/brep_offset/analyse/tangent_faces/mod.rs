//! Artificial perpendicular faces between tangent faces with different offsets.
mod blocks;
mod history;
mod selection;
use super::{BRepOffsetAnalyse, ShapeBindings};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::{
    topods::{self, Edge, Face, Shape},
    toptools::MapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::top_abs::ShapeEnum;
#[derive(Default)]
struct TangentSelection {
    edges: Vec<Shape>,
    offsets: ShapeBindings<f64>,
    minimum_edges: ShapeBindings<Shape>,
    vertex_faces: ShapeBindings<Vec<Shape>>,
}
impl BRepOffsetAnalyse {
    pub(super) fn treat_tangent_faces(&mut self, edges: &[Shape]) -> Result<(), Failure> {
        if edges.is_empty() || self.face_offsets.is_empty() {
            return Ok(());
        }
        let selection = self.select_different_offsets(edges)?;
        if selection.edges.is_empty() {
            return Ok(());
        }
        self.build_tangent_blocks(&selection)
    }
    fn face_offset(&self, face: &Shape) -> f64 {
        self.face_offsets
            .find(face)
            .map_or(self.offset, |offset| offset.abs())
    }
    fn insert_ancestors(&mut self, shape: &Shape, parents: Vec<Shape>) {
        if self.ancestors.is_bound(shape) {
            return;
        }
        self.ancestor_order.push(shape.clone());
        self.ancestors.bind(shape.clone(), parents);
    }
    fn reclassify(&mut self, edge: &Edge) -> Result<(), Failure> {
        let parents = self.ancestors(edge)?;
        let interval = super::classify::classify_incidence(edge, parents, self.angle.sin().abs())?;
        self.intervals
            .bind(edge.clone().into(), interval.into_iter().collect());
        Ok(())
    }
}
fn edges_in_face(face: &Shape) -> Result<Vec<Shape>, Failure> {
    Ok(Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)?.collect())
}
fn children(shape: &Shape) -> Vec<Shape> {
    topods::ShapeIterator::new(shape, true, true).collect()
}
fn occurrence(face: &Shape, edge: &Shape) -> Result<Edge, Failure> {
    for candidate in edges_in_face(face)? {
        if candidate.is_same(edge) {
            return topods::edge(&candidate);
        }
    }
    topods::edge(edge)
}

mod side_edges;

fn first_generated(
    prism: &mut rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism,
    source: &Shape,
) -> Result<Shape, Failure> {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
    prism.generated(source)?.first().cloned().ok_or_else(|| {
        Failure::with_message(
            "Standard_NoSuchObject",
            &format!(
                "prism generated list empty for {source:?}; expected a generated face or edge"
            ),
        )
    })
}
