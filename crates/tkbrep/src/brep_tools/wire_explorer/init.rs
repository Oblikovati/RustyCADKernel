// Created on: 1993-01-21
// Created by: Remi LEQUETTE
// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of the BRepTools_WireExplorer::Init overloads
// (ModelingData/TKBRep/BRepTools/BRepTools_WireExplorer.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools_WireExplorer::Init`: the vertex map of the wire, the UV tolerances of
//! the face and the first edge.
use super::WireExplorer;
use crate::brep::BRepTool;
use crate::brep_adaptor::BRepAdaptorSurface;
use crate::brep_tools::uv_bounds;
use crate::topexp::{self, Explorer};
use crate::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use crate::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp;

/// UV bounds in the `Init` argument order `(UMin, UMax, VMin, VMax)`.
type InitBounds = (f64, f64, f64, f64);

/// What the vertex scan of `Init` collects.
#[derive(Default)]
struct VertexScan {
    /// The vertices met once (`vmap`), in insertion order.
    open_ends: IndexedMapOfShape,
    /// The edges infinite at their start (`anInfEmap`).
    infinite_edges: MapOfShape,
    /// The value the native `V1` variable keeps after the scan.
    last_first: Vertex,
}

impl WireExplorer {
    /// `Init(W)`: explores `wire` without a face; `explorer.init(&wire)?`.
    pub fn init(&mut self, wire: &Wire) -> Result<(), Failure> {
        self.init_on_face(wire, &Face::default())
    }

    /// `Init(W, F)`: explores `wire` on `face`; the UV bounds of the face refine the UV
    /// tolerances on cones, B-spline and Bezier surfaces; `explorer.init_on_face(&w, &f)?`.
    pub fn init_on_face(&mut self, wire: &Wire, face: &Face) -> Result<(), Failure> {
        self.reset();
        if wire.is_null() {
            return Ok(());
        }
        let mut bounds = (0.0, 0.0, 0.0, 0.0);
        if !face.is_null() {
            let kind = BRepAdaptorSurface::with_face(face, false)?.get_type()?;
            if matches!(
                kind,
                SurfaceType::Cone | SurfaceType::BSplineSurface | SurfaceType::BezierSurface
            ) {
                bounds = uv_bounds(face)?;
            }
        }
        self.init_with_bounds(wire, face, bounds)
    }

    /// `Init(W, F, UMin, UMax, VMin, VMax)`: explores `wire` on `face`, with the UV
    /// tolerances derived from the vertex (or edge) tolerances and the given bounds;
    /// `explorer.init_with_bounds(&wire, &face, (0.0, 1.0, 0.0, 1.0))?`.
    pub fn init_with_bounds(
        &mut self,
        wire: &Wire,
        face: &Face,
        bounds: InitBounds,
    ) -> Result<(), Failure> {
        self.reset();
        if wire.is_null() {
            return Ok(());
        }
        self.face = face.clone();
        self.reverse = false;
        if !self.face.is_null() && !self.set_uv_tolerances(wire, bounds)? {
            return Ok(());
        }
        let scan = self.map_vertices(wire)?;
        self.map_doubles(wire);
        let Some(first) = self.first_vertex(wire, scan)? else {
            return Ok(());
        };
        self.start_at(&first)
    }

    /// The first part of every `Init`: no current edge or vertex, empty maps.
    fn reset(&mut self) {
        self.edge = Edge::default();
        self.vertex = Vertex::default();
        self.map.clear(true);
        self.doubles.clear(true);
    }

    /// The UV tolerances on the face; false for an empty wire (every tolerance below
    /// `Precision::Confusion()`), which stops the initialisation.
    fn set_uv_tolerances(&mut self, wire: &Wire, bounds: InitBounds) -> Result<bool, Failure> {
        let (surface, _) = BRepTool::surface_with_location(&self.face)?;
        let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
        let tolerance = wire_tolerance(wire)?;
        if tolerance < precision::confusion() {
            return Ok(false);
        }
        self.tol_u = 2.0 * adaptor.u_resolution(tolerance)?;
        self.tol_v = 2.0 * adaptor.v_resolution(tolerance)?;
        let kind = adaptor.get_type()?;
        if kind == SurfaceType::Cone {
            self.tol_u = cone_u_tolerance(&adaptor, tolerance, bounds)?;
        }
        if matches!(
            kind,
            SurfaceType::BSplineSurface | SurfaceType::BezierSurface
        ) {
            self.refine_spline_tolerances(&adaptor, tolerance, bounds)?;
        }
        self.reverse = self.face.orientation() == Orientation::Reversed;
        Ok(true)
    }

    /// The B-spline and Bezier refinement: both tolerances become the larger one, raised
    /// so that it spans at least 1.5 vertex tolerances at the middle of the bounds.
    fn refine_spline_tolerances(
        &mut self,
        adaptor: &GeomAdaptorSurface,
        tolerance: f64,
        (umin, umax, vmin, vmax): InitBounds,
    ) -> Result<(), Failure> {
        let mut max_tol = self.tol_u.max(self.tol_v);
        let d1 = adaptor.d1((umax - umin) / 2.0, (vmax - vmin) / 2.0)?;
        let modulus = (d1.d1u.dot(d1.d1u) + d1.d1v.dot(d1.d1v)).sqrt();
        if modulus > gp::resolution() {
            if modulus * max_tol / tolerance < 1.5 {
                max_tol = 1.5 * tolerance / modulus;
            }
            self.tol_u = max_tol;
            self.tol_v = max_tol;
        }
        Ok(())
    }

    /// Maps each vertex to the edges starting there (INTERNAL and EXTERNAL edges
    /// skipped) and collects the unmatched vertices (the open ends, with the orientation
    /// of their first insertion), the infinite edges and the last first vertex met.
    fn map_vertices(&mut self, wire: &Wire) -> Result<VertexScan, Failure> {
        let mut scan = VertexScan::default();
        for child in ShapeIterator::new(wire, true, true) {
            let edge = topods::edge(&child)?;
            let orientation = edge.orientation();
            if matches!(orientation, Orientation::Internal | Orientation::External) {
                continue;
            }
            let (first, last) = topexp::vertices(&edge, true);
            scan.last_first = first.clone();
            if !first.is_null() {
                self.map_edge_from(&first, &edge);
                // The native `V1.Orientation(TopAbs_FORWARD)` changes the variable itself.
                scan.last_first = topods::vertex(&first.oriented(Orientation::Forward))?;
                toggle(&mut scan.open_ends, scan.last_first.clone().into_shape());
            }
            if !last.is_null() {
                toggle(&mut scan.open_ends, last.oriented(Orientation::Reversed));
            }
            if (first.is_null() || last.is_null()) && is_infinite_start(&edge)? {
                scan.infinite_edges.add(edge.into_shape());
            }
        }
        Ok(scan)
    }

    /// `myMap(V1).Append(E)`, binding an empty list first.
    fn map_edge_from(&mut self, vertex: &Vertex, edge: &Edge) {
        let key: Shape = vertex.clone().into_shape();
        if !self.map.is_bound(&key) {
            self.map.bind(key.clone(), Vec::new());
        }
        if let Ok(list) = self.map.change_find(&key) {
            list.push(edge.clone().into_shape());
        }
    }

    /// The edges met twice in the wire (`myDoubles`).
    fn map_doubles(&mut self, wire: &Wire) {
        let mut met = MapOfShape::default();
        for child in ShapeIterator::new(wire, true, true) {
            if !met.add(child.clone()) {
                self.doubles.add(child);
            }
        }
    }

    /// The vertex to start from: the first FORWARD open end (else the last first vertex
    /// the scan met, the native variable keeping its value); for a closed wire, none
    /// when an edge is infinite at its start (that edge becomes current), else the first
    /// vertex of the first FORWARD or REVERSED edge (else the last one scanned).
    fn first_vertex(&mut self, wire: &Wire, scan: VertexScan) -> Result<Option<Vertex>, Failure> {
        if !scan.open_ends.is_empty() {
            let first = first_forward_end(&scan.open_ends)?;
            return Ok(Some(first.unwrap_or(scan.last_first)));
        }
        for edge in scan.infinite_edges.iter() {
            let edge = topods::edge(edge)?;
            if is_infinite_start(&edge)? {
                self.edge = edge;
                self.vertex = Vertex::default();
                return Ok(None);
            }
        }
        for child in ShapeIterator::new(wire, true, true) {
            let edge = topods::edge(&child)?;
            if !matches!(
                edge.orientation(),
                Orientation::Internal | Orientation::External
            ) {
                return Ok(Some(topexp::vertices(&edge, true).0));
            }
        }
        Ok(Some(scan.last_first))
    }

    /// Makes the first edge starting at `first` current, removing it from the map.
    fn start_at(&mut self, first: &Vertex) -> Result<(), Failure> {
        if first.is_null() {
            return Ok(());
        }
        let Ok(list) = self.map.change_find(first) else {
            return Ok(());
        };
        if list.is_empty() {
            // `l.First()` on an empty list raises Standard_NoSuchObject natively.
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                "BRepTools_WireExplorer::Init: no edge starts at the first vertex; expected a non-empty list",
                "",
            ));
        }
        self.edge = topods::edge(&list.remove(0))?;
        self.vertex = topexp::first_vertex(&self.edge, true);
        Ok(())
    }
}

/// The largest vertex tolerance of the wire, or of its edges when every vertex
/// tolerance is below `Precision::Confusion()`.
fn wire_tolerance(wire: &Wire) -> Result<f64, Failure> {
    let mut tolerance: f64 = 0.0;
    for vertex in Explorer::new(wire, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        tolerance = BRepTool::vertex_tolerance(&topods::vertex(&vertex)?)?.max(tolerance);
    }
    if tolerance < precision::confusion() {
        for child in ShapeIterator::new(wire, true, true) {
            tolerance = BRepTool::edge_tolerance(&topods::edge(&child)?)?.max(tolerance);
        }
    }
    Ok(tolerance)
}

/// The U tolerance on a cone, whose U resolution is too small for infinite V bounds:
/// twice the larger of the U tolerances at the two V bounds, each capped at
/// `0.0005 * (UMax - UMin)`.
fn cone_u_tolerance(
    adaptor: &GeomAdaptorSurface,
    tolerance: f64,
    (umin, umax, vmin, vmax): InitBounds,
) -> Result<f64, Failure> {
    let max_tol = 0.0005 * (umax - umin);
    let at = |v: f64| -> Result<f64, Failure> {
        let magnitude = adaptor.d1(umin, v)?.d1u.magnitude();
        if magnitude <= precision::confusion() {
            return Ok(max_tol);
        }
        Ok(max_tol.min(tolerance / magnitude))
    };
    let (first, second) = (at(vmin)?, at(vmax)?);
    Ok(2.0 * first.max(second))
}

/// Adds `vertex` to the open ends, or removes it when it is already there.
fn toggle(open_ends: &mut IndexedMapOfShape, vertex: Shape) {
    let size = open_ends.extent();
    let index = open_ends.add(vertex.clone());
    if size >= index {
        open_ends.remove_key(&vertex);
    }
}

/// Whether the edge starts at infinity: a FORWARD edge whose range begins at
/// `-Precision::Infinite()`, or another one whose range ends at `Precision::Infinite()`.
fn is_infinite_start(edge: &Edge) -> Result<bool, Failure> {
    let (first, last) = BRepTool::range(edge)?;
    if edge.orientation() == Orientation::Forward {
        return Ok(first == -precision::infinite());
    }
    Ok(last == precision::infinite())
}

/// The first open end of FORWARD orientation, in insertion order.
fn first_forward_end(open_ends: &IndexedMapOfShape) -> Result<Option<Vertex>, Failure> {
    for index in 1..=open_ends.extent() {
        let end = open_ends.find_key(index)?;
        if end.orientation() == Orientation::Forward {
            return Ok(Some(topods::vertex(end)?));
        }
    }
    Ok(None)
}
