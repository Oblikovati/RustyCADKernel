use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp,
    topods::{Edge, Shape, ShapeIterator, Vertex, Wire},
    toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
struct WireOrienter {
    adjacent: IndexedDataMapOfShapeListOfShape,
    visited: MapOfShape,
    result: Wire,
}
/// Rebuild wire adjacency chains with consistent endpoint directions; `orient_edges_on_wire(&mut wire)?`.
pub fn orient_edges_on_wire(wire: &mut Shape) -> Result<(), Failure> {
    let mut adjacent = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(wire, ShapeEnum::Vertex, ShapeEnum::Edge, &mut adjacent)?;
    if adjacent.is_empty() {
        return Ok(());
    }
    let mut result = Wire::default();
    BRepBuilder::default().make_wire(&mut result);
    let mut orienter = WireOrienter {
        adjacent,
        visited: MapOfShape::default(),
        result,
    };
    for shape in ShapeIterator::new(wire, true, true) {
        orienter.start_chain(&Edge::unchecked(shape))?;
    }
    *wire = orienter.result.into_shape();
    Ok(())
}
impl WireOrienter {
    fn start_chain(&mut self, edge: &Edge) -> Result<(), Failure> {
        if !self.visited.add(edge.clone().into_shape()) {
            return Ok(());
        }
        BRepBuilder::default().add(&mut self.result, edge)?;
        let (first, last) = topexp::vertices(edge, true);
        if first.is_same(&last) {
            return Ok(());
        }
        self.extend_chain(first, false)?;
        self.extend_chain(last, true)
    }
    fn extend_chain(&mut self, mut vertex: Vertex, from_last: bool) -> Result<(), Failure> {
        while let Some(edge) = self.next_edge(&vertex)? {
            let (first, last) = topexp::vertices(&edge, true);
            if first.is_same(&last) {
                break;
            }
            let unchanged = if from_last {
                vertex.is_same(&first)
            } else {
                vertex.is_same(&last)
            };
            let oriented = if unchanged {
                edge.clone().into_shape()
            } else {
                edge.reversed()
            };
            BRepBuilder::default().add(&mut self.result, &oriented)?;
            self.visited.add(edge.into_shape());
            vertex = if vertex.is_same(&first) { last } else { first };
        }
        Ok(())
    }
    fn next_edge(&self, vertex: &Vertex) -> Result<Option<Edge>, Failure> {
        let adjacent = self.adjacent.find_from_key(vertex)?;
        if adjacent.extent() != 2 {
            return Ok(None);
        }
        Ok(adjacent
            .iter()
            .find(|edge| !self.visited.contains(edge))
            .map(|edge| Edge::unchecked(edge.clone())))
    }
}
