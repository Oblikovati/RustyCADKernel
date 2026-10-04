mod rectangles;
use super::spline_fixture;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, Shape},
    toptools::ShapeMapHasher,
};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkoffset::brep_offset::BRepOffsetAnalyse;
#[test]
fn native_differing_offset_tangent_faces_match() {
    let actual: Vec<String> = (0..5)
        .flat_map(|kind| (0..4).map(move |mode| observe(kind, mode)))
        .collect();
    assert_eq!(
        actual.join("\n") + "\n",
        include_str!("../../../../tools/oracle/tkoffset_analyse_tangent.expected")
    );
}
fn observe(kind: usize, mode: usize) -> String {
    let shape = if kind == 0 {
        spline_fixture::pair(0.)
    } else {
        rectangles::pair(kind == 2 || kind == 4, kind >= 3)
    };
    let faces: Vec<Shape> = Explorer::new(&shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .collect();
    let mut offsets: DataMap<Shape, f64, ShapeMapHasher> = DataMap::default();
    offsets.bind(faces[0].clone(), [1., 2., -2., 0.][mode]);
    let mut analysis = BRepOffsetAnalyse::new();
    analysis.set_offset_value(1.);
    analysis.set_face_offset_map(&offsets);
    analysis.perform(&shape, 0.01).unwrap();
    let edge = Explorer::new(&faces[0], ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .find(|edge| {
            Explorer::new(&faces[1], ShapeEnum::Edge, ShapeEnum::Shape)
                .unwrap()
                .any(|other| other.is_same(edge))
        })
        .unwrap();
    let edge = topods::edge(&edge).unwrap();
    let mut row = vec![
        kind,
        mode,
        usize::from(analysis.is_done()),
        analysis.new_faces().len(),
        usize::from(analysis.has_generated(&edge)),
        analysis.ancestors(&edge).unwrap().len(),
        analysis.type_of(&edge).unwrap()[0].get_type().unwrap() as usize,
    ];
    for face in &faces[..2] {
        row.push(usize::from(
            analysis
                .edge_replacement(&topods::face(face).unwrap(), &edge)
                .is_same(&edge),
        ));
    }
    for vertex in Explorer::new(&edge, ShapeEnum::Vertex, ShapeEnum::Shape).unwrap() {
        row.extend([
            usize::from(analysis.has_generated(&vertex)),
            analysis.ancestors(&vertex).unwrap().len(),
        ]);
        if analysis.has_generated(&vertex) {
            let generated = analysis.generated(&vertex);
            row.extend([
                analysis
                    .ancestors(&vertex)
                    .unwrap()
                    .last()
                    .unwrap()
                    .orientation() as usize,
                generated.orientation() as usize,
                analysis.ancestors(&generated).unwrap().len(),
            ]);
        }
    }
    for face in analysis.new_faces() {
        append_face(&mut row, &analysis, face);
    }
    let mut output = row
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    analysis.perform(&shape, 0.01).unwrap();
    output.push_str(&lifecycle("repeat", kind, mode, &analysis, &edge));
    analysis.clear();
    analysis.perform(&shape, 0.01).unwrap();
    output.push_str(&lifecycle("clear", kind, mode, &analysis, &edge));
    output
}
fn append_face(row: &mut Vec<usize>, analysis: &BRepOffsetAnalyse, face: &Shape) {
    row.push(face.orientation() as usize);
    let edges: Vec<Shape> = Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)
        .unwrap()
        .collect();
    for edge in &edges {
        let intervals = analysis.type_of(&topods::edge(edge).unwrap()).unwrap();
        row.extend([analysis.ancestors(edge).unwrap().len(), intervals.len()]);
        if let Some(interval) = intervals.first() {
            row.push(interval.get_type().unwrap() as usize);
        }
    }
    row.extend([
        edges.len(),
        analysis.descendants(face, false).unwrap().len(),
    ]);
}

fn lifecycle(
    label: &str,
    kind: usize,
    mode: usize,
    analysis: &BRepOffsetAnalyse,
    edge: &topods::Edge,
) -> String {
    format!(
        "\n{label} {kind} {mode} {} {} {}",
        analysis.new_faces().len(),
        usize::from(analysis.has_generated(edge)),
        analysis.type_of(edge).unwrap()[0].get_type().unwrap() as usize
    )
}
