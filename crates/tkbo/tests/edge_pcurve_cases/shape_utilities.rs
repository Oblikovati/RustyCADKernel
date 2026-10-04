use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::{
    topods::{CompSolid, Compound, Face, Shape, Shell, Solid, Vertex, Wire},
    toptools::MapOfShape,
};
fn bare_shape(kind: usize) -> Shape {
    let builder = BRepBuilder::default();
    macro_rules! made {
        ($kind:ty,$method:ident) => {{
            let mut shape = <$kind>::default();
            builder.$method(&mut shape);
            shape.into_shape()
        }};
    }
    match kind {
        0 => made!(Compound, make_compound),
        1 => made!(CompSolid, make_comp_solid),
        2 => made!(Solid, make_solid),
        3 => made!(Shell, make_shell),
        4 => made!(Face, make_face),
        5 => made!(Wire, make_wire),
        6 => {
            let mut edge = Edge::default();
            builder.make_edge(&mut edge).unwrap();
            edge.into_shape()
        }
        _ => made!(Vertex, make_vertex),
    }
}
fn dimension_fixture(mode: usize) -> Shape {
    if mode < 8 {
        return bare_shape(mode);
    }
    let builder = BRepBuilder::default();
    let mut shape = bare_shape(0);
    let mut nested = bare_shape(0);
    let mut vertex = bare_shape(7);
    builder.add(&mut nested, &vertex).unwrap();
    vertex.set_orientation(Orientation::Reversed);
    builder.add(&mut nested, &vertex).unwrap();
    builder.add(&mut shape, &nested).unwrap();
    builder
        .add(&mut shape, &bare_shape(if mode == 8 { 7 } else { 6 }))
        .unwrap();
    shape
}
#[test]
fn native_compound_dimensions_and_fence_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_shape_utilities.expected").lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut shape = dimension_fixture(row[0] as usize);
        shape.set_orientation(orientations()[row[1] as usize]);
        let (mut minimum, mut maximum) = (99, 99);
        bop_tools::dimensions(&shape, &mut minimum, &mut maximum).unwrap();
        let mut leaves = Vec::new();
        let mut unique = Vec::new();
        let mut fence = MapOfShape::default();
        bop_tools::treat_compound(&shape, &mut leaves, None).unwrap();
        bop_tools::treat_compound(&shape, &mut unique, Some(&mut fence)).unwrap();
        let before = unique.len();
        bop_tools::treat_compound(&shape, &mut unique, Some(&mut fence)).unwrap();
        let mut actual = vec![
            minimum,
            maximum,
            bop_tools::dimension(&shape).unwrap(),
            leaves.len() as i32,
            before as i32,
            unique.len() as i32,
        ];
        for leaf in leaves {
            actual.extend([leaf.shape_type().unwrap() as i32, leaf.orientation() as i32]);
        }
        assert_eq!(actual, row[2..], "{line}");
    }
}
fn orientations() -> [Orientation; 4] {
    [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ]
}
fn shell_fixture(mode: usize) -> Shell {
    let mut face = fixture(0).face;
    let builder = BRepBuilder::default();
    let mut shell = bare_shape(3);
    if mode == 3 || mode == 4 {
        face.set_orientation(Orientation::Internal);
    }
    if mode != 0 {
        builder.add(&mut shell, &face).unwrap();
    }
    if mode == 2 || mode == 4 || mode == 6 {
        face.set_orientation(Orientation::Forward);
        builder.add(&mut shell, &face).unwrap();
    }
    if mode == 6 {
        builder.add(&mut shell, &face).unwrap();
    }
    if mode == 5 {
        degenerate_edges(&face);
    }
    if mode == 7 {
        builder.add(&mut shell, &fixture(0).face).unwrap();
    }
    Shell::unchecked(shell)
}
fn degenerate_edges(face: &Face) {
    use rustycad_tkg3d::top_abs::ShapeEnum;
    for shape in
        rustycad_tkbrep::topexp::Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape).unwrap()
    {
        BRepBuilder::default()
            .degenerated(&Edge::unchecked(shape), true)
            .unwrap();
    }
}
#[test]
fn native_open_shell_occurrence_count_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_open_shell.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut shell = shell_fixture(row[0]);
        shell.set_orientation(orientations()[row[1]]);
        assert_eq!(
            bop_tools::is_open_shell(&shell).unwrap(),
            row[2] != 0,
            "{line}"
        );
    }
}
