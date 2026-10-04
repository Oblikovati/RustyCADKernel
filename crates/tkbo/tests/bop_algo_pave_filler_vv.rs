//! Native differential coverage for PaveFiller's first intersection stage.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkmath::gp::Point3;

#[test]
fn perform_vv_creates_one_same_domain_vertex_and_interference() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&compound_vertex(0.0));
    filler.add_argument(&compound_vertex(5.0e-7));
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    filler.perform_vv().unwrap();

    let structure = filler.data_structure().unwrap();
    let merged = Vertex::unchecked(structure.shape(4).unwrap().clone());
    let point = BRepTool::pnt(&merged).unwrap();
    let mut actual = format!(
        "{} {} {}\n{:.8} {:.8}\n{} {} {}\n{} {}\n{} {}",
        structure.nb_shapes(),
        structure.interf_vv_records().len(),
        structure.shape_sd(1).is_some() as i32 + structure.shape_sd(3).is_some() as i32,
        point.x(),
        BRepTool::vertex_tolerance(&merged).unwrap(),
        structure.interf_vv_records().value(0).unwrap().index1(),
        structure.interf_vv_records().value(0).unwrap().index2(),
        structure.interf_vv_records().value(0).unwrap().index_new(),
        1,
        structure.shape_sd(1).unwrap(),
        3,
        structure.shape_sd(3).unwrap()
    );
    let mut separate = BOPAlgoPaveFiller::new();
    separate.add_argument(&compound_vertex(0.0));
    separate.add_argument(&compound_vertex(1.0e-2));
    separate.initialize().unwrap();
    separate.perform_vv().unwrap();
    let separate_structure = separate.data_structure().unwrap();
    actual.push_str(&format!(
        "\nseparate {} {} {}",
        separate_structure.nb_shapes(),
        separate_structure.interf_vv_records().len(),
        separate_structure.interferences().extent()
    ));
    let mut chain = BOPAlgoPaveFiller::new();
    for x in [0.0, 5.0e-7, 1.0e-6] {
        chain.add_argument(&compound_vertex(x));
    }
    chain.options_mut().set_fuzzy_value(1.0e-6);
    chain.initialize().unwrap();
    chain.perform_vv().unwrap();
    let chain_structure = chain.data_structure().unwrap();
    actual.push_str(&format!(
        "\nchain {} {} {}",
        chain_structure.nb_shapes(),
        chain_structure.interf_vv_records().len(),
        chain_structure.shape_sd_items().count()
    ));
    for index in 0..chain_structure.interf_vv_records().len() {
        let record = chain_structure.interf_vv_records().value(index).unwrap();
        actual.push_str(&format!(
            "\n{} {} {}",
            record.index1(),
            record.index2(),
            record.index_new()
        ));
    }
    chain
        .data_structure_mut()
        .unwrap()
        .change_shape_info(6)
        .unwrap()
        .change_box()
        .add_point(Point3::new(10.0, 0.0, 0.0));
    chain.perform_vv().unwrap();
    let chain_structure = chain.data_structure().unwrap();
    let merged = Vertex::unchecked(chain_structure.shape(6).unwrap().clone());
    actual.push_str(&format!(
        "\nchain-repeat {} {} {} {:.8} {:.8}",
        chain_structure.nb_shapes(),
        chain_structure.interf_vv_records().len(),
        chain_structure.shape_sd_items().count(),
        BRepTool::pnt(&merged).unwrap().x(),
        BRepTool::vertex_tolerance(&merged).unwrap()
    ));
    let bounds = chain_structure
        .shape_info(6)
        .unwrap()
        .box_bounds()
        .get()
        .unwrap();
    actual.push_str(&format!(
        "\nchain-box {:.8} {:.8} {:.8} {:.8} {:.8} {:.8}",
        bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5]
    ));
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkbo_pave_filler_vv.expected").trim()
    );
}

fn compound_vertex(x: f64) -> Shape {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1.0e-7)
        .unwrap();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &vertex.into_shape()).unwrap();
    shape
}
