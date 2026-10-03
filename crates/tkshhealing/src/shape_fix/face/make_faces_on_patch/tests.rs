use super::make_faces_on_patch;
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Builder, Edge, Face, Shape, ShapeIterator, Wire};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn one_loop_is_added_to_a_face_on_the_requested_support() {
    let surface = plane_surface();
    let support = make_support(&surface);
    let wire = square_wire();
    let mut loops = Sequence::default();
    loops.append(wire.into_shape());

    let faces = make_faces_on_patch(
        &surface,
        &mut loops,
        &support,
        &Location::default(),
        &Handle::<ShapeBuildReShape>::default(),
        false,
    )
    .unwrap();
    assert_eq!(faces.len(), 1);
    assert_eq!(count_wires(&faces[0]), 1);
}

#[test]
fn separate_loop_fallback_builds_one_face_for_each_root() {
    let surface = plane_surface();
    let support = make_support(&surface);
    let mut loops = Sequence::default();
    loops.append(square_wire().into_shape());
    loops.append(square_wire_shifted().into_shape());

    let faces = make_faces_on_patch(
        &surface,
        &mut loops,
        &support,
        &Location::default(),
        &Handle::<ShapeBuildReShape>::default(),
        false,
    )
    .unwrap();
    assert_eq!(faces.len(), 2);
    assert!(faces.iter().all(|face| count_wires(face) == 1));
}

#[test]
fn contained_pcurve_loop_is_assigned_as_a_hole() {
    let surface = plane_surface();
    let support = make_support(&surface);
    let mut loops = Sequence::default();
    loops.append(square_patch_wire(&surface, (0.0, 0.0, 5.0, 5.0)).into_shape());
    loops.append(square_patch_wire(&surface, (1.0, 1.0, 2.0, 2.0)).into_shape());

    let faces = make_faces_on_patch(
        &surface,
        &mut loops,
        &support,
        &Location::default(),
        &Handle::<ShapeBuildReShape>::default(),
        false,
    )
    .unwrap();
    assert_eq!(faces.len(), 1);
    assert_eq!(count_wires(&faces[0]), 2);
}

fn plane_surface() -> Handle<dyn GeomSurface> {
    let plane =
        GeomPlane::from_point(Point3::default(), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    Handle::new(plane).cast()
}

fn make_support(surface: &Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .make_face_on_surface(&mut face, surface, 1e-7)
        .unwrap();
    face
}

fn square_wire() -> Wire {
    square_wire_at(0.0)
}

fn square_wire_shifted() -> Wire {
    square_wire_at(2.0)
}

fn square_wire_at(offset: f64) -> Wire {
    let corners = [
        Point3::new(offset, 0.0, 0.0),
        Point3::new(offset + 1.0, 0.0, 0.0),
        Point3::new(offset + 1.0, 1.0, 0.0),
        Point3::new(offset, 1.0, 0.0),
    ];
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for edge in square_edges(&corners) {
        Builder.add(&mut wire, &edge.into_shape()).unwrap();
    }
    wire
}

fn square_edges(corners: &[Point3; 4]) -> [rustycad_tkbrep::topods::Edge; 4] {
    std::array::from_fn(|index| {
        BRepBuilderAPIMakeEdge::from_points(corners[index], corners[(index + 1) % corners.len()])
            .unwrap()
            .edge()
            .unwrap()
    })
}

fn square_patch_wire(
    surface: &Handle<dyn GeomSurface>,
    (u0, v0, u1, v1): (f64, f64, f64, f64),
) -> Wire {
    let corners = [
        Point3::new(u0, v0, 0.0),
        Point3::new(u1, v0, 0.0),
        Point3::new(u1, v1, 0.0),
        Point3::new(u0, v1, 0.0),
    ];
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    for index in 0..corners.len() {
        let edge = edge_with_pcurve(
            corners[index],
            corners[(index + 1) % corners.len()],
            surface,
        );
        Builder.add(&mut wire, &edge.into_shape()).unwrap();
    }
    wire
}

fn edge_with_pcurve(start: Point3, end: Point3, surface: &Handle<dyn GeomSurface>) -> Edge {
    let edge = BRepBuilderAPIMakeEdge::from_points(start, end)
        .unwrap()
        .edge()
        .unwrap();
    let delta = Point2::new(end.x() - start.x(), end.y() - start.y());
    let direction = Direction2::new(delta.x(), delta.y()).unwrap();
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(start.x(), start.y()),
        direction,
    ))
    .cast();
    BRepBuilder::default()
        .update_edge_pcurve(&edge, &pcurve, surface, &Location::default(), 1.0e-7, None)
        .unwrap();
    edge
}

fn count_wires(face: &Face) -> usize {
    ShapeIterator::new(&face.clone().into_shape(), false, false)
        .filter(|shape: &Shape| {
            shape.shape_type().ok() == Some(rustycad_tkg3d::top_abs::ShapeEnum::Wire)
        })
        .count()
}
