//! Rust-only replay of actual StdPrs_ShadedShape float vertices/normals/UVs and indices.
#[path = "support/textured_face_fixture.rs"]
mod fixture;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point2;
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
use rustycad_tkv3d::std_prs::StdPrsShadedShape;

#[test]
fn textured_triangles_match_all_native_orientation_mirror_surface_normal_cases() {
    for line in include_str!("fixtures/ais_textured_shape/native-triangles.txt").lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        check_native_textured_case(&fields);
    }
}
fn compare_native_textured_array(
    array: &Handle<Graphic3dArrayOfTriangles>,
    fields: &[&str],
    scenario: i32,
) {
    assert_eq!(array.vertex_number(), fields[0].parse::<i32>().unwrap());
    assert_eq!(array.edge_number(), fields[1].parse::<i32>().unwrap());
    let actual = textured_array_components(array);
    let expected: Vec<f64> = fields[2..]
        .iter()
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(actual, expected, "scenario {scenario}");
}
fn textured_array_components(array: &Graphic3dArrayOfTriangles) -> Vec<f64> {
    let mut components = Vec::new();
    for index in 1..=array.vertex_number() {
        components.extend(array.vertice_xyz(index).unwrap());
        components.extend(array.vertex_normal_xyz(index).unwrap());
        components.extend(array.vertex_texel_xy(index).unwrap());
    }
    components
        .extend((1..=array.edge_number()).map(|index| f64::from(array.edge(index).unwrap() - 1)));
    components
}
#[test]
fn untextured_filler_keeps_native_geometry_and_omits_only_uv_attributes() {
    let face = fixture::textured_face_fixture(3).unwrap();
    let array = StdPrsShadedShape::fill_triangles(&face).unwrap();
    assert!(!array.has_vertex_texels());
    assert_eq!((array.vertex_number(), array.edge_number()), (4, 6));
    assert_eq!(array.vertex_normal_xyz(1).unwrap(), [0., 0., -1.]);
    assert!(
        StdPrsShadedShape::fill_triangles(&Shape::default())
            .unwrap()
            .is_null()
    );
}

fn check_native_textured_case(fields: &[&str]) {
    let scenario: i32 = fields[1].parse().unwrap();
    let array = textured_native_case_array(scenario);
    assert_eq!(array.is_null(), fields[2] == "1", "scenario {scenario}");
    if array.is_null() {
        return;
    }
    compare_native_textured_array(&array, &fields[3..], scenario);
}

fn textured_native_case_array(scenario: i32) -> Handle<Graphic3dArrayOfTriangles> {
    let shape = if scenario == 16 {
        Shape::default()
    } else {
        fixture::textured_face_fixture(scenario).unwrap().into()
    };
    StdPrsShadedShape::fill_triangles_with_texels(
        &shape,
        true,
        Point2::new(0.25, -0.5),
        Point2::new(2., 3.),
        Point2::new(0.5, 2.),
    )
    .unwrap()
}
