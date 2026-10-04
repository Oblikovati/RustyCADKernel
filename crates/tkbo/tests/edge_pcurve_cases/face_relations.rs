use super::*;
use rustycad_tkbo::bop_tools;
use rustycad_tkbrep::{brep::BRepTool, topexp::Explorer, topods::Face};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Location,
};
fn first_edge(face: &Face) -> Edge {
    Edge::unchecked(
        Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .current()
            .clone(),
    )
}
#[test]
fn native_edge_occurrence_lookup_and_retained_outputs_match() {
    let orientations = [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ];
    for line in include_str!("../../../../tools/oracle/tkbo_edge_on_face.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut face = fixture(row[0]).face;
        let source = if row[1] == 0 {
            face.clone()
        } else {
            fixture(row[0]).face
        };
        let mut edge = first_edge(&source);
        edge.set_orientation(orientations[row[2]]);
        face.set_orientation(orientations[row[3]]);
        let mut initial = Edge::default();
        BRepBuilder::default().make_edge(&mut initial).unwrap();
        let (mut off, mut on) = (initial.clone(), initial.clone());
        let off_found = bop_tools::get_edge_off(&edge, &face, &mut off).unwrap();
        let on_found = bop_tools::get_edge_on_face(&edge, &face, &mut on).unwrap();
        assert_eq!(
            [
                usize::from(off_found),
                off.orientation() as usize,
                usize::from(off.is_same(&initial)),
                usize::from(on_found),
                on.orientation() as usize,
                usize::from(on.is_same(&initial))
            ],
            row[4..],
            "{line}"
        );
    }
}
fn domain_faces(kind: usize, mode: usize, tolerance: f64) -> (Face, Face) {
    let mut first = fixture(kind).face;
    let mut second = if mode == 0 {
        first.clone()
    } else {
        fixture(kind).face
    };
    if mode == 2 || mode == 3 {
        let mut transform = Transform3::default();
        transform.set_translation(if mode == 2 {
            Vector3::new(0., 0., 1e-5)
        } else {
            Vector3::new(100., 0., 0.)
        });
        second.move_by(&Location::new(transform), true).unwrap();
    }
    let builder = BRepBuilder::default();
    for shape in Explorer::new(&first, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        builder
            .update_edge_tolerance(&Edge::unchecked(shape), tolerance)
            .unwrap();
    }
    if mode == 4 {
        let mut empty = Face::default();
        builder
            .make_face_on_surface(&mut empty, &BRepTool::surface(&first).unwrap(), 1e-7)
            .unwrap();
        first = empty;
    }
    (first, second)
}
#[test]
fn native_same_domain_point_tolerance_criterion_matches() {
    for line in include_str!("../../../../tools/oracle/tkbo_face_domain.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (first, second) = domain_faces(row[0], row[1], if row[3] == 0 { 1e-7 } else { 1e-3 });
        let fuzz = [0., 1e-6, 1e-4][row[2]];
        assert_eq!(
            bop_tools::are_faces_same_domain(&first, &second, &Context::default(), fuzz).unwrap(),
            row[4] != 0,
            "{line}"
        );
    }
}
