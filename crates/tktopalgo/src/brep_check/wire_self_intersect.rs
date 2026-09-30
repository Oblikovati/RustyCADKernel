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
// Rust translation of BRepCheck_Wire::SelfIntersect (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Wire.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Wire::SelfIntersect`: the 2d intersections of the pcurves of the wire
//! edges on a face (each edge with itself and with the later edges), accepted within
//! the tolerance of a vertex. The pair checks are in `wire_self_pairs.rs`.
use super::explore::sub_shapes;
use super::status::BRepCheckStatus;
use super::wire::BRepCheckWire;
use super::wire_closed::record;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator};
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::{Domain, Position};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point3;

/// `tolint`: the tolerance of the 2d intersections.
pub(super) const TOLINT: f64 = 1.0e-10;

/// How the scan of the edges ended.
pub(super) enum Scan {
    /// No intersection outside the vertex tolerances.
    Clean,
    /// `SelfIntersectingWire` on this edge (with itself, or without pcurve).
    FoundOne(Edge),
    /// `SelfIntersectingWire` between these edges.
    Found(Edge, Edge),
    /// A later edge without pcurve or with an empty range: returned without update.
    Leave(BRepCheckStatus),
}

/// The state of the scan: the native locals of SelfIntersect.
pub(super) struct SelfIntersect<'a> {
    pub(super) face: &'a Face,
    /// `HS`: the unrestricted face surface.
    pub(super) hs: BRepAdaptorSurface,
    pub(super) inter: GInter,
    pub(super) c1: Geom2dAdaptorCurve,
    pub(super) c2: Geom2dAdaptorCurve,
    pub(super) domain1: Domain,
    tab_dom: Vec<Domain>,
    tab_cur: Vec<Handle<dyn Geom2dCurve>>,
    boxes: Vec<BoundingBox2>,
    edges: Vec<Edge>,
}

/// The 3d point of a curve parameter: on the 3d curve of `edge` when it has one, else
/// on the surface at the pcurve point.
pub(super) fn point_on_edge(
    edge: &Edge,
    parameter: f64,
    pcurve: &Geom2dAdaptorCurve,
    hs: &BRepAdaptorSurface,
) -> Result<Point3, Failure> {
    let found = BRepTool::curve_with_location(edge)?;
    if let Some(curve) = found.curve.get() {
        let mut point = curve.value(parameter)?;
        point.transform(found.location.transformation());
        return Ok(point);
    }
    let uv = pcurve.value(parameter)?;
    hs.value(uv.x(), uv.y())
}

impl<'a> SelfIntersect<'a> {
    fn new(face: &'a Face, edges: Vec<Edge>) -> Result<Self, Failure> {
        let count = edges.len();
        Ok(Self {
            face,
            hs: BRepAdaptorSurface::with_face(face, false)?,
            inter: GInter::new(),
            c1: Geom2dAdaptorCurve::default(),
            c2: Geom2dAdaptorCurve::default(),
            domain1: Domain::new(),
            tab_dom: vec![Domain::new(); count],
            tab_cur: vec![Handle::default(); count],
            boxes: vec![BoundingBox2::default(); count],
            edges,
        })
    }

    /// Loads `curve` in `adaptor` over `[first, last]` clamped to a non-periodic
    /// curve's bounds ("to avoid exception in Segment if C1 is BSpline - IFV"), sets
    /// `domain` and the box of the edge.
    fn prepare(
        &mut self,
        index: usize,
        curve: &Handle<dyn Geom2dCurve>,
        (mut first, mut last): (f64, f64),
        second: bool,
    ) -> Result<(), Failure> {
        let adaptor = if second { &self.c2 } else { &self.c1 };
        adaptor.load(curve)?;
        if !adaptor.is_periodic()? {
            if adaptor.first_parameter()? > first {
                first = adaptor.first_parameter()?;
            }
            if adaptor.last_parameter()? < last {
                last = adaptor.last_parameter()?;
            }
        }
        let (pfirst, plast) = BRepTool::uv_points_on_face(&self.edges[index], self.face)?;
        let domain = if second {
            &mut self.tab_dom[index]
        } else {
            &mut self.domain1
        };
        domain.set_values(pfirst, first, TOLINT, plast, last, TOLINT);
        let adaptor: &dyn Adaptor2dCurve2d = if second { &self.c2 } else { &self.c1 };
        Add2dCurve::add_adaptor_range(
            adaptor,
            first,
            last,
            precision::p_confusion(),
            &mut self.boxes[index],
        )
    }

    /// The whole scan.
    fn run(&mut self) -> Result<Scan, Failure> {
        let count = self.edges.len();
        for i in 0..count {
            if i == 0 {
                let found = BRepTool::curve_on_face(&self.edges[0], self.face)?;
                if found.pcurve.is_null() {
                    return Ok(Scan::FoundOne(self.edges[0].clone()));
                }
                self.prepare(0, &found.pcurve, (found.first, found.last), false)?;
            } else {
                self.c1.load(&self.tab_cur[i])?;
                self.domain1 = self.tab_dom[i];
            }
            if !self.self_points_ok(i)? {
                return Ok(Scan::FoundOne(self.edges[i].clone()));
            }
            if let Some(scan) = self.scan_pairs(i)? {
                return Ok(scan);
            }
        }
        Ok(Scan::Clean)
    }

    /// The pairs of edge `i` with the later edges (the later pcurves are prepared
    /// while `i` is the first edge).
    fn scan_pairs(&mut self, i: usize) -> Result<Option<Scan>, Failure> {
        for j in i + 1..self.edges.len() {
            if i == 0 {
                let found = BRepTool::curve_on_face(&self.edges[j], self.face)?;
                self.tab_cur[j] = found.pcurve.clone();
                if found.pcurve.is_null() {
                    return Ok(Some(Scan::Leave(BRepCheckStatus::NoCurveOnSurface)));
                }
                // A NaN range fails the native `last2 > first2` too.
                #[allow(clippy::neg_cmp_op_on_partial_ord)]
                let empty: bool = !(found.last > found.first);
                if empty {
                    return Ok(Some(Scan::Leave(BRepCheckStatus::InvalidRange)));
                }
                self.prepare(j, &found.pcurve, (found.first, found.last), true)?;
            } else {
                self.c2.load(&self.tab_cur[j])?;
            }
            if self.boxes[i].is_out_box(&self.boxes[j]) {
                continue;
            }
            let (e1, e2) = (self.edges[i].clone(), self.edges[j].clone());
            // NIZNHY-PKV 2010: an edge met twice is not intersected with itself.
            if e1.is_same(&e2) {
                continue;
            }
            let domain2 = self.tab_dom[j];
            self.inter
                .perform(&self.c1, &self.domain1, &self.c2, &domain2, TOLINT, TOLINT)?;
            if self.inter.is_done() && !self.pair_ok(&e1, &e2)? {
                return Ok(Some(Scan::Found(e1, e2)));
            }
        }
        Ok(None)
    }

    /// The self intersection of edge `i`: every inner point must lie within the
    /// tolerance of a vertex of the edge.
    fn self_points_ok(&mut self, i: usize) -> Result<bool, Failure> {
        self.inter
            .perform_self_on(&self.c1, &self.domain1, TOLINT, TOLINT)?;
        if !self.inter.is_done() {
            return Ok(true);
        }
        let e1 = self.edges[i].clone();
        for p in 1..=self.inter.nb_points()? {
            let ip = self.inter.point(p)?;
            if ip.transition_of_first().position_on_curve() != Position::Middle
                && ip.transition_of_second().position_on_curve() != Position::Middle
            {
                continue;
            }
            // Checked with the 3d tolerances: an intersection within the tolerance of
            // a vertex is correct.
            let p3d = point_on_edge(&e1, ip.param_on_first(), &self.c1, &self.hs)?;
            let mut local_ok = false;
            for vertex in sub_shapes(&e1, ShapeEnum::Vertex)? {
                let v = rustycad_tkbrep::topods::Vertex::unchecked(vertex);
                let tol: f64 = BRepTool::vertex_tolerance(&v)?;
                if BRepTool::pnt(&v)?.square_distance(&p3d) <= tol * tol {
                    local_ok = true;
                    break;
                }
            }
            if !local_ok {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl BRepCheckWire {
    /// `SelfIntersect(F, E1, E2, Update)`: `EmptyWire` for a wire without edges, else
    /// `SelfIntersectingWire` with the first intersecting edges written to `e1` and `e2`
    /// (only `e1` for an edge intersecting itself or a first edge without pcurve; the
    /// other is left unchanged, as the native output references are), or
    /// `NoCurveOnSurface`/`InvalidRange` (never recorded) for a later edge without
    /// pcurve or with an empty range; `checker.self_intersect(&face, &mut e1, &mut e2, false)?`.
    pub fn self_intersect(
        &self,
        face: &Face,
        e1: &mut Edge,
        e2: &mut Edge,
        update: bool,
    ) -> Result<BRepCheckStatus, Failure> {
        let list = self.base_own_list()?;
        let edges = wire_edges(&self.wire())?;
        if edges.is_empty() {
            record(&list, update, BRepCheckStatus::EmptyWire);
            return Ok(BRepCheckStatus::EmptyWire);
        }
        let mut scan = SelfIntersect::new(face, edges)?;
        let status = match scan.run()? {
            Scan::Clean => BRepCheckStatus::NoError,
            Scan::FoundOne(first) => {
                *e1 = first;
                BRepCheckStatus::SelfIntersectingWire
            }
            Scan::Found(first, second) => {
                (*e1, *e2) = (first, second);
                BRepCheckStatus::SelfIntersectingWire
            }
            Scan::Leave(status) => return Ok(status),
        };
        record(&list, update, status);
        Ok(status)
    }
}

/// `EMap`: the distinct edges among the children of the wire, in order (the native
/// map has the default hasher, so one edge with two orientations counts twice).
fn wire_edges(wire: &Shape) -> Result<Vec<Edge>, Failure> {
    let mut map: IndexedMap<Shape> = IndexedMap::default();
    let mut iterator = ShapeIterator::new(wire, true, true);
    while let Ok(child) = iterator.value() {
        if child.shape_type()? == ShapeEnum::Edge {
            map.add(child.clone());
        }
        iterator.next();
    }
    let mut edges = Vec::with_capacity(map.extent());
    for index in 1..=map.extent() {
        edges.push(Edge::unchecked(map.find_key(index)?.clone()));
    }
    Ok(edges)
}
