//! Native closure orchestration and status lifetime.
#[path = "support/surfaces.rs"]
mod surfaces;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Builder, Wire},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::{shape_extend::ShapeExtendStatus, shape_fix::ShapeFixWire};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn closure_wire(gap: f64) -> Wire {
    let mut wire = Wire::default();
    BRepBuilder::default().make_wire(&mut wire);
    let origin = Point3::new(0., 0., 0.);
    let turn = Point3::new(1., 0., 0.);
    for (start, end) in [(origin, turn), (turn, Point3::new(0., gap, 0.))] {
        let edge = BRepBuilderAPIMakeEdge::from_points(start, end)
            .unwrap()
            .edge()
            .unwrap();
        Builder.add(&mut wire, &edge).unwrap();
    }
    wire
}

fn record(name: &str, fixer: &ShapeFixWire) -> String {
    let changed = fixer.fix_closed(1e-3).unwrap();
    let flags = [
        ShapeExtendStatus::Done1,
        ShapeExtendStatus::Done2,
        ShapeExtendStatus::Done3,
        ShapeExtendStatus::Fail1,
        ShapeExtendStatus::Fail2,
        ShapeExtendStatus::Fail3,
    ];
    let mut row = format!("{name} {}", changed as u8);
    for flag in flags {
        row.push_str(&format!(" {}", fixer.status_closed(flag) as u8));
    }
    row.push_str(&format!(
        " {} {} {}",
        fixer.last_fix_status(ShapeExtendStatus::Done) as u8,
        fixer.last_fix_status(ShapeExtendStatus::Fail) as u8,
        fixer
            .wire_data()
            .unwrap()
            .get()
            .map_or(0, |wire| wire.nb_edges())
    ));
    row
}

fn fixture_rows(mode: i32) -> Vec<String> {
    let gap = if mode == 0 {
        0.
    } else if mode == 1 {
        1e-4
    } else {
        0.1
    };
    let fixer = ShapeFixWire::new();
    fixer.load(&closure_wire(gap)).unwrap();
    if mode == 3 {
        fixer.set_face(&surfaces::planar_face()).unwrap();
    }
    let name = ["coincident", "near", "far", "face_far"][mode as usize];
    let mut rows = vec![record(name, &fixer), record("repeat", &fixer)];
    fixer.load(&closure_wire(0.)).unwrap();
    rows.push(format!(
        "load_reset {} {}",
        fixer.status_closed(ShapeExtendStatus::Done) as u8,
        fixer.status_closed(ShapeExtendStatus::Fail) as u8
    ));
    rows
}

#[test]
fn closure_repair_sequence_and_statuses_match_native() {
    let mut rows = vec![record("unloaded", &ShapeFixWire::new())];
    for mode in 0..4 {
        rows.extend(fixture_rows(mode));
    }
    assert_eq!(
        rows.join("\n"),
        include_str!("../../../tools/oracle/tkshhealing_wire_closed.expected").trim()
    );
}
