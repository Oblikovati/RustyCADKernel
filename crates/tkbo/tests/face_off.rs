//! Native face selection and shape-pair value semantics.
use rustycad_tkbo::{
    bop_tools::{self, CoupleOfShape},
    int_tools::Context,
};
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Edge, Face},
};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis, BRepPrimAPIMakeSphere,
};
#[test]
fn native_face_off_selection_and_ambiguity_match() {
    assert_eq!(
        include_str!("../../../tools/oracle/tkbo_face_off.expected")
            .lines()
            .count(),
        280
    );
    for line in include_str!("../../../tools/oracle/tkbo_face_off.expected").lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(observe(&row), row[4..9], "{line}");
    }
}
fn fixture(row: &[i32]) -> (Vec<Face>, Edge, Face) {
    let solid = match row[0] {
        1 => BRepPrimAPIMakeCylinder::new(2., 3.).solid().unwrap(),
        2 => BRepPrimAPIMakeSphere::new(2.).unwrap().solid().unwrap(),
        _ => BRepPrimAPIMakeBox::from_sizes(1., if row[0] == 3 { 1e-6 } else { 2. }, 3.)
            .unwrap()
            .solid()
            .unwrap(),
    };
    let faces: Vec<Face> = Explorer::new(&solid, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(Face::unchecked)
        .collect();
    let edge = Edge::unchecked(
        Explorer::new(&faces[0], ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .nth(row[1] as usize)
            .unwrap(),
    );
    let mut first = faces[0].clone();
    first.set_orientation(
        [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ][row[3] as usize],
    );
    (faces, edge, first)
}
fn observe(row: &[i32]) -> Vec<i32> {
    let (faces, edge, first) = fixture(row);
    let pairs = candidates(&edge, &first, &faces, row[2]);
    let mut selected = Face::default();
    let result = bop_tools::get_face_off(&edge, &first, &pairs, &mut selected, &Context::default());
    let index = faces
        .iter()
        .position(|face| face.is_same(&selected))
        .map_or(-1, |i| i as i32);
    vec![
        if result.is_ok() { 0 } else { -1 },
        i32::from(result.unwrap_or(false)),
        index,
        selected.orientation() as i32,
        pairs.extent(),
    ]
}
fn candidates(edge: &Edge, first: &Face, faces: &[Face], mode: i32) -> List<CoupleOfShape> {
    let mut other_edge = edge.clone();
    let other = faces[1..]
        .iter()
        .find(|face| bop_tools::get_edge_on_face(edge, face, &mut other_edge).unwrap())
        .unwrap_or(&faces[0]);
    let mut pairs = List::default();
    if mode != 0 {
        append_pair(&mut pairs, &other_edge, other);
    }
    if mode == 2 {
        append_pair(&mut pairs, edge, first);
    }
    if mode == 3 {
        append_pair(
            &mut pairs,
            &Edge::unchecked(edge.reversed()),
            &Face::unchecked(first.reversed()),
        );
    }
    if mode == 4 {
        append_pair(&mut pairs, &other_edge, other);
    }
    pairs
}
fn append_pair(pairs: &mut List<CoupleOfShape>, edge: &Edge, face: &Face) {
    let mut pair = CoupleOfShape::new();
    pair.set_shape1(edge);
    pair.set_shape2(face);
    pairs.append(pair);
}
#[test]
fn couple_preserves_occurrences_and_shared_topology() {
    let mut pair = CoupleOfShape::new();
    assert!(pair.shape1().is_null());
    assert!(pair.shape2().is_null());
    let solid = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
        .unwrap()
        .solid()
        .unwrap();
    pair.set_shape1(&solid);
    pair.set_shape2(&solid.reversed());
    assert!(pair.shape1().is_equal(&solid));
    assert!(pair.shape2().is_equal(&solid.reversed()));
    let copy = pair.clone();
    pair.set_shape1(&Default::default());
    assert!(copy.shape1().is_same(&solid));
    assert!(pair.shape1().is_null());
}

#[test]
fn native_internal_face_pair_classification_matches() {
    assert_eq!(
        include_str!("../../../tools/oracle/tkbo_face_off.expected")
            .lines()
            .count(),
        280
    );
    for line in include_str!("../../../tools/oracle/tkbo_face_off.expected").lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (faces, edge, first) = fixture(&row);
        let mut found = Edge::default();
        let last = if row[2] == 0 {
            &faces[0]
        } else {
            faces[1..]
                .iter()
                .find(|face| bop_tools::get_edge_on_face(&edge, face, &mut found).unwrap())
                .unwrap_or(&faces[0])
        };
        let result =
            bop_tools::internal_face_between(&first, &edge, &faces[0], last, &Context::default());
        assert_eq!(
            vec![if result.is_ok() { 0 } else { -1 }, result.unwrap_or(-99)],
            row[9..11],
            "{line}"
        );
    }
}

#[test]
fn native_internal_face_list_classification_matches() {
    let fixture = include_str!("../../../tools/oracle/tkbo_internal_face_list.expected");
    assert_eq!(fixture.lines().count(), 160);
    for line in fixture.lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let (faces, edge, first) = self::fixture(&row);
        let supports = classification_supports(&faces, &edge, row[2]);
        let result = bop_tools::internal_face_among(&first, &edge, &supports, &Context::default());
        assert_eq!(
            vec![
                if result.is_ok() { 0 } else { -1 },
                result.unwrap_or(-99),
                supports.extent()
            ],
            row[11..],
            "{line}"
        );
    }
}
fn classification_supports(
    faces: &[Face],
    edge: &Edge,
    mode: i32,
) -> List<rustycad_tkbrep::topods::Shape> {
    let mut occurrence = Edge::default();
    let other = faces[1..]
        .iter()
        .find(|face| bop_tools::get_edge_on_face(edge, face, &mut occurrence).unwrap())
        .unwrap_or(&faces[0]);
    let mut supports = List::default();
    if mode != 0 {
        supports.append(faces[0].clone().into_shape());
        supports.append(
            if mode == 3 {
                faces[0].clone()
            } else {
                other.clone()
            }
            .into_shape(),
        );
    }
    for _ in 0..if mode == 4 { 2 } else { i32::from(mode == 2) } {
        supports.append(faces[0].clone().into_shape());
        supports.append(other.clone().into_shape());
    }
    supports
}
