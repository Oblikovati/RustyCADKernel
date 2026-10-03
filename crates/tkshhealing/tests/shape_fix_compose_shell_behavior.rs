//! Native constructor and public mutable-option parity for ShapeFix_ComposeShell.
use rustycad_tkernel::{ncollection::HArray2, standard::Handle};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisTransferParameters,
    shape_extend::{ShapeExtendCompositeSurface, ShapeExtendParametrisation, ShapeExtendStatus},
    shape_fix::ShapeFixComposeShell,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeWire,
};

#[test]
fn constructor_modes_and_transfer_tool_match_native() {
    let expected = include_str!("../../../tools/oracle/tkshhealing_compose_shell.expected");
    let composer = ShapeFixComposeShell::new();
    let tool = composer.get_transfer_param_tool();
    assert_eq!(
        format!(
            "default {} {} {} {}",
            composer.closed_mode() as u8,
            composer.status(ShapeExtendStatus::Done1) as u8,
            composer.result().is_null() as u8,
            tool.dynamic_type().unwrap().name()
        ),
        expected.lines().next().unwrap()
    );
    composer.set_closed_mode(true);
    assert_eq!(
        format!("closed_mode {}", composer.closed_mode() as u8),
        expected.lines().nth(1).unwrap()
    );
    let replacement = Handle::new(ShapeAnalysisTransferParameters::new()).cast();
    composer.set_transfer_param_tool(&replacement);
    assert_eq!(
        format!(
            "transfer_type {}",
            composer
                .get_transfer_param_tool()
                .dynamic_type()
                .unwrap()
                .name()
        ),
        expected.lines().nth(2).unwrap()
    );
    assert_init_matches_native(expected);
}

fn assert_init_matches_native(expected: &str) {
    let face = face_fixture();
    let grid = plane_grid();
    let composer = ShapeFixComposeShell::new();
    composer
        .init(&grid, &Location::default(), &face, 1.0e-7)
        .unwrap();
    assert_eq!(
        format!(
            "init {} {} {}",
            composer.closed_mode() as u8,
            composer.status(ShapeExtendStatus::Done1) as u8,
            composer.result().is_null() as u8,
        ),
        expected.lines().nth(3).unwrap()
    );
}

fn plane_grid() -> Handle<ShapeExtendCompositeSurface> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let cells = HArray2::with_value(1, 1, 1, 1, Handle::<dyn GeomSurface>::default()).unwrap();
    cells.change_array2().set_value(1, 1, plane).unwrap();
    let patches = Handle::new(cells);
    Handle::new(
        ShapeExtendCompositeSurface::with_grid(&patches, ShapeExtendParametrisation::Natural)
            .unwrap(),
    )
}

fn face_fixture() -> rustycad_tkbrep::topods::Face {
    let points = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(2.0, 0.0, 0.0),
        Point3::new(2.0, 2.0, 0.0),
        Point3::new(0.0, 2.0, 0.0),
    ];
    let mut builder = BRepBuilderAPIMakeWire::new();
    for index in 0..points.len() {
        let edge = BRepBuilderAPIMakeEdge::from_points(points[index], points[(index + 1) % 4])
            .unwrap()
            .edge()
            .unwrap();
        builder.add_edge(&edge).unwrap();
    }
    BRepBuilderAPIMakeFace::from_plane_wire(
        rustycad_tkmath::gp::Plane3::default(),
        &builder.wire().unwrap(),
        true,
    )
    .unwrap()
    .face()
    .unwrap()
}
