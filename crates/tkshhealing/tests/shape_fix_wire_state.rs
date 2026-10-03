//! Native working-analyzer ownership and state delegation.
#[path = "support/surfaces.rs"]
mod surfaces;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Builder, Face, Wire},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::{
    gp::{Axis3, Point3},
    toploc::Location,
};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysisSurface, ShapeAnalysisWire},
    shape_fix::{FixRoot, ShapeFixWire},
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

fn state_wire() -> Wire {
    let mut wire = Wire::default();
    BRepBuilder::default().make_wire(&mut wire);
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0., 0., 0.), Point3::new(1., 0., 0.))
            .unwrap()
            .edge()
            .unwrap();
    Builder.add(&mut wire, &edge).unwrap();
    wire
}

fn compare_row(name: &str, actual: &[f64]) {
    let expected = include_str!("../../../tools/oracle/tkshhealing_wire_state.expected");
    let row = expected
        .lines()
        .find(|row| row.split_whitespace().next() == Some(name))
        .unwrap();
    let values: Vec<f64> = row
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    assert_eq!(actual, values, "native state case {name}");
}

fn check_state(name: &str, fixer: &ShapeFixWire) {
    compare_row(
        name,
        &[
            fixer.precision(),
            fixer.working_analyzer().precision(),
            fixer.min_tolerance(),
            fixer.max_tolerance(),
            f64::from(fixer.nb_edges()),
            f64::from(fixer.is_loaded()),
            f64::from(fixer.is_ready()),
            f64::from(fixer.face().is_null()),
            f64::from(fixer.closed_mode()),
        ],
    );
}

#[test]
fn constructors_and_shared_analyzer_precision_match_native() {
    let fixer = ShapeFixWire::new();
    check_state("default", &fixer);
    let root: &dyn FixRoot = &fixer;
    root.set_precision(0.25);
    check_state("precision", &fixer);
    fixer.set_closed_mode(false);
    check_state("open", &fixer);
    let (wire, face) = (state_wire(), surfaces::planar_face());
    check_state(
        "constructor",
        &ShapeFixWire::from_wire_face(&wire, &face, 0.125).unwrap(),
    );
    fixer.init_wire_face(&wire, &face, 0.5).unwrap();
    check_state("init", &fixer);
    check_shared_precision(&fixer);
    check_surface_setters(&fixer, &face);
}

fn check_shared_precision(fixer: &ShapeFixWire) {
    let analyzer = Handle::new(ShapeAnalysisWire::new());
    analyzer.set_precision(0.75);
    fixer.init(&analyzer);
    assert!(fixer.working_analyzer() == analyzer);
    check_state("shared", fixer);
    fixer.set_precision(0.375);
    assert_eq!(analyzer.precision(), 0.375);
    check_state("shared_updated", fixer);
}

fn check_surface_setters(fixer: &ShapeFixWire, face: &Face) {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    fixer.set_surface(&surface).unwrap();
    check_state("surface", fixer);
    fixer
        .set_located_surface(&surface, &Location::default())
        .unwrap();
    check_state("located", fixer);
    let analysis = Handle::new(ShapeAnalysisSurface::new(&surface).unwrap());
    fixer.set_face_with_analysis(face, &analysis);
    compare_row(
        "face_analysis",
        &[
            f64::from(fixer.face().is_same(face)),
            f64::from(fixer.working_analyzer().surface() == analysis),
        ],
    );
    let replacement = Handle::new(ShapeAnalysisSurface::new(&surface).unwrap());
    fixer.set_surface_analysis(&replacement);
    compare_row(
        "surface_analysis",
        &[
            f64::from(fixer.face().is_same(face)),
            f64::from(fixer.working_analyzer().surface() == replacement),
        ],
    );
}
