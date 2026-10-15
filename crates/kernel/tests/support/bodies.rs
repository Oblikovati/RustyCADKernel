// SPDX-License-Identifier: GPL-2.0-only
//! The fixture bodies with their closed-form surfaces, areas and volumes.
use super::analytic::Surface;
use rustycad_tkbrep::topods::{Shape, Wire};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Point3, Vector3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakePrism, BRepPrimAPIMakeSphere,
    BRepPrimAPIMakeTorus,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};
use std::f64::consts::PI;

/// A closed fixture body and its independent closed-form description.
pub struct Fixture {
    pub name: &'static str,
    pub body: Shape,
    pub surfaces: Vec<Surface>,
    pub area: f64,
    pub volume: f64,
    /// The smallest radius of curvature of its curved faces (infinite when flat).
    pub min_radius: f64,
}

/// Box 2 x 3 x 4 at the origin.
pub fn box_fixture() -> Fixture {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(2.0, 3.0, 4.0).unwrap();
    let surfaces = [
        (0, 0.0, -1.0),
        (0, 2.0, 1.0),
        (1, 0.0, -1.0),
        (1, 3.0, 1.0),
        (2, 0.0, -1.0),
        (2, 4.0, 1.0),
    ]
    .map(|(axis, offset, sign)| Surface::axis_plane(axis, offset, sign))
    .to_vec();
    let body = maker.shape().unwrap().clone();
    Fixture {
        name: "box",
        body,
        surfaces,
        area: 52.0,
        volume: 24.0,
        min_radius: f64::INFINITY,
    }
}

/// Cylinder of radius 1 and height 2 on the Z axis.
pub fn cylinder_fixture() -> Fixture {
    let mut maker = BRepPrimAPIMakeCylinder::new(1.0, 2.0);
    let surfaces = vec![
        Surface::cylinder(1.0, 1.0),
        Surface::axis_plane(2, 0.0, -1.0),
        Surface::axis_plane(2, 2.0, 1.0),
    ];
    let body = maker.shape().unwrap().clone();
    Fixture {
        name: "cylinder",
        body,
        surfaces,
        area: 6.0 * PI,
        volume: 2.0 * PI,
        min_radius: 1.0,
    }
}

/// Sphere of radius 1.5 at the origin.
pub fn sphere_fixture() -> Fixture {
    let mut maker = BRepPrimAPIMakeSphere::new(1.5).unwrap();
    let body = maker.shape().unwrap().clone();
    let surfaces = vec![Surface::Sphere { radius: 1.5 }];
    let (area, volume) = (4.0 * PI * 2.25, 4.0 / 3.0 * PI * 3.375);
    Fixture {
        name: "sphere",
        body,
        surfaces,
        area,
        volume,
        min_radius: 1.5,
    }
}

/// Torus of major radius 3 and minor radius 1 around the Z axis.
pub fn torus_fixture() -> Fixture {
    let mut maker = BRepPrimAPIMakeTorus::new(3.0, 1.0).unwrap();
    let body = maker.shape().unwrap().clone();
    let surfaces = vec![Surface::Torus {
        major: 3.0,
        minor: 1.0,
    }];
    let (area, volume) = (4.0 * PI * PI * 3.0, 2.0 * PI * PI * 3.0);
    Fixture {
        name: "torus",
        body,
        surfaces,
        area,
        volume,
        min_radius: 1.0,
    }
}

/// Annulus between radii 1 and 2 in the XY plane, extruded 1.5 along Z.
pub fn annulus_prism_fixture() -> Fixture {
    let height = Vector3::new(0.0, 0.0, 1.5);
    let mut prism = BRepPrimAPIMakePrism::new(&annulus_profile(), height, false, true).unwrap();
    let surfaces = vec![
        Surface::cylinder(2.0, 1.0),
        Surface::cylinder(1.0, -1.0),
        Surface::axis_plane(2, 0.0, -1.0),
        Surface::axis_plane(2, 1.5, 1.0),
    ];
    let body = prism.shape().unwrap().clone();
    let (area, volume) = (15.0 * PI, 4.5 * PI);
    let name = "annulus prism";
    Fixture {
        name,
        body,
        surfaces,
        area,
        volume,
        min_radius: 1.0,
    }
}

fn annulus_profile() -> Shape {
    let inner = rustycad_tkbrep::topods::wire(&circle_wire(1.0).reversed()).unwrap();
    let mut face = BRepBuilderAPIMakeFace::from_wire(&circle_wire(2.0), true).unwrap();
    face.add(&inner).unwrap();
    face.shape().unwrap().clone()
}

fn circle_wire(radius: f64) -> Wire {
    let frame = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let circle = Circle3::new(frame, radius).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
        .unwrap()
        .edge()
        .unwrap();
    BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap()
}

/// All five acceptance fixtures.
pub fn all_fixtures() -> Vec<Fixture> {
    vec![
        box_fixture(),
        cylinder_fixture(),
        sphere_fixture(),
        torus_fixture(),
        annulus_prism_fixture(),
    ]
}
