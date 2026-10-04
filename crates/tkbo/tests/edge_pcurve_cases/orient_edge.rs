use super::super::fixture;
use rustycad_tkbo::bop_tools::algo_tools3d;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{self, Edge},
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
#[test]
fn native_seam_and_face_occurrences_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_orient_edge.expected").lines() {
        let row: Vec<usize> = line
            .split_whitespace()
            .map(|token| token.parse().unwrap())
            .collect();
        let orientations = [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ];
        let mut face = fixture(row[0]).face;
        face.set_orientation(orientations[row[1]]);
        let occurrence = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)
            .unwrap()
            .nth(row[2])
            .unwrap();
        let mut edge = topods::edge(&occurrence).unwrap();
        edge.set_orientation(orientations[row[3]]);
        let mut result = Edge::default();
        algo_tools3d::orient_edge_on_face(&edge, &face, &mut result).unwrap();
        assert_eq!(result.orientation(), orientations[row[4]], "{line}");
        assert_eq!(usize::from(result.is_same(&edge)), row[5], "{line}");
    }
}
