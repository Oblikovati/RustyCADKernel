//! Native loop detection, output-map accumulation and precision/status lifetime.
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Edge, Shape, Vertex},
    toptools::{IndexedMapOfShape, MapOfShape, ShapeMapHasher},
};
use rustycad_tkernel::{
    ncollection::{DataMap, List},
    precision,
    standard::Handle,
};
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisWire,
    shape_extend::{ShapeExtendStatus, ShapeExtendWireData},
};
const EXPECTED: &str = include_str!("../../../tools/oracle/tkshhealing_wire_loops.expected");

#[derive(Default)]
struct LoopOutputs {
    loops: IndexedMapOfShape,
    incidence: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    small: MapOfShape,
    seams: MapOfShape,
}

#[test]
fn loop_incidence_and_state_match_pinned_native_cases() {
    for (mode, row) in EXPECTED.lines().enumerate() {
        let vertices: [Vertex; 4] = std::array::from_fn(|index| support::vertex(index as f64, 0.));
        let analyzer = ShapeAnalysisWire::new();
        analyzer.load_wire_data(&fixture_wire(mode, &vertices));
        analyzer.set_precision(0.125);
        let mut outputs = LoopOutputs::default();
        if mode == 9 {
            outputs.loops.add(vertices[3].clone().into_shape());
        }
        let mut observed = outputs.observe(&analyzer, &vertices);
        observed.extend(outputs.observe(&analyzer, &vertices));
        analyzer.clear_statuses();
        observed.push(i32::from(analyzer.status_loop(ShapeExtendStatus::Ok)));
        let expected: Vec<i32> = row
            .split_whitespace()
            .skip(1)
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(observed, expected, "native loop case {row}");
    }
}

impl LoopOutputs {
    fn observe(&mut self, analyzer: &ShapeAnalysisWire, vertices: &[Vertex; 4]) -> Vec<i32> {
        let result = analyzer
            .check_loop(
                &mut self.loops,
                &mut self.incidence,
                &mut self.small,
                &mut self.seams,
            )
            .unwrap();
        let mut observed = vec![i32::from(result)];
        for flag in [
            ShapeExtendStatus::Done1,
            ShapeExtendStatus::Fail1,
            ShapeExtendStatus::Fail2,
        ] {
            observed.extend([
                i32::from(analyzer.last_check_status(flag)),
                i32::from(analyzer.status_loop(flag)),
            ]);
        }
        observed.extend([
            i32::from(analyzer.precision() == precision::infinite()),
            self.loops.extent() as i32,
            self.incidence.extent() as i32,
            self.small.extent() as i32,
            self.seams.extent() as i32,
        ]);
        for vertex in vertices {
            let shape = vertex.clone().into_shape();
            observed.push(self.loops.find_index(&shape) as i32);
            observed.push(self.incidence.find(&shape).map_or(0, List::extent));
        }
        observed
    }
}

fn fixture_wire(mode: usize, vertices: &[Vertex; 4]) -> Handle<ShapeExtendWireData> {
    let edges: [Edge; 3] =
        std::array::from_fn(|index| support::edge(&vertices[0], &vertices[index + 1]));
    let wire = Handle::new(ShapeExtendWireData::new());
    if mode != 0 {
        wire.add_edge(&edges[0], 0).unwrap();
    }
    if mode >= 2 {
        wire.add_edge(&edges[usize::from(mode != 4)], 0).unwrap();
    }
    if matches!(mode, 3 | 4 | 6) {
        wire.add_edge(&edges[if mode == 4 { 0 } else { 2 }], 0)
            .unwrap();
    }
    append_special_edge(mode, &wire, &edges, vertices);
    wire
}

fn append_special_edge(
    mode: usize,
    wire: &ShapeExtendWireData,
    edges: &[Edge; 3],
    vertices: &[Vertex; 4],
) {
    match mode {
        5 => wire
            .add_edge(&Edge::unchecked(edges[0].reversed()), 0)
            .unwrap(),
        6 => BRepBuilder::default().degenerated(&edges[2], true).unwrap(),
        7 => wire
            .add_edge(&support::edge(&vertices[0], &vertices[0]), 0)
            .unwrap(),
        8 => {
            let mut edge = Edge::default();
            BRepBuilder::default().make_edge(&mut edge).unwrap();
            wire.add_edge(&edge, 0).unwrap();
        }
        _ => (),
    }
}
