//! Native differential coverage for vertex/face and generated EE vertex classification.
use rustycad_tkbo::{
    bop_algo::{BOPAlgoPaveFiller, Glue},
    bop_ds::DataStructure,
};
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Shape, Vertex},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::fmt::Write;

fn vertex_argument(mode: i32) -> Shape {
    let mut vertex = Vertex::default();
    let x = match mode {
        3 => 2.,
        4 | 8 => 1.,
        _ => 0.25,
    };
    let z = if mode == 1 || mode == 2 { 5e-7 } else { 0. };
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, 0.5, z), 1e-7)
        .unwrap();
    vertex.into_shape()
}

fn configured_filler(mode: i32) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    if mode == 6 {
        for (first, last) in [
            (Point3::new(-0.5, 0., 0.), Point3::new(0.5, 0., 0.)),
            (Point3::new(0., -0.5, 0.), Point3::new(0., 0.5, 0.)),
        ] {
            filler.add_argument(
                &BRepBuilderAPIMakeEdge::from_points(first, last)
                    .unwrap()
                    .edge()
                    .unwrap()
                    .into_shape(),
            );
        }
    } else {
        filler.add_argument(&vertex_argument(mode));
    }
    if mode == 7 {
        let mut vertex = Vertex::default();
        BRepBuilder::default()
            .make_vertex_at(&mut vertex, Point3::new(0.25, 0.5, 1e-7), 1e-7)
            .unwrap();
        filler.add_argument(&vertex.into_shape());
    }
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-1., 1., -1., 1.))
        .unwrap()
        .face()
        .unwrap();
    filler.add_argument(&face.into_shape());
    filler.options_mut().set_fuzzy_value(1e-6);
    filler.set_non_destructive(mode == 2);
    if mode == 5 {
        filler.set_glue(Glue::Full);
    }
    filler.initialize().unwrap();
    if mode == 6 {
        filler.perform_ee().unwrap();
    }
    if mode == 7 {
        filler.perform_vv().unwrap();
    }
    if mode == 8 {
        filler.perform_ve().unwrap();
    }
    filler
}

fn append_snapshot(filler: &mut BOPAlgoPaveFiller, mode: i32, pass: i32, rows: &mut String) {
    let ds = filler.data_structure_mut().unwrap();
    let count = ds.interf_vf().len();
    writeln!(rows, "case {mode} {pass} {} {count}", ds.nb_shapes()).unwrap();
    for record in ds.interf_vf().iter() {
        let (u, v) = record.uv();
        writeln!(
            rows,
            "vf {} {} {} {u:.9} {v:.9}",
            record.index1(),
            record.index2(),
            record.index_new()
        )
        .unwrap();
    }
    for index in 0..ds.nb_shapes() {
        append_shape_row(ds, index, rows);
    }
}

fn append_shape_row(ds: &DataStructure, index: i32, rows: &mut String) {
    match ds.shape_info(index).unwrap().shape_type() {
        ShapeEnum::Vertex => {
            let vertex = Vertex::unchecked(ds.shape(index).unwrap().clone());
            writeln!(
                rows,
                "v {index} {:.9} {}",
                BRepTool::vertex_tolerance(&vertex).unwrap(),
                ds.same_domain_index(index)
            )
            .unwrap();
        }
        ShapeEnum::Face => append_face_row(ds, index, rows),
        _ => (),
    }
}

fn append_face_row(ds: &DataStructure, index: i32, rows: &mut String) {
    let info = ds.face_info(index).unwrap();
    write!(
        rows,
        "f {index} {} {}",
        i32::from(ds.has_face_info(index).unwrap()),
        info.vertices_in().extent()
    )
    .unwrap();
    for vertex in info.vertices_in().iter() {
        write!(rows, " {vertex}").unwrap();
    }
    rows.push('\n');
}

#[test]
fn perform_vf_matches_native_for_projection_gluing_repetition_and_ee_vertices() {
    let mut rows = String::new();
    for mode in 0..9 {
        let mut filler = configured_filler(mode);
        for pass in 0..2 {
            filler.perform_vf().unwrap();
            append_snapshot(&mut filler, mode, pass, &mut rows);
        }
    }
    assert_eq!(rows.lines().count(), 161);
    assert_eq!(
        rows,
        include_str!("../../../tools/oracle/tkbo_pave_filler_vf.expected")
    );
}
