// Created on: 1995-12-15
// Created by: Jacques GOUSSARD
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of BRepCheck_Face::IntersectWires and Intersect (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Face.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Face::IntersectWires`: two wires of a face must not intersect in 2d,
//! except at their common vertices (the non-manifold case).
use super::errors::type_mismatch;
use super::explore::sub_shapes;
use super::face::BRepCheckFace;
use super::status::BRepCheckStatus;
use super::wire_closed::record;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point3;

/// `DataMapOfShapeBox2d`: the 2d boxes of the edges and wires (the default hasher, so
/// the orientation counts).
pub(super) type DataMapOfShapeBox2d = DataMap<Shape, BoundingBox2>;

/// `Inter2dTol`: the tolerance of the 2d intersections of two wires.
const INTER2D_TOL: f64 = 1.0e-10;

/// `C.Load(pcurve)` of `edge` on `face`, and its range clamped to the curve bounds
/// ("to avoid exception in Segment if C1 is BSpline - IFV"); Standard_NullObject for an
/// edge without pcurve.
fn load_pcurve(
    adaptor: &Geom2dAdaptorCurve,
    edge: &Edge,
    face: &Face,
) -> Result<(f64, f64), Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    adaptor.load(&found.pcurve)?;
    let (mut first, mut last) = (found.first, found.last);
    if adaptor.first_parameter()? > first {
        first = adaptor.first_parameter()?;
    }
    if adaptor.last_parameter()? < last {
        last = adaptor.last_parameter()?;
    }
    Ok((first, last))
}

/// The boxes of every edge and wire of `face`.
fn wire_boxes(face: &Face) -> Result<DataMapOfShapeBox2d, Failure> {
    let mut boxes = DataMapOfShapeBox2d::default();
    let adaptor = Geom2dAdaptorCurve::default();
    for wire in sub_shapes(face, ShapeEnum::Wire)? {
        let mut wire_box = BoundingBox2::default();
        for shape in sub_shapes(&wire, ShapeEnum::Edge)? {
            let edge = Edge::unchecked(shape);
            let (first, last) = load_pcurve(&adaptor, &edge, face)?;
            let mut edge_box = BoundingBox2::default();
            Add2dCurve::add_adaptor_range(&adaptor, first, last, 0.0, &mut edge_box)?;
            boxes.bind(edge.into_shape(), edge_box.clone());
            wire_box.add_box(&edge_box);
        }
        boxes.bind(wire, wire_box);
    }
    Ok(boxes)
}

/// The common vertices of two wires and their points on the surface.
struct CommonPoints {
    vertices: Vec<Vertex>,
    points: Vec<Point3>,
}

impl CommonPoints {
    fn of(
        wire1: &Shape,
        wire2: &Shape,
        face: &Face,
        surface: &BRepAdaptorSurface,
    ) -> Result<Self, Failure> {
        let mut map_w1 = MapOfShape::default();
        for vertex in sub_shapes(wire1, ShapeEnum::Vertex)? {
            map_w1.add(vertex);
        }
        let mut vertices = Vec::new();
        for vertex in sub_shapes(wire2, ShapeEnum::Vertex)? {
            if map_w1.contains(&vertex) {
                vertices.push(Vertex::unchecked(vertex));
            }
        }
        let mut points = Vec::with_capacity(vertices.len());
        for vertex in &vertices {
            let uv = BRepTool::parameters(vertex, face)?;
            points.push(surface.value(uv.x(), uv.y())?);
        }
        Ok(Self { vertices, points })
    }

    /// Whether `point` is within the tolerance of a common vertex plus 1e-8 (the
    /// possible tolerance of the intersection point).
    fn holds(&self, point: &Point3) -> Result<bool, Failure> {
        for (vertex, common) in self.vertices.iter().zip(&self.points) {
            let tolv: f64 = BRepTool::vertex_tolerance(vertex)? + 1.0e-8;
            if point.square_distance(common) <= tolv * tolv {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// The 2d intersection of two wires of the face.
struct WirePair<'a> {
    face: &'a Face,
    boxes: &'a DataMapOfShapeBox2d,
    surface: BRepAdaptorSurface,
    common: CommonPoints,
}

impl WirePair<'_> {
    /// The box of `edge`: the recorded one, else the box of its range.
    fn edge_box(
        &self,
        edge: &Edge,
        adaptor: &Geom2dAdaptorCurve,
        range: (f64, f64),
    ) -> Result<BoundingBox2, Failure> {
        let mut edge_box = BoundingBox2::default();
        if let Some(found) = self.boxes.seek(edge) {
            edge_box = found.clone();
        }
        if edge_box.is_void() {
            Add2dCurve::add_adaptor_range(adaptor, range.0, range.1, 0.0, &mut edge_box)?;
        }
        Ok(edge_box)
    }

    /// `Intersect(wir1, wir2, F, theMapEdgeBox)`.
    fn intersect(&self, wire1: &Shape, wire2: &Shape) -> Result<bool, Failure> {
        let (c1, c2) = (Geom2dAdaptorCurve::default(), Geom2dAdaptorCurve::default());
        let mut inter = GInter::new();
        for s1 in sub_shapes(wire1, ShapeEnum::Edge)? {
            let edge1 = Edge::unchecked(s1);
            let range1 = load_pcurve(&c1, &edge1, self.face)?;
            let box1 = self.edge_box(&edge1, &c1, range1)?;
            for s2 in sub_shapes(wire2, ShapeEnum::Edge)? {
                let edge2 = Edge::unchecked(s2);
                if edge1.is_same(&edge2) {
                    continue;
                }
                let range2 = load_pcurve(&c2, &edge2, self.face)?;
                let box2 = self.edge_box(&edge2, &c2, range2)?;
                if box1.is_out_box(&box2) {
                    continue;
                }
                let d1 = self.domain(&edge1, range1)?;
                let d2 = self.domain(&edge2, range2)?;
                inter.perform(&c1, &d1, &c2, &d2, INTER2D_TOL, INTER2D_TOL)?;
                if let Some(found) = self.verdict(&inter)? {
                    return Ok(found);
                }
            }
        }
        Ok(false)
    }

    fn domain(&self, edge: &Edge, (first, last): (f64, f64)) -> Result<Domain, Failure> {
        let (pfirst, plast) = BRepTool::uv_points_on_face(edge, self.face)?;
        let mut domain = Domain::new();
        domain.set_values(pfirst, first, INTER2D_TOL, plast, last, INTER2D_TOL);
        Ok(domain)
    }

    /// The answer of one edge pair: `Some(true)` for an intersection outside the
    /// common vertices (or a failed intersection), `Some(false)` when all segments or
    /// points are at common vertices, `None` without results.
    fn verdict(&self, inter: &GInter) -> Result<Option<bool>, Failure> {
        if !inter.is_done() {
            return Ok(Some(true));
        }
        let nb_segments = inter.nb_segments()?;
        if nb_segments > 0 {
            if self.common.points.is_empty() {
                return Ok(Some(true));
            }
            let mut coincident = 0;
            for i in 1..=nb_segments {
                let segment = inter.segment(i)?;
                if !segment.has_first_point() || !segment.has_last_point() {
                    return Ok(Some(true));
                }
                let first = segment.first_point()?.value();
                let last = segment.last_point()?.value();
                let first3d = self.surface.value(first.x(), first.y())?;
                let last3d = self.surface.value(last.x(), last.y())?;
                if self.common.holds_either(&first3d, &last3d)? {
                    coincident += 1;
                }
            }
            return Ok(Some(coincident != nb_segments));
        }
        let nb_points = inter.nb_points()?;
        if nb_points == 0 {
            return Ok(None);
        }
        if self.common.points.is_empty() {
            return Ok(Some(true));
        }
        let mut coincident = 0;
        for i in 1..=nb_points {
            let uv = inter.point(i)?.value();
            let point = self.surface.value(uv.x(), uv.y())?;
            if self.common.holds(&point)? {
                coincident += 1;
            }
        }
        Ok(Some(coincident != nb_points))
    }
}

impl CommonPoints {
    /// Whether one of the segment ends is `IsEqual` to a common vertex point within the
    /// tolerance of that vertex (the first vertex matching either end counts).
    fn holds_either(&self, first: &Point3, last: &Point3) -> Result<bool, Failure> {
        for (vertex, common) in self.vertices.iter().zip(&self.points) {
            let tolv: f64 = BRepTool::vertex_tolerance(vertex)?;
            if first.is_equal(common, tolv) || last.is_equal(common, tolv) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl BRepCheckFace {
    /// `IntersectWires(Update)`: `RedundantWire` for a wire met twice, else
    /// `IntersectingWires` when two wires whose boxes meet intersect outside their
    /// common vertices; the result is kept for later calls (the wire map is not reset,
    /// so a second computation finds every wire again, as natively);
    /// `checker.intersect_wires(false)?`.
    pub fn intersect_wires(&self, update: bool) -> Result<BRepCheckStatus, Failure> {
        let list = self.own_list()?;
        let (done, result) = {
            let fields = self.fields.read();
            (fields.intdone, fields.intres)
        };
        if done {
            record(&list, update, result);
            return Ok(result);
        }
        {
            let mut fields = self.fields.write();
            fields.intdone = true;
            fields.intres = BRepCheckStatus::NoError;
        }
        let status = self.compute_intersect_wires()?;
        self.fields.write().intres = status;
        record(&list, update, status);
        Ok(status)
    }

    fn compute_intersect_wires(&self) -> Result<BRepCheckStatus, Failure> {
        let face = self.face();
        for wire in sub_shapes(&face.oriented(Orientation::Forward), ShapeEnum::Wire)? {
            let mut fields = self.fields.write();
            if fields.map_imb.is_bound(&wire) {
                // The same wire is met twice.
                return Ok(BRepCheckStatus::RedundantWire);
            }
            fields.map_imb.bind(wire, List::default());
        }
        let boxes = wire_boxes(&face)?;
        let wires = sub_shapes(&face, ShapeEnum::Wire)?;
        let nb_wire = self.fields.read().map_imb.extent();
        for index in 1..nb_wire {
            let wire1 = wires.get(index - 1).cloned().ok_or_else(|| {
                type_mismatch(&format!(
                    "BRepCheck_Face::IntersectWires: wire {index} of {}",
                    wires.len()
                ))
            })?;
            let box1 = boxes.seek(&wire1).cloned().unwrap_or_default();
            let mut box2 = BoundingBox2::default();
            for wire2 in wires.iter().skip(index) {
                // A wire without a recorded box keeps the previous box, as natively.
                if let Some(found) = boxes.seek(wire2) {
                    box2 = found.clone();
                }
                if !box1.is_void() && !box2.is_void() && box1.is_out_box(&box2) {
                    continue;
                }
                let surface = BRepAdaptorSurface::with_face(&face, false)?;
                let common = CommonPoints::of(&wire1, wire2, &face, &surface)?;
                let pair = WirePair {
                    face: &face,
                    boxes: &boxes,
                    surface,
                    common,
                };
                if pair.intersect(&wire1, wire2)? {
                    return Ok(BRepCheckStatus::IntersectingWires);
                }
            }
        }
        Ok(BRepCheckStatus::NoError)
    }
}
