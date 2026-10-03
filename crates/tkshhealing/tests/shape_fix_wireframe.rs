//! Native small-edge maps and wireframe state preservation.
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Compound, Edge, Face, Shape, Wire},
    toptools::MapOfShape,
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::{
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakePolygon;
fn polygon() -> Wire {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(0., 0., 0.),
        Point3::new(0.01, 0., 0.),
        Point3::new(1., 1., 0.),
        Point3::new(0., 1., 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap()
}
fn fixture(mode: i32) -> (Shape, Edge) {
    let builder = BRepBuilder::default();
    let mut wire = polygon();
    let first = topods::edge(
        Explorer::new(&wire, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .current(),
    )
    .unwrap();
    if mode == 1 {
        return (first.clone().into_shape(), first);
    }
    if mode == 2 || mode == 3 {
        builder.make_wire(&mut wire);
        builder.add(&mut wire, &first).unwrap();
        let other = if mode == 2 {
            first.clone().into_shape()
        } else {
            first.reversed()
        };
        builder.add(&mut wire, &other).unwrap();
    }
    if mode == 4 {
        builder.make_wire(&mut wire);
    }
    if mode < 5 {
        return (wire.into_shape(), first);
    }
    (face_fixture(&wire, mode), first)
}
fn face_fixture(wire: &Wire, mode: i32) -> Shape {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    builder
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    builder.add(&mut face, wire).unwrap();
    if mode == 6 {
        face.reverse();
    }
    if mode != 7 {
        return face.into_shape();
    }
    let mut other = topods::face(&face.empty_copied().unwrap()).unwrap();
    builder.add(&mut other, wire).unwrap();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &face).unwrap();
    builder.add(&mut compound, &other).unwrap();
    compound.into_shape()
}
fn expected(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_small.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}
fn run(name: &str, mode: i32, preload: bool) {
    let (shape, first) = fixture(mode);
    let tool = ShapeFixWireframe::from_shape(&shape);
    tool.set_precision(0.1);
    let mut small = MapOfShape::default();
    let mut multiple = MapOfShape::default();
    let mut edge_faces = ShapeFixWireframeShapeLists::default();
    let mut face_small = ShapeFixWireframeShapeLists::default();
    if preload {
        small.add(first.into_shape());
    }
    let changed = tool
        .check_small_edges(&mut small, &mut edge_faces, &mut face_small, &mut multiple)
        .unwrap();
    let (face_links, small_links, reverse_links) = link_counts(&edge_faces, &face_small);
    let values = vec![
        f64::from(changed),
        small.extent() as f64,
        edge_faces.extent() as f64,
        face_small.extent() as f64,
        multiple.extent() as f64,
        face_links as f64,
        small_links as f64,
        reverse_links as f64,
        f64::from(tool.status_small_edges(ShapeExtendStatus::Ok)),
        f64::from(tool.shape().is_same(&shape)),
    ];
    assert_eq!(values, expected(name), "{name}");
}
fn link_counts(
    edge_faces: &ShapeFixWireframeShapeLists,
    face_small: &ShapeFixWireframeShapeLists,
) -> (usize, usize, usize) {
    let mut face_links = 0;
    let mut reverse_links = 0;
    for (_, faces) in edge_faces.items() {
        for face in faces {
            face_links += 1;
            if face.orientation() == Orientation::Reversed {
                reverse_links += 1;
            }
        }
    }
    let small_links = face_small.items().map(|(_, edges)| edges.len()).sum();
    (face_links, small_links, reverse_links)
}
fn state_values(tool: &ShapeFixWireframe) -> Vec<f64> {
    vec![
        f64::from(tool.shape().is_null()),
        f64::from(tool.mode_drop_small_edges()),
        tool.limit_angle(),
        f64::from(tool.status_wire_gaps(ShapeExtendStatus::Ok)),
        f64::from(tool.status_small_edges(ShapeExtendStatus::Ok)),
    ]
}
#[test]
fn native_wireframe_configuration_survives_load_and_status_reset() {
    let tool = ShapeFixWireframe::new();
    assert_eq!(state_values(&tool), expected("default"));
    tool.set_mode_drop_small_edges(true);
    tool.set_limit_angle(0.25);
    tool.load(&polygon().into_shape());
    tool.clear_statuses();
    assert_eq!(state_values(&tool), expected("loaded"));
}
#[test]
fn native_small_edge_incidence_and_multiplicity() {
    for (name, mode) in [
        ("free", 0),
        ("edge", 1),
        ("duplicate", 2),
        ("seam", 3),
        ("empty", 4),
        ("face", 5),
        ("reversed_face", 6),
        ("shared_faces", 7),
    ] {
        run(name, mode, false);
    }
}
#[test]
fn native_preloaded_maps_and_empty_wire_early_return() {
    run("preloaded_edge", 1, true);
    run("preloaded_empty", 4, true);
}
