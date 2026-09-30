//! Extract closed cycles first, then chain remaining edges into open wires.
use super::*;
impl ShapeAnalysisFreeBounds {
    /// Split wires into elementary closed loops and remaining open chains.
    /// Example: `let (closed,open)=ShapeAnalysisFreeBounds::split_wires(&wires,1e-7,true)?;`.
    pub fn split_wires(
        wires: &[Shape],
        tolerance: f64,
        shared: bool,
    ) -> Result<(Vec<Shape>, Vec<Shape>), Failure> {
        let mut closed = Vec::new();
        let mut open = Vec::new();
        Self::split_wires_into(wires, tolerance, shared, &mut closed, &mut open)?;
        Ok((closed, open))
    }
    /// Retain completed preceding wires if a later split raises, as the native output overload.
    /// Example: `ShapeAnalysisFreeBounds::split_wires_into(&wires,1e-7,true,&mut closed,&mut open)?`.
    pub fn split_wires_into(
        wires: &[Shape],
        tolerance: f64,
        shared: bool,
        closed: &mut Vec<Shape>,
        open: &mut Vec<Shape>,
    ) -> Result<(), Failure> {
        closed.clear();
        open.clear();
        for wire in wires {
            let (loops, chains) = split_one(&topods::wire(wire)?, tolerance, shared)?;
            closed.extend(loops);
            open.extend(chains);
        }
        Ok(())
    }
}
struct LoopSearch {
    edges: Vec<Edge>,
    statuses: Vec<u8>,
    sequence: Vec<usize>,
    closed: Vec<Shape>,
    tolerance: f64,
    shared: bool,
}
fn split_one(
    wire: &Wire,
    tolerance: f64,
    shared: bool,
) -> Result<(Vec<Shape>, Vec<Shape>), Failure> {
    let mut search = LoopSearch::new(wire, tolerance, shared)?;
    for index in 0..search.edges.len() {
        if search.statuses[index] != 0 {
            continue;
        }
        search.sequence.push(index);
        search.statuses[index] = 1;
        search.search_branch()?;
    }
    let mut remaining = search
        .edges
        .iter()
        .zip(&search.statuses)
        .filter(|(_, status)| **status != 2)
        .map(|(edge, _)| edge.clone().into_shape())
        .collect::<Vec<_>>();
    let open = ShapeAnalysisFreeBounds::connect_edges_to_wires(&mut remaining, tolerance, shared)?;
    Ok((search.closed, open))
}
impl LoopSearch {
    fn new(wire: &Wire, tolerance: f64, shared: bool) -> Result<Self, Failure> {
        let wire = ShapeExtendWireData::from_wire(wire, true, true)?;
        let edges = (1..=wire.nb_edges())
            .map(|i| wire.edge(i))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            statuses: vec![0; edges.len()],
            edges,
            sequence: Vec::new(),
            closed: Vec::new(),
            tolerance: tolerance.max(precision::confusion()),
            shared,
        })
    }
    fn next_edge(&self) -> Result<Option<usize>, Failure> {
        for index in 0..self.edges.len() {
            if self.statuses[index] == 0 && self.connects(index)? {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn search_branch(&mut self) -> Result<(), Failure> {
        let mut backward = true;
        while !self.sequence.is_empty() {
            if backward {
                backward = false;
                self.extract_cycle()?;
            }
            if self.sequence.is_empty() {
                return Ok(());
            }
            if let Some(index) = self.next_edge()? {
                self.sequence.push(index);
                self.statuses[index] = 1;
                backward = true;
            } else if let Some(index) = self.sequence.pop() {
                self.statuses[index] = 3;
            }
        }
        Ok(())
    }
    fn connects(&self, index: usize) -> Result<bool, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let last = analyzer.last_vertex(&self.edges[*self.sequence.last().unwrap()]);
        let first = analyzer.first_vertex(&self.edges[index]);
        let p1 = BRepTool::pnt(&last)?;
        let p2 = BRepTool::pnt(&first)?;
        Ok(if self.shared {
            last.is_same(&first)
        } else {
            p1.is_equal(&p2, self.tolerance)
        })
    }
    fn extract_cycle(&mut self) -> Result<(), Failure> {
        let mut start = None;
        for position in (0..self.sequence.len()).rev() {
            if self.connects(self.sequence[position])? {
                start = Some(position);
                break;
            }
        }
        let Some(start) = start else {
            return Ok(());
        };
        self.store_cycle(start)
    }
    fn store_cycle(&mut self, start: usize) -> Result<(), Failure> {
        let indices = self.sequence.split_off(start);
        let edges = indices
            .iter()
            .map(|&i| self.edges[i].clone().into_shape())
            .collect::<Vec<_>>();
        let wire = make_wire(&edges)?;
        wire.set_closed(true)?;
        for index in indices {
            self.statuses[index] = 2;
        }
        self.closed.push(wire.into_shape());
        Ok(())
    }
}
