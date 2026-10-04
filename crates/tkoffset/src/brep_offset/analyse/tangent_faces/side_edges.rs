use super::*;
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
impl BRepOffsetAnalyse {
    pub(super) fn apply_side_edge(
        &mut self,
        vertex: &Shape,
        face: &Face,
        prism: &mut BRepPrimAPIMakePrism,
        ancestors: &IndexedDataMapOfShapeListOfShape,
        selection: &TangentSelection,
        used: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let generated = first_generated(prism, vertex)?;
        self.generated.bind(vertex.clone(), generated.clone());
        let mut edge = occurrence(face, &generated)?;
        orient_side_edge(vertex, &mut edge, selection)?;
        let incident = self.ancestors.change_find(vertex)?;
        if !incident.iter().any(|old| old.is_equal(&edge)) {
            incident.push(edge.clone().into());
        }
        used.add(edge.clone().into());
        self.insert_ancestors(
            &edge,
            ancestors.find_from_key(&edge)?.iter().cloned().collect(),
        );
        if let Ok(faces) = selection.vertex_faces.find(vertex) {
            self.ancestors
                .change_find(&edge)?
                .extend(faces.iter().cloned());
        }
        self.intervals.bind(edge.clone().into(), Vec::new());
        if self.ancestors(&edge)?.len() == 2 {
            self.reclassify(&edge)?;
        }
        Ok(())
    }
}
fn orient_side_edge(
    vertex: &Shape,
    edge: &mut Edge,
    selection: &TangentSelection,
) -> Result<(), Failure> {
    let Ok(minimum) = selection.minimum_edges.find(vertex) else {
        return Ok(());
    };
    let Ok(faces) = selection.vertex_faces.find(vertex) else {
        return Ok(());
    };
    if faces.len() != 1 {
        return Ok(());
    }
    let minimum = occurrence(&faces[0], minimum)?;
    if vertex_orientation(vertex, &minimum) == vertex_orientation(vertex, edge) {
        edge.reverse();
    }
    Ok(())
}
fn vertex_orientation(vertex: &Shape, edge: &Edge) -> Orientation {
    children(edge)
        .iter()
        .find(|child| child.is_same(vertex))
        .map_or(Orientation::Forward, |child| child.orientation())
}
