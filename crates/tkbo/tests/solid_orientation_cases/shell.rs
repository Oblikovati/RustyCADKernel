use super::*;
use rustycad_tkbo::bop_tools::orient_faces_on_shell;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Face, Shape, ShapeIterator, Shell},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
#[test]
fn native_shell_orientation_graph_cases_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_orient_shell.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (mut shell, faces) = shell_fixture(row[0], row[1]);
        shell.set_orientation(
            [
                Orientation::Forward,
                Orientation::Reversed,
                Orientation::Internal,
                Orientation::External,
            ][row[2]],
        );
        let original = shell.clone();
        orient_faces_on_shell(&mut shell).unwrap();
        assert_eq!(observe(&shell, &original, &faces), row[3..], "{line}");
    }
}
fn observe(shell: &Shape, original: &Shape, faces: &[Shape]) -> Vec<usize> {
    let children: Vec<Shape> = ShapeIterator::new(shell, false, true).collect();
    let mut actual = vec![
        usize::from(shell.is_same(original)),
        shell.orientation() as usize,
        children.len(),
    ];
    for child in children {
        let index = faces.iter().position(|face| child.is_same(face)).unwrap();
        actual.extend([index, child.orientation() as usize]);
    }
    actual
}
fn shell_fixture(mode: usize, mask: usize) -> (Shape, Vec<Shape>) {
    let faces = source_faces(mode);
    let mut shell = Shell::default();
    let builder = BRepBuilder::default();
    builder.make_shell(&mut shell);
    for (index, source) in faces.iter().enumerate() {
        let mut face = source.clone();
        if mask & (1 << (index % 3)) != 0 {
            face.reverse();
        }
        builder.add(&mut shell, &face).unwrap();
        if mode == 5 {
            builder.add(&mut shell, &face.reversed()).unwrap();
        }
    }
    (shell.into_shape(), faces)
}
fn source_faces(mode: usize) -> Vec<Shape> {
    if mode == 4 {
        return Vec::new();
    }
    if mode == 2 {
        let mut face = Face::default();
        BRepBuilder::default().make_face(&mut face);
        return vec![face.into_shape()];
    }
    let kind = match mode {
        1 => 1,
        3 => 2,
        _ => 0,
    };
    Explorer::new(&solid_fixture(kind), ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .collect()
}
