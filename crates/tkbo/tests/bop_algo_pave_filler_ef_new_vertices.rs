//! Native differential coverage for EF fused-vertex VE post-treatment.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, Vertex},
};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::fmt::Write;

#[test]
fn perform_ef_reprojects_fused_vertices_before_splitting_all_source_edges() {
    let edges = translated_edge_arguments();
    let face = plane_face();
    let mut filler = configured_filler(&edges, &face);
    filler.perform_ef().unwrap();
    assert_eq!(
        snapshot(&mut filler, &edges),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_new_vertices.expected")
    );
}

fn translated_edge_arguments() -> [Shape; 3] {
    [
        line_edge((-1.0, 0.0, -1.0), (1.0, 0.0, 1.0)),
        line_edge((-0.99975, 0.00025, -1.0), (1.00025, 0.00025, 1.0)),
        line_edge((-0.9995, 0.0005, -1.0), (1.0005, 0.0005, 1.0)),
    ]
}

fn line_edge(first: (f64, f64, f64), last: (f64, f64, f64)) -> Shape {
    BRepBuilderAPIMakeEdge::from_points(point(first), point(last))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn point(coordinates: (f64, f64, f64)) -> Point3 {
    Point3::new(coordinates.0, coordinates.1, coordinates.2)
}

fn plane_face() -> Shape {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2.0, 2.0, -2.0, 2.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn configured_filler(edges: &[Shape; 3], face: &Shape) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    edges.iter().for_each(|edge| filler.add_argument(edge));
    filler.add_argument(face);
    filler.options_mut().set_fuzzy_value(0.0004);
    filler.initialize().unwrap();
    filler
}

fn snapshot(filler: &mut BOPAlgoPaveFiller, edges: &[Shape; 3]) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let mut rows = format!(
        "after {} {} {}\n",
        structure.nb_shapes(),
        structure.interf_ef().len(),
        structure.interf_ve().len()
    );
    append_records(structure, &mut rows);
    append_blocks(structure, edges, &mut rows);
    append_generated_vertex(structure, &mut rows);
    rows
}

fn append_records(structure: &mut rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    for record in structure.interf_ef().iter() {
        writeln!(
            rows,
            "ef {} {} {}",
            record.index1(),
            record.index2(),
            record.index_new()
        )
        .unwrap();
    }
}

fn append_blocks(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edges: &[Shape; 3],
    rows: &mut String,
) {
    for edge in edges {
        let index = structure.index(edge);
        for block in structure.pave_blocks(index).unwrap() {
            let block = block.lock().unwrap();
            let (first_vertex, last_vertex) = block.indices();
            let (first, last) = block.range();
            writeln!(
                rows,
                "pb {index} {first_vertex} {last_vertex} {first:.12} {last:.12}"
            )
            .unwrap();
        }
    }
}

fn append_generated_vertex(structure: &rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    let index = structure.nb_shapes() - 1;
    let vertex = Vertex::unchecked(structure.shape(index).unwrap().clone());
    let point = BRepTool::pnt(&vertex).unwrap();
    writeln!(
        rows,
        "v {index} {:.12} {:.12} {:.12} {:.12}",
        point.x(),
        point.y(),
        point.z(),
        BRepTool::vertex_tolerance(&vertex).unwrap()
    )
    .unwrap();
}
