use super::*;
use rustycad_tkbo::bop_tools;
#[test]
fn native_face_orientation_comparisons_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_face_orientation.expected").lines() {
        let row: Vec<i32> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        let mut split = fixture(row[0] as usize).face;
        let mut original = if row[1] == 0 {
            split.clone()
        } else {
            fixture(row[0] as usize).face
        };
        if row[1] == 2 {
            let mut empty = rustycad_tkbrep::topods::Face::default();
            BRepBuilder::default()
                .make_face_on_surface(
                    &mut empty,
                    &rustycad_tkbrep::brep::BRepTool::surface(&split).unwrap(),
                    1e-7,
                )
                .unwrap();
            split = empty;
        }
        let orientations = [
            Orientation::Forward,
            Orientation::Reversed,
            Orientation::Internal,
            Orientation::External,
        ];
        split.set_orientation(orientations[row[2] as usize]);
        original.set_orientation(orientations[row[3] as usize]);
        let context = Context::default();
        let mut error = 99;
        let mut shape_error = 99;
        let reverse =
            bop_tools::is_split_face_to_reverse(&split, &original, &context, &mut error).unwrap();
        let shape_reverse =
            bop_tools::is_split_shape_to_reverse(&split, &original, &context, &mut shape_error)
                .unwrap();
        let sense = bop_tools::face_sense(&split, &original, &context).unwrap();
        assert_eq!(
            [
                i32::from(reverse),
                error,
                i32::from(shape_reverse),
                shape_error,
                sense
            ],
            row[4..],
            "{line}"
        );
    }
}
#[test]
fn unsupported_split_shape_reports_native_100() {
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    let mut error = 99;
    assert!(
        !bop_tools::is_split_shape_to_reverse(
            &compound,
            &compound,
            &Context::default(),
            &mut error
        )
        .unwrap()
    );
    assert_eq!(error, 100);
}
