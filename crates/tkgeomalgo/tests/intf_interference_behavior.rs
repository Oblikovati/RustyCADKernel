// Copyright (c) 1991-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Behavior of Intf_InterferencePolygon2d and of the generic polygon-polyhedron
//! interference over fake polygon and polyhedron tools; the bits are compared with
//! OCCT by the numeric probe.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeomalgo::intf::{
    InterferencePolygon2d, InterferencePolygonPolyhedron, PIType, Polygon2d, Polygon3dTool,
    PolyhedronTool,
};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{Direction3, Line3, Point2, Point3};

/// A fake polyline over given points, with no deflection.
struct FakePolyline {
    points: Vec<Point2>,
    bounds: BoundingBox2,
}

impl FakePolyline {
    fn new(points: &[(f64, f64)]) -> Self {
        let points: Vec<Point2> = points.iter().map(|&(x, y)| Point2::new(x, y)).collect();
        let mut bounds = BoundingBox2::default();
        points.iter().for_each(|point| bounds.add_point(*point));
        Self { points, bounds }
    }
}

impl Polygon2d for FakePolyline {
    fn bounding(&self) -> &BoundingBox2 {
        &self.bounds
    }
    fn deflection_over_estimation(&self) -> f64 {
        0.0
    }
    fn nb_segments(&self) -> i32 {
        self.points.len() as i32 - 1
    }
    fn segment(&self, index: i32) -> Result<(Point2, Point2), Failure> {
        Ok((self.points[index as usize - 1], self.points[index as usize]))
    }
}

#[test]
fn two_crossing_segments_give_one_edge_edge_point() {
    let first = FakePolyline::new(&[(0.0, 0.0), (2.0, 2.0)]);
    let second = FakePolyline::new(&[(0.0, 2.0), (2.0, 0.0)]);
    let interference = InterferencePolygon2d::between(&first, &second).unwrap();
    assert_eq!(
        (
            interference.nb_section_points(),
            interference.nb_tangent_zones()
        ),
        (1, 0)
    );
    assert_eq!(interference.pnt2d_value(1).unwrap(), Point2::new(1.0, 1.0));
    assert_eq!(
        interference.pnt_value(1).unwrap().type_on_first(),
        PIType::Edge
    );
}

#[test]
fn overlapping_collinear_segments_give_a_tangent_zone() {
    let first = FakePolyline::new(&[(0.0, 0.0), (4.0, 0.0)]);
    let second = FakePolyline::new(&[(1.0, 0.0), (3.0, 0.0)]);
    let interference = InterferencePolygon2d::between(&first, &second).unwrap();
    assert_eq!(interference.nb_tangent_zones(), 1);
    assert_eq!(
        interference.zone_value(1).unwrap().param_on_second(),
        (0.0, 1.0)
    );
}

/// A fake polyline tool over a vector of 3d points.
struct FakePolygonTool;

struct FakePolygon3d {
    points: Vec<Point3>,
    bounds: BoundingBox3,
}

impl Polygon3dTool for FakePolygonTool {
    type Polygon = FakePolygon3d;
    fn bounding(polygon: &FakePolygon3d) -> &BoundingBox3 {
        &polygon.bounds
    }
    fn deflection_over_estimation(_polygon: &FakePolygon3d) -> f64 {
        0.0
    }
    fn closed(_polygon: &FakePolygon3d) -> bool {
        false
    }
    fn nb_segments(polygon: &FakePolygon3d) -> i32 {
        polygon.points.len() as i32 - 1
    }
    fn begin_of_seg(polygon: &FakePolygon3d, index: i32) -> Result<Point3, Failure> {
        Ok(polygon.points[index as usize - 1])
    }
    fn end_of_seg(polygon: &FakePolygon3d, index: i32) -> Result<Point3, Failure> {
        Ok(polygon.points[index as usize])
    }
}

/// A fake one-triangle polyhedron in the plane z = 0.
struct FakeTriangleTool;

struct FakeTriangle {
    points: [Point3; 3],
    bounds: BoundingBox3,
    boxes: Array1<'static, BoundingBox3>,
}

impl FakeTriangle {
    fn new() -> Self {
        let points = [
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
        ];
        let mut bounds = BoundingBox3::default();
        points.iter().for_each(|point| bounds.add_point(*point));
        let mut boxes: Array1<'static, BoundingBox3> = Array1::with_bounds(1, 1).unwrap();
        *boxes.change_value(1).unwrap() = bounds.clone();
        Self {
            points,
            bounds,
            boxes,
        }
    }
}

impl PolyhedronTool for FakeTriangleTool {
    type Polyhedron = FakeTriangle;
    fn bounding(polyhedron: &FakeTriangle) -> &BoundingBox3 {
        &polyhedron.bounds
    }
    fn components_bounding(polyhedron: &FakeTriangle) -> &Array1<'static, BoundingBox3> {
        &polyhedron.boxes
    }
    fn deflection_over_estimation(_polyhedron: &FakeTriangle) -> f64 {
        0.0
    }
    fn triangle(_polyhedron: &FakeTriangle, _index: i32) -> Result<[i32; 3], Failure> {
        Ok([1, 2, 3])
    }
    fn point(polyhedron: &FakeTriangle, index: i32) -> Result<Point3, Failure> {
        Ok(polyhedron.points[index as usize - 1])
    }
    fn tri_connex(
        _polyhedron: &FakeTriangle,
        _triang: i32,
        _pivot: i32,
        _pedge: i32,
    ) -> Result<(i32, i32, i32), Failure> {
        Ok((0, 0, 0))
    }
    fn is_on_bound(
        _polyhedron: &FakeTriangle,
        _index1: i32,
        _index2: i32,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    fn get_border_deflection(_polyhedron: &FakeTriangle) -> f64 {
        0.0
    }
}

type FakeInterference = InterferencePolygonPolyhedron<FakePolygonTool, FakeTriangleTool>;

#[test]
fn a_segment_through_the_triangle_hits_its_face() {
    let points = vec![Point3::new(0.5, 0.5, -1.0), Point3::new(0.5, 0.5, 1.0)];
    let mut bounds = BoundingBox3::default();
    points.iter().for_each(|point| bounds.add_point(*point));
    let polygon = FakePolygon3d { points, bounds };
    let interference = FakeInterference::with_polygon(&polygon, &FakeTriangle::new()).unwrap();
    assert!(interference.nb_section_points() >= 1);
    let point = interference.pnt_value(1).unwrap();
    assert_eq!(point.type_on_second(), PIType::Face);
    assert_eq!(point.pnt(), Point3::new(0.5, 0.5, 0.0));
}

#[test]
fn a_line_through_a_vertex_hits_the_first_vertex_or_the_edge_before_it() {
    let vertical = |x: f64, y: f64| {
        Line3::new(
            Point3::new(x, y, 3.0),
            Direction3::new(0.0, 0.0, 1.0).unwrap(),
        )
    };
    let first = FakeInterference::with_line(vertical(0.0, 0.0), &FakeTriangle::new()).unwrap();
    let point = first.pnt_value(1).unwrap();
    assert_eq!(
        (point.type_on_second(), point.info_second().1),
        (PIType::Vertex, 1)
    );
    // The second corner lies on the first edge, which is tested before it, as natively.
    let second = FakeInterference::with_line(vertical(2.0, 0.0), &FakeTriangle::new()).unwrap();
    let (dimension, low, high, _) = second.pnt_value(1).unwrap().info_second();
    assert_eq!((dimension, low, high), (PIType::Edge, 1, 2));
}
