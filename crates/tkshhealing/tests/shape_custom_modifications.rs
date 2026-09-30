//! Behavior tests of the ShapeCustom modifications (TrsfModification, DirectModification,
//! ConvertToRevolution and the message registrator of ShapeCustom_Modification).
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsModification, Modifier, NewCurve, NewSurface};
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::message::{Gravity, Msg};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomSurface,
    GeomSurfaceOfRevolution,
};
use rustycad_tkmath::gp::{
    Axis1, Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Transform3,
};
use rustycad_tkshhealing::shape_custom::{
    ShapeCustomConvertToRevolution, ShapeCustomDirectModification, ShapeCustomModification,
    ShapeCustomTrsfModification,
};
use rustycad_tkshhealing::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};

fn frame(indirect: bool) -> Axis3 {
    let z = Direction3::new(0.0, 0.0, 1.0).unwrap();
    let x = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let mut frame = Axis3::new(Point3::new(0.0, 0.0, 0.0), z, x).unwrap();
    if indirect {
        frame.y_reverse();
    }
    frame
}

fn cylinder(indirect: bool) -> Handle<dyn GeomSurface> {
    Handle::new(GeomCylindricalSurface::new(frame(indirect), 2.0).unwrap()).cast()
}

/// A face on the surface with one edge: a 3D line and a p-curve line on the face.
fn face_with_edge(surface: &Handle<dyn GeomSurface>, tolerance: f64) -> (Face, Edge) {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    builder
        .make_face_on_surface(&mut face, surface, tolerance)
        .unwrap();
    let axis = Axis1::new(
        Point3::new(2.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    );
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
    let mut edge = Edge::default();
    builder
        .make_edge_with_curve(&mut edge, &line, tolerance)
        .unwrap();
    let axis2d = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(0.0, 1.0).unwrap());
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(axis2d)).cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, &face, tolerance)
        .unwrap();
    (face, edge)
}

#[test]
fn direct_modification_reverses_indirect_surfaces_only() {
    let modification = ShapeCustomDirectModification::new();
    let (direct, direct_edge) = face_with_edge(&cylinder(false), 1e-7);
    let mut result = NewSurface::default();
    assert!(!modification.new_surface(&direct, &mut result).unwrap());
    assert!(
        !modification
            .new_curve(&direct_edge, &mut NewCurve::default())
            .unwrap()
    );
    let (indirect, edge) = face_with_edge(&cylinder(true), 1e-7);
    let mut result = NewSurface::default();
    assert!(modification.new_surface(&indirect, &mut result).unwrap());
    assert!(result.rev_face && result.rev_wires);
    let reversed = result.surface.get().unwrap();
    let original = cylinder(true);
    let p = original.get().unwrap().value(0.3, 0.5).unwrap();
    let q = reversed
        .value(reversed.u_reversed_parameter(0.3).unwrap(), 0.5)
        .unwrap();
    assert!(p.distance(&q) < 1e-12);
    let mut curve = NewCurve::default();
    assert!(modification.new_curve(&edge, &mut curve).unwrap());
    assert!(!curve.curve.is_null());
}

#[test]
fn direct_modification_flips_negative_cones_in_both_directions() {
    let cone: Handle<dyn GeomSurface> =
        Handle::new(GeomConicalSurface::new(frame(false), -0.3, 1.0).unwrap()).cast();
    let (face, edge) = face_with_edge(&cone, 1e-7);
    let modification = ShapeCustomDirectModification::new();
    let mut result = NewSurface::default();
    assert!(modification.new_surface(&face, &mut result).unwrap());
    assert!(!result.rev_face && !result.rev_wires);
    let (mut pcurve, mut tolerance) = (Handle::default(), 0.0);
    assert!(
        modification
            .new_curve2d(&edge, &face, &edge, &face, &mut pcurve, &mut tolerance)
            .unwrap()
    );
    let mirrored = pcurve.get().unwrap().value(0.5).unwrap();
    assert_eq!((mirrored.x(), mirrored.y()), (-0.0, -0.5));
}

#[test]
fn convert_to_revolution_makes_a_surface_of_revolution() {
    let (face, edge) = face_with_edge(&cylinder(false), 1e-7);
    let modification = ShapeCustomConvertToRevolution::new();
    let mut result = NewSurface::default();
    assert!(modification.new_surface(&face, &mut result).unwrap());
    assert!(!result.surface.cast::<GeomSurfaceOfRevolution>().is_null());
    let p = cylinder(false).get().unwrap().value(0.7, 0.4).unwrap();
    let q = result.surface.get().unwrap().value(0.7, 0.4).unwrap();
    assert!(p.distance(&q) < 1e-12);
    let modifier = Modifier::with_modification(&face, &Handle::new(modification).cast()).unwrap();
    assert!(modifier.is_done());
    let rebuilt = modifier.modified_shape(&face).unwrap();
    assert!(!rebuilt.is_same(&face));
    let _ = edge;
}

#[test]
fn trsf_modification_scales_the_stored_tolerances() {
    let (face, edge) = face_with_edge(&cylinder(false), 1e-9);
    let mut trsf = Transform3::default();
    trsf.set_scale(Point3::new(0.0, 0.0, 0.0), -2.0).unwrap();
    let modification = ShapeCustomTrsfModification::new(trsf);
    let mut surface = NewSurface::default();
    assert!(modification.new_surface(&face, &mut surface).unwrap());
    assert_eq!(surface.tolerance, 2e-9);
    assert!(BRepTool::face_tolerance(&face).unwrap() > 1e-9);
    let mut curve = NewCurve::default();
    assert!(modification.new_curve(&edge, &mut curve).unwrap());
    assert_eq!(curve.tolerance, 2e-9);
    let (mut p, mut tolerance) = (0.0, 0.0);
    assert!(
        modification
            .new_parameter(&Vertex::default(), &edge, &mut p, &mut tolerance)
            .is_err()
    );
}

#[test]
fn modifications_send_their_messages_to_the_registrator() {
    let (face, _) = face_with_edge(&cylinder(true), 1e-7);
    let modification = ShapeCustomDirectModification::new();
    modification.send_msg(&face, &Msg::new(), Gravity::Info);
    let registrator = Handle::new(ShapeExtendMsgRegistrator::new());
    modification.set_msg_registrator(&registrator.clone().cast::<dyn BasicMsgRegistrator>());
    assert!(!modification.msg_registrator().is_null());
    modification
        .new_surface(&face, &mut NewSurface::default())
        .unwrap();
    assert!(registrator.map_shape(|map| map.seek(&face).is_some_and(|list| list.extent() == 1)));
}
