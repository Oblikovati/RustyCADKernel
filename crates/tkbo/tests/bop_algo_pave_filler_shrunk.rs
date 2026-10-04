//! Native differential coverage for PaveFiller's vertex/edge shrink preparation.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn fill_shrunk_data_matches_native_vertex_edge_fixture() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&vertex_argument(0.5));
    filler.add_argument(&edge_argument());
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    assert!(filler.context().is_some());
    filler
        .fill_shrunk_data(ShapeEnum::Vertex, ShapeEnum::Edge)
        .unwrap();

    let structure = filler.data_structure().unwrap();
    let blocks = structure.pave_blocks(3).unwrap();
    assert_eq!(blocks.len(), 1);
    let block = blocks[0].lock().unwrap();
    let (first, last, bounds, splittable) = block.shrunk_data();
    let gap = bounds.gap();
    let bounds = bounds.get().unwrap();
    let actual = format!(
        "{} {}\n{first:.8} {last:.8} {} {:.8}\n{:.8} {:.8} {:.8} {:.8} {:.8} {:.8}",
        structure.nb_shapes(),
        blocks.len(),
        splittable as i32,
        gap,
        bounds[0],
        bounds[1],
        bounds[2],
        bounds[3],
        bounds[4],
        bounds[5]
    );
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkbo_pave_filler_shrunk.expected").trim()
    );
}

#[test]
fn fill_shrunk_data_returns_before_creating_blocks_for_disjoint_arguments() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&vertex_argument(20.0));
    filler.add_argument(&edge_argument());
    filler.initialize().unwrap();
    filler
        .fill_shrunk_data(ShapeEnum::Vertex, ShapeEnum::Edge)
        .unwrap();
    assert!(
        filler
            .data_structure()
            .unwrap()
            .pave_blocks(3)
            .unwrap()
            .is_empty()
    );
}

fn vertex_argument(x: f64) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1.0e-7)
        .unwrap();
    compound_with(vertex.into_shape())
}

fn edge_argument() -> Shape {
    let edge = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(1.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap();
    compound_with(edge.into_shape())
}

fn compound_with(child: Shape) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &child).unwrap();
    shape
}
