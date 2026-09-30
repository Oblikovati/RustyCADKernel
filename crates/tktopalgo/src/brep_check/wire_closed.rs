// Created on: 1995-12-12
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
// Rust translation of BRepCheck_Wire::Closed, Closed2d and their helpers (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Wire.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Wire::Closed` (3d closure of the oriented edges) and `Closed2d` (closure
//! on a face, dpf 1997 and eap OCC35), with the distance helpers.
use super::errors::no_such_object;
use super::explore::{is_oriented, sub_shapes};
use super::result::{BRepCheckResult, SharedStatusList, add_status};
use super::status::BRepCheckStatus;
use super::wire::{BRepCheckWire, append_under_key, propagate};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Point2, Point3};

/// `IsDistanceIn3DTolerance`: the two points are closer than `tol3d`.
pub(super) fn is_distance_in_3d_tolerance(first: &Point3, last: &Point3, tol3d: f64) -> bool {
    first.distance(last) < tol3d
}

/// `IsDistanceIn2DTolerance`: within 1% of the surface ranges, else within twice the
/// larger parametric resolution of `tol3d` (refined by the first derivatives at the
/// middle point).
pub(super) fn is_distance_in_2d_tolerance(
    surface: &BRepAdaptorSurface,
    point: &Point2,
    reference: &Point2,
    tol3d: f64,
) -> Result<bool, Failure> {
    let mut dumax: f64 = 0.01 * (surface.last_u_parameter()? - surface.first_u_parameter()?);
    let mut dvmax: f64 = 0.01 * (surface.last_v_parameter()? - surface.first_v_parameter()?);
    let dumin: f64 = (point.x() - reference.x()).abs();
    let dvmin: f64 = (point.y() - reference.y()).abs();
    if dumin < dumax && dvmin < dvmax {
        return Ok(true);
    }
    dumax = surface.u_resolution(tol3d)?;
    dvmax = surface.v_resolution(tol3d)?;
    let um: f64 = (point.x() + reference.x()) / 2.0;
    let vm: f64 = (point.y() + reference.y()) / 2.0;
    let d1 = surface.d1(um, vm)?;
    let mdu: f64 = d1.d1u.magnitude();
    if mdu > precision::confusion() {
        dumax = std_max(tol3d / mdu, dumax);
    }
    let mdv: f64 = d1.d1v.magnitude();
    if mdv > precision::confusion() {
        dvmax = std_max(tol3d / mdv, dvmax);
    }
    let tol2d: f64 = 2.0 * std_max(dumax, dvmax);
    Ok(std_max(dumin, dvmin) < tol2d)
}

/// `GetPnt2d`: the point of `vertex` on the pcurve of `edge` on `face`, when the vertex
/// is an end of the edge and the pcurve exists.
fn get_pnt2d(vertex: &Vertex, edge: &Edge, face: &Face) -> Result<Option<Point2>, Failure> {
    let (first, last) = topexp::vertices(edge, false);
    if !vertex.is_same(&first) && !vertex.is_same(&last) {
        return Ok(None);
    }
    let found = BRepTool::curve_on_face(edge, face)?;
    let Some(pcurve) = found.pcurve.get() else {
        return Ok(None);
    };
    let parameter: f64 = BRepTool::parameter(vertex, edge)?;
    Ok(Some(pcurve.value(parameter)?))
}

/// The seam edges of `face`: the oriented closed edges met twice.
fn seam_edges(face: &Face) -> Result<Vec<Edge>, Failure> {
    let mut seams: Vec<Edge> = Vec::new();
    let mut not_seams = MapOfShape::default();
    let mut closed_edges = MapOfShape::default();
    for shape in sub_shapes(face, ShapeEnum::Edge)? {
        let edge = Edge::unchecked(shape);
        if not_seams.contains(&edge) {
            continue;
        }
        if !is_oriented(&edge) || !BRepTool::is_closed_on_face(&edge, face)? {
            not_seams.add(edge.into_shape());
            continue;
        }
        if !closed_edges.add(edge.clone().into_shape()) {
            seams.push(edge);
        }
    }
    Ok(seams)
}

/// `IsClosed2dForPeriodicFace` (Sergey KHROMOV, OCC325): on a face with seams, the 2d
/// distance of the wire ends at a seam vertex must be within 1% of the seam width (at
/// least the vertex resolution). The seam variable is reversed at each match, as
/// natively.
fn is_closed2d_for_periodic_face(
    face: &Face,
    (p1, p2): (&Point2, &Point2),
    vertex: &Vertex,
) -> Result<bool, Failure> {
    let seams = seam_edges(face)?;
    if seams.is_empty() {
        return Ok(true);
    }
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let tol: f64 = BRepTool::vertex_tolerance(vertex)?;
    let (ures, vres) = (surface.u_resolution(tol)?, surface.v_resolution(tol)?);
    let vicinity: f64 = (ures * ures + vres * vres).sqrt();
    let dist: f64 = p1.distance(*p2);
    for original in seams {
        let mut seam = original.clone();
        for v in sub_shapes(&original, ShapeEnum::Vertex)? {
            if !is_oriented(&v) || !v.is_same(vertex) {
                continue;
            }
            let Some(pnt1) = get_pnt2d(vertex, &seam, face)? else {
                continue;
            };
            seam = Edge::unchecked(seam.reversed());
            let Some(pnt2) = get_pnt2d(vertex, &seam, face)? else {
                continue;
            };
            let tol2d: f64 = std_max(pnt1.distance(pnt2) * 1.0e-2, vicinity);
            if dist > tol2d {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

/// The walk of the wire on a face by `BRepTools_WireExplorer`: the first edge and
/// vertex, the last edge, the number of edges, and the current vertex at the end.
struct Walk {
    first_edge: Edge,
    first_vertex: Vertex,
    last_edge: Edge,
    count: usize,
    end_vertex: Vertex,
}

impl Walk {
    fn of(wire: &rustycad_tkbrep::topods::Wire, face: &Face) -> Result<Self, Failure> {
        let mut explorer = WireExplorer::with_wire_on_face(wire, face)?;
        let first_edge = explorer.current().clone();
        let first_vertex = explorer.current_vertex().clone();
        let mut last_edge = Edge::default();
        let mut count: usize = 0;
        while explorer.more() {
            count += 1;
            last_edge = explorer.current().clone();
            explorer.next()?;
        }
        let end_vertex = explorer.current_vertex().clone();
        Ok(Self {
            first_edge,
            first_vertex,
            last_edge,
            count,
            end_vertex,
        })
    }

    /// Whether the walk starts and ends on infinite edge ends (the wire is then closed).
    fn ends_infinite(&self) -> Result<bool, Failure> {
        let (f, l) = BRepTool::range(&self.first_edge)?;
        let first_infinite = match self.first_edge.orientation() {
            Orientation::Forward => precision::is_negative_infinite(f),
            Orientation::Reversed => precision::is_positive_infinite(l),
            _ => false,
        };
        let (f, l) = BRepTool::range(&self.last_edge)?;
        let last_infinite = match self.last_edge.orientation() {
            Orientation::Forward => precision::is_positive_infinite(l),
            Orientation::Reversed => precision::is_negative_infinite(f),
            _ => false,
        };
        Ok(first_infinite && last_infinite)
    }

    /// The 2d start of the first edge and end of the last edge on `face`.
    fn uv_ends(&self, face: &Face) -> Result<(Point2, Point2), Failure> {
        let (temp, mut last) = BRepTool::uv_points_on_face(&self.last_edge, face)?;
        if self.last_edge.orientation() == Orientation::Reversed {
            last = temp;
        }
        let (a, b) = BRepTool::uv_points_on_face(&self.first_edge, face)?;
        let first = if self.first_edge.orientation() == Orientation::Reversed {
            b
        } else {
            a
        };
        Ok((first, last))
    }
}

impl BRepCheckWire {
    /// `Closed(Update)`: `NotConnected` when the oriented edges are not connected,
    /// `RedundantEdge` for an edge met three times or twice with one orientation,
    /// `NotClosed` when an oriented vertex is met an odd number of times; the result
    /// is kept for later calls (and a defect of the minimum check is returned as is).
    /// With `update`, the status is recorded; `checker.closed(false)?`.
    pub fn closed(&self, update: bool) -> Result<BRepCheckStatus, Failure> {
        let list = self.base_own_list()?;
        let (cdone, cstat) = {
            let fields = self.fields.read();
            (fields.cdone, fields.cstat)
        };
        if cdone {
            record(&list, update, cstat);
            return Ok(cstat);
        }
        self.fields.write().cdone = true;
        // `itl.Value()` of the wire's own list: a defect of the minimum check is kept.
        let head: Option<BRepCheckStatus> = list.read().first().copied();
        let head =
            head.ok_or_else(|| no_such_object("BRepCheck_Wire::Closed: empty status list"))?;
        if head != BRepCheckStatus::NoError {
            self.fields.write().cstat = head;
            return Ok(head);
        }
        let status = self.compute_closed()?;
        self.fields.write().cstat = status;
        record(&list, update, status);
        Ok(status)
    }

    /// The checks of `Closed`, filling `myMapVE` with the oriented vertices.
    fn compute_closed(&self) -> Result<BRepCheckStatus, Failure> {
        let edges = sub_shapes(&self.wire(), ShapeEnum::Edge)?;
        let mut map_s = MapOfShape::default();
        let mut cradoc: DataMap<Shape, List<Shape>, ShapeMapHasher> = DataMap::default();
        let mut map_ve = IndexedDataMapOfShapeListOfShape::default();
        for edge in edges.iter().filter(|e| is_oriented(e)) {
            if !cradoc.is_bound(edge) {
                cradoc.bind(edge.clone(), List::default());
            }
            cradoc.change_find(edge)?.append(edge.clone());
            map_s.add(edge.clone());
            for vertex in sub_shapes(edge, ShapeEnum::Vertex)? {
                if is_oriented(&vertex) {
                    append_under_key(&mut map_ve, &vertex, edge)?;
                }
            }
        }
        let nb_ori: usize = map_s.extent();
        if nb_ori >= 2 {
            map_s.clear(false);
            let first = edges
                .iter()
                .find(|e| is_oriented(e))
                .expect("an oriented edge");
            propagate(&map_ve, first, &mut map_s)?;
        }
        let connected: bool = nb_ori == map_s.extent();
        let odd_vertex: bool = map_ve.iter().any(|edges| edges.extent() % 2 != 0);
        self.fields.write().map_ve = map_ve;
        if !connected {
            return Ok(BRepCheckStatus::NotConnected);
        }
        if cradoc.values().any(redundant_occurrences) {
            return Ok(BRepCheckStatus::RedundantEdge);
        }
        if odd_vertex {
            return Ok(BRepCheckStatus::NotClosed);
        }
        Ok(BRepCheckStatus::NoError)
    }

    /// `Closed2d(F, Update)`: `Closed` first, then the oriented edges must all be
    /// walked by `BRepTools_WireExplorer` on `face`, and the walk's 2d and 3d ends must
    /// meet within the vertex tolerances (a walk between two infinite ends is closed);
    /// `checker.closed2d(&face, false)?`.
    pub fn closed2d(&self, face: &Face, update: bool) -> Result<BRepCheckStatus, Failure> {
        let list = self.base_own_list()?;
        let closed = self.closed(false)?;
        if closed != BRepCheckStatus::NoError {
            record(&list, update, closed);
            return Ok(closed);
        }
        let status = self.compute_closed2d(face)?;
        record(&list, update, status);
        Ok(status)
    }

    fn compute_closed2d(&self, face: &Face) -> Result<BRepCheckStatus, Failure> {
        let surface = BRepAdaptorSurface::with_face(face, false)?;
        let wire = self.wire();
        let oriented: usize = sub_shapes(&wire, ShapeEnum::Edge)?
            .iter()
            .filter(|e| is_oriented(e))
            .count();
        if oriented == 0 {
            return Ok(BRepCheckStatus::NoError);
        }
        let walk = Walk::of(&wire, face)?;
        if walk.count != oriented {
            return Ok(BRepCheckStatus::NotClosed);
        }
        if walk.ends_infinite()? {
            return Ok(BRepCheckStatus::NoError);
        }
        if walk.first_vertex.is_null() {
            return Ok(BRepCheckStatus::NotClosed);
        }
        let (first, last) = walk.uv_ends(face)?;
        if !is_closed2d_for_periodic_face(face, (&first, &last), &walk.first_vertex)? {
            return Ok(BRepCheckStatus::NotClosed);
        }
        let tol3d: f64 = std_max(
            BRepTool::vertex_tolerance(&walk.first_vertex)?,
            BRepTool::vertex_tolerance(&walk.end_vertex)?,
        );
        let reference = BRepTool::pnt(&walk.first_vertex)?;
        let end = BRepTool::pnt(&walk.end_vertex)?;
        let mut status = BRepCheckStatus::NoError;
        if !is_distance_in_2d_tolerance(&surface, &first, &last, tol3d)? {
            status = BRepCheckStatus::NotClosed;
        }
        if !is_distance_in_3d_tolerance(&reference, &end, tol3d) {
            status = BRepCheckStatus::NotClosed;
        }
        Ok(status)
    }

    /// `myMap(myShape)`.
    pub(super) fn base_own_list(&self) -> Result<SharedStatusList, Failure> {
        self.result_base().own_list()
    }
}

/// An edge met three times, or twice with the same orientation.
fn redundant_occurrences(occurrences: &List<Shape>) -> bool {
    if occurrences.extent() >= 3 {
        return true;
    }
    if occurrences.extent() != 2 {
        return false;
    }
    let first = occurrences.first().map(|s| s.orientation());
    let last = occurrences.last().map(|s| s.orientation());
    first == last
}

/// Records `status` in `list` when `update` is set.
pub(super) fn record(list: &SharedStatusList, update: bool, status: BRepCheckStatus) {
    if update {
        add_status(list, status);
    }
}
