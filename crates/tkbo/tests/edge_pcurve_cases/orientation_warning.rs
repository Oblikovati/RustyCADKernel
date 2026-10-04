use super::*;
use rustycad_tkbo::{bop_algo::AlertUnableToOrientTheShape, bop_tools};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Compound, Face, Shape, ShapeIterator},
};
use rustycad_tkernel::message::{Gravity, Report, downcast_alert};
#[test]
fn native_orientation_warning_payload_and_repeat_match() {
    for line in include_str!("../../../../tools/oracle/tkbo_orientation_warning.expected").lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let (split, original) = warning_shapes(fields[0].parse().unwrap());
        let report = Report::default();
        let present = fields[1] == "1";
        let context = Context::default();
        let mut reverse = false;
        for _ in 0..2 {
            reverse = bop_tools::is_split_to_reverse_with_warning(
                &split,
                &original,
                &context,
                if present { Some(&report) } else { None },
            )
            .unwrap();
        }
        assert_eq!(reverse, fields[2] == "1");
        let alerts = report.get_alerts(Gravity::Warning);
        assert_eq!(alerts.len(), fields[3].parse::<usize>().unwrap());
        for (index, alert) in alerts.iter().enumerate() {
            assert_eq!(
                alert.get_message_key().as_bytes(),
                fields[5 + 2 * index].as_bytes()
            );
            let alert = downcast_alert::<AlertUnableToOrientTheShape>(alert).unwrap();
            let children: Vec<Shape> = ShapeIterator::new(&alert.get_shape(), true, true).collect();
            assert_eq!(
                children.len(),
                fields[4 + 2 * index].parse::<usize>().unwrap()
            );
            assert!(children[0].is_same(&split));
            assert!(children[1].is_same(&original));
        }
    }
}
fn warning_shapes(mode: usize) -> (Shape, Shape) {
    let face = fixture(0).face;
    if mode == 1 {
        let mut compound = Compound::default();
        BRepBuilder::default().make_compound(&mut compound);
        return (compound.clone().into_shape(), compound.into_shape());
    }
    if mode == 2 {
        let mut empty = Face::default();
        BRepBuilder::default()
            .make_face_on_surface(&mut empty, &BRepTool::surface(&face).unwrap(), 1e-7)
            .unwrap();
        return (empty.into_shape(), fixture(0).face.into_shape());
    }
    (face.clone().into_shape(), face.into_shape())
}
