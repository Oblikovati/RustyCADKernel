use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{self, Compound, Shape},
    toptools::MapOfShape,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkoffset::brep_offset::BRepOffsetAnalyse;
pub fn replay(fields: &[&str], shape: &Shape, analysis: &BRepOffsetAnalyse) {
    let kinds = super::concavity_types();
    let first = kinds[super::number(fields[2])];
    let second = fields[3].parse::<i32>().unwrap();
    let mut groups = vec![Shape::default()];
    if second < 0 {
        analysis.explode(first, &mut groups).unwrap();
    } else {
        analysis
            .explode_two_types(first, kinds[second as usize], &mut groups)
            .unwrap();
    }
    assert_eq!(groups.len(), super::number(fields[4]));
    let faces: Vec<Shape> = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .collect();
    let mut expected = 5;
    for group in &groups {
        let members: Vec<Shape> = Explorer::new(group, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .collect();
        assert_eq!(members.len(), super::number(fields[expected]));
        expected += 1;
        for member in members {
            assert!(member.is_same(&faces[super::number(fields[expected])]));
            expected += 1;
        }
    }
    if let Some(first_group) = groups.first() {
        check_direct_add(&faces, analysis, fields, first_group);
    }
}
fn check_direct_add(
    faces: &[Shape],
    analysis: &BRepOffsetAnalyse,
    fields: &[&str],
    expected: &Shape,
) {
    let kinds = super::concavity_types();
    let first = kinds[super::number(fields[2])];
    let second = fields[3].parse::<i32>().unwrap();
    let face = topods::face(&faces[0]).unwrap();
    let mut compound = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut compound);
    builder.add(&mut compound, &face).unwrap();
    let mut visited = MapOfShape::default();
    visited.add(face.clone().into_shape());
    if second < 0 {
        analysis
            .add_faces(&face, &mut compound, &mut visited, first)
            .unwrap();
    } else {
        analysis
            .add_faces_two_types(
                &face,
                &mut compound,
                &mut visited,
                first,
                kinds[second as usize],
            )
            .unwrap();
    }
    let actual: Vec<Shape> = Explorer::new(&compound, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .collect();
    let expected: Vec<Shape> = Explorer::new(expected, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .collect();
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(&expected) {
        assert!(a.is_same(b));
    }
}
