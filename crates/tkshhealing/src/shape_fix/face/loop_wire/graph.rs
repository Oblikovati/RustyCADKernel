use super::{Failure, Shape, ShapeExtendWireData, ShapeFixFace, Wire, reorder_wire};
use rustycad_tkbrep::{
    topexp,
    topods::ShapeIterator,
    toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::{
    ncollection::{DataMap, List, Sequence},
    standard::Handle,
};
#[derive(Default)]
pub(super) struct LoopGraph {
    junctions: IndexedMapOfShape,
    incidence: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    small: MapOfShape,
    seams: MapOfShape,
    used: MapOfShape,
}
impl LoopGraph {
    pub fn analyze(tool: &ShapeFixFace) -> Result<Option<Self>, Failure> {
        let mut graph = Self::default();
        let found = tool.fix_wire_tool().working_analyzer().check_loop(
            &mut graph.junctions,
            &mut graph.incidence,
            &mut graph.small,
            &mut graph.seams,
        )?;
        Ok(found.then_some(graph))
    }
    pub fn collect_paths(&mut self, output: &mut Sequence<Shape>) -> Result<Vec<Wire>, Failure> {
        let mut open = Vec::new();
        for index in 1..=self.junctions.extent() {
            let vertex = self.junctions.find_key(index)?.clone();
            let edges = self
                .incidence
                .find(&vertex)?
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            for edge in edges {
                if self.used.contains(&edge) {
                    continue;
                }
                self.collect_path(&vertex, &edge, output, &mut open)?;
            }
        }
        Ok(open)
    }
    fn collect_path(
        &mut self,
        vertex: &Shape,
        edge: &Shape,
        output: &mut Sequence<Shape>,
        open: &mut Vec<Wire>,
    ) -> Result<(), Failure> {
        let wire = Handle::new(ShapeExtendWireData::new());
        self.append_edge(&wire, edge)?;
        self.follow_path(vertex.clone(), edge.clone(), &wire)?;
        if wire.nb_edges() == 1 && self.small.contains(&wire.edge(1)?.into_shape()) {
            return Ok(());
        }
        let wire = wire.wire()?;
        let (first, last) = topexp::wire_vertices(&wire);
        if first.is_same(&last) {
            output.append(reorder_wire(&wire)?.into_shape());
        } else {
            open.push(wire);
        }
        Ok(())
    }
    fn append_edge(&mut self, wire: &ShapeExtendWireData, edge: &Shape) -> Result<(), Failure> {
        wire.add_shape(edge, 0)?;
        if self.seams.contains(edge) {
            wire.add_shape(&edge.reversed(), 0)?;
        }
        self.used.add(edge.clone());
        Ok(())
    }
    fn follow_path(
        &mut self,
        mut vertex: Shape,
        mut edge: Shape,
        wire: &ShapeExtendWireData,
    ) -> Result<(), Failure> {
        loop {
            let Some(next_vertex) = self.next_vertex(&vertex, &edge) else {
                return Ok(());
            };
            let next = self
                .incidence
                .find(&next_vertex)?
                .iter()
                .find(|candidate| !self.used.contains(candidate) && !candidate.is_same(&edge))
                .cloned();
            let Some(next) = next else {
                return Ok(());
            };
            self.append_edge(wire, &next)?;
            vertex = next_vertex;
            edge = next;
        }
    }
    fn next_vertex(&self, vertex: &Shape, edge: &Shape) -> Option<Shape> {
        let next = ShapeIterator::new(edge, true, true).find(|child| !child.is_same(vertex));
        match next {
            Some(next) if self.junctions.contains(&next) => None,
            Some(next) => Some(next),
            None if self.small.contains(edge) => Some(vertex.clone()),
            None => None,
        }
    }
}
