use super::*;
use rustycad_tkbo::bop_tools::make_connexity_block;
use rustycad_tkbrep::{
    topexp::{self, Explorer},
    topods::Shape,
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::ShapeEnum;
#[test]
fn native_first_face_component_and_append_semantics_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_face_connexity.expected").lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(
            observe(row[0] as usize, row[1] as usize, row[2] != 0),
            row[3..],
            "{line}"
        );
    }
}
fn observe(mode: usize, seed: usize, existing: bool) -> Vec<i32> {
    let (faces, input, avoided) = fixture(mode, seed);
    let mut output = List::default();
    if existing {
        output.append(Shape::default());
    }
    let result = make_connexity_block(&input, &avoided, &mut output);
    let status = match result {
        Ok(()) => 0,
        Err(error) => {
            assert_eq!(error.exception_type(), "Standard_NoSuchObject");
            -1
        }
    };
    let mut actual = vec![
        status,
        input.extent(),
        avoided.extent() as i32,
        output.extent(),
    ];
    for face in output.iter() {
        let index = faces
            .iter()
            .position(|candidate| face.is_same(candidate))
            .map_or(-1, |index| index as i32);
        actual.extend([index, face.orientation() as i32]);
    }
    actual
}
fn fixture(mode: usize, seed: usize) -> (Vec<Shape>, List<Shape>, IndexedMapOfShape) {
    let solid = solid_fixture(0);
    let mut faces: Vec<Shape> = Explorer::new(&solid, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .collect();
    if mode == 3 {
        let second = BRepPrimAPIMakeBox::from_sizes(3., 4., 5.)
            .unwrap()
            .solid()
            .unwrap();
        faces.extend(Explorer::new(&second, ShapeEnum::Face, ShapeEnum::Shape).unwrap());
    }
    let mut input = List::default();
    if mode != 4 {
        for index in 0..faces.len() {
            input.append(faces[(seed + index) % faces.len()].clone());
        }
    }
    let mut avoided = IndexedMapOfShape::default();
    if mode == 1 {
        topexp::map_shapes_of_type(&solid, ShapeEnum::Edge, &mut avoided).unwrap();
    }
    if mode == 2 {
        topexp::map_shapes_of_type(&faces[seed], ShapeEnum::Edge, &mut avoided).unwrap();
    }
    (faces, input, avoided)
}
