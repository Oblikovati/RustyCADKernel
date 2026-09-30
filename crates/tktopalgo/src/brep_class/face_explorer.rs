// Created on: 1992-11-19
// Created by: Remi LEQUETTE
// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of BRepClass_FaceExplorer.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass_FaceExplorer`: the wires and edges of a face, the classification
//! segments from a point and the face bounds, for the TopClass face classifier.
use super::edge::{BRepClassEdge, BndBoxState};
use super::face_explorer_build::ExplorerTopology;
use super::intersector::BRepClassIntersector;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::topclass::FaceExplorer;
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};

/// `Probing_Start`: the first fraction of an edge range where a segment is probed.
pub(super) const PROBING_START: f64 = 0.123;

/// `THE_MIN_EDGES_FOR_BOUNDING_BOX`: at most this many edges never use boxes.
const MIN_EDGES_FOR_BOUNDING_BOX: usize = 10;

/// `BRepClass_FaceExplorer::WireData`: the edges of one wire, a range of the edge list.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WireData {
    pub(super) first_edge: usize,
    pub(super) nb_edges: usize,
}

/// `BRepClass_FaceExplorer` (docs/topalgo-design.md section 2): the face (FORWARD),
/// its wires and edges with their pcurves cached, the edges probed for segments, the
/// iteration state and the face UV bounds (computed on the first CheckPoint).
///
/// The native default maximum tolerance is 0.1 and boxes are off. A failure of the
/// face bounds in the TopClass `CheckPoint` is kept and returned by the next
/// `Segment`/`OtherSegment` (the trait's CheckPoint cannot fail).
///
/// ```
/// use rustycad_tkbrep::topods::Face;
/// use rustycad_tktopalgo::brep_class::BRepClassFaceExplorer;
/// let explorer = BRepClassFaceExplorer::new(&Face::default()).unwrap();
/// assert_eq!(explorer.max_tolerance(), 0.1);
/// assert!(!explorer.more_wires());
/// ```
pub struct BRepClassFaceExplorer {
    my_face: Face,
    my_wires: Vec<WireData>,
    pub(super) my_edges: Vec<BRepClassEdge>,
    pub(super) my_probe_edges: Vec<usize>,
    my_current_wire: usize,
    my_current_edge: usize,
    my_current_edge_end: usize,
    pub(super) my_cur_edge_ind: usize,
    pub(super) my_cur_edge_par: f64,
    my_max_tolerance: f64,
    my_use_bnd_box: bool,
    my_u_min: f64,
    my_u_max: f64,
    my_v_min: f64,
    my_v_max: f64,
    // The failure of the last trait CheckPoint, returned by the next segment request.
    pending_failure: Option<Failure>,
}

impl BRepClassFaceExplorer {
    /// `BRepClass_FaceExplorer(F)`: explores `face` oriented FORWARD, caching the
    /// pcurve of each edge and its next edge at its last vertex; the exceptions of
    /// the topology casts and pcurve queries are returned;
    /// `BRepClassFaceExplorer::new(&face)?`.
    pub fn new(face: &Face) -> Result<Self, Failure> {
        let forward = Face::unchecked(face.oriented(Orientation::Forward));
        let topology = ExplorerTopology::of_face(&forward)?;
        Ok(Self {
            my_face: forward,
            my_wires: topology.wires,
            my_edges: topology.edges,
            my_probe_edges: topology.probe_edges,
            my_current_wire: 0,
            my_current_edge: 0,
            my_current_edge_end: 0,
            my_cur_edge_ind: 0,
            my_cur_edge_par: PROBING_START,
            my_max_tolerance: 0.1,
            my_use_bnd_box: false,
            my_u_min: precision::infinite(),
            my_u_max: -precision::infinite(),
            my_v_min: precision::infinite(),
            my_v_max: -precision::infinite(),
            pending_failure: None,
        })
    }

    /// `ComputeFaceBounds()` (protected): the surface bounds, or BRepTools::UVBounds
    /// when one of them is infinite; `explorer.compute_face_bounds()?`.
    pub fn compute_face_bounds(&mut self) -> Result<(), Failure> {
        let (surface, _location) = BRepTool::surface_with_location(&self.my_face)?;
        let Some(surface) = surface.get() else {
            return Err(Failure::with_message(
                "Standard_NullObject",
                "BRepClass_FaceExplorer::ComputeFaceBounds - the face has no surface; expected a face with a surface",
            ));
        };
        (self.my_u_min, self.my_u_max, self.my_v_min, self.my_v_max) = surface.bounds();
        if self.has_infinite_bound() {
            (self.my_u_min, self.my_u_max, self.my_v_min, self.my_v_max) =
                brep_tools::uv_bounds(&self.my_face)?;
        }
        Ok(())
    }

    fn has_infinite_bound(&self) -> bool {
        precision::is_infinite(self.my_u_min)
            || precision::is_infinite(self.my_u_max)
            || precision::is_infinite(self.my_v_min)
            || precision::is_infinite(self.my_v_max)
    }

    /// `CheckPoint(thePoint)`: computes the face bounds on the first call; moves a
    /// point so far from the bounds' centre that the bounds are below its epsilon
    /// (or at an infinite distance) near the bounds and returns false, else returns
    /// true; `explorer.check_point(&mut point)?`.
    pub fn check_point(&mut self, point: &mut Point2) -> Result<bool, Failure> {
        if self.my_u_min > self.my_u_max {
            self.compute_face_bounds()?;
        }
        if self.has_infinite_bound() {
            return Ok(true);
        }
        let center = Point2::new(
            (self.my_u_min + self.my_u_max) / 2.0,
            (self.my_v_min + self.my_v_max) / 2.0,
        );
        let distance: f64 = center.distance(*point);
        if precision::is_infinite(distance) {
            let (du, dv) = (self.my_u_max - self.my_u_min, self.my_v_max - self.my_v_min);
            *point = Point2::new(self.my_u_min - du, self.my_v_min - dv);
            return Ok(false);
        }
        let an_epsilon: f64 = epsilon(distance);
        if an_epsilon > std_max(self.my_u_max - self.my_u_min, self.my_v_max - self.my_v_min) {
            let direction = Direction2::from_vector(Vector2::between(center, *point))?;
            let moved = center.xy() + direction.xy() * (2.0 * an_epsilon);
            *point = Point2::new(moved.x(), moved.y());
            return Ok(false);
        }
        Ok(true)
    }

    /// `Reject(P)`: never rejects; `explorer.reject(&point)`.
    pub fn reject(&self, _point: &Point2) -> bool {
        false
    }

    /// `Segment(P, L, Par)`: restarts the probing at the first probe edge and
    /// returns the first segment ([`Self::other_segment`]);
    /// `let (found, line, par) = explorer.segment(&point)?;`.
    pub fn segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        self.my_cur_edge_ind = 0;
        self.my_cur_edge_par = PROBING_START;
        self.other_segment(point)
    }

    /// `InitWires()`; `explorer.init_wires()`.
    pub fn init_wires(&mut self) {
        self.my_current_wire = 0;
    }

    /// `MoreWires()`; `explorer.more_wires()`.
    pub fn more_wires(&self) -> bool {
        self.my_current_wire < self.my_wires.len()
    }

    /// `NextWire()`; `explorer.next_wire()`.
    pub fn next_wire(&mut self) {
        self.my_current_wire += 1;
    }

    /// `RejectWire(L, Par)`: never rejects; `explorer.reject_wire(&line, 1.0)`.
    pub fn reject_wire(&self, _line: &Line2, _param: f64) -> bool {
        false
    }

    /// `InitEdges()`: the edges of the current wire. Past the last wire (an
    /// unchecked vector access natively) the edge range is empty;
    /// `explorer.init_edges()`.
    pub fn init_edges(&mut self) {
        let wire: WireData = self
            .my_wires
            .get(self.my_current_wire)
            .copied()
            .unwrap_or_default();
        self.my_current_edge = wire.first_edge;
        self.my_current_edge_end = wire.first_edge + wire.nb_edges;
    }

    /// `MoreEdges()`; `explorer.more_edges()`.
    pub fn more_edges(&self) -> bool {
        self.my_current_edge < self.my_current_edge_end
    }

    /// `NextEdge()`; `explorer.next_edge()`.
    pub fn next_edge(&mut self) {
        self.my_current_edge += 1;
    }

    /// `RejectEdge(L, Par)`: never rejects; `explorer.reject_edge(&line, 1.0)`.
    pub fn reject_edge(&self, _line: &Line2, _param: f64) -> bool {
        false
    }

    /// `CurrentEdge(E, Or)`: a copy of the current edge with the explorer's maximum
    /// tolerance and box switch, and its orientation; Standard_OutOfRange past the
    /// last edge; `let (edge, orientation) = explorer.current_edge()?;`.
    pub fn current_edge(&self) -> Result<(BRepClassEdge, Orientation), Failure> {
        let Some(current) = self.my_edges.get(self.my_current_edge) else {
            return Err(Failure::out_of_range(&format!(
                "BRepClass_FaceExplorer::CurrentEdge - edge {} outside [0, {})",
                self.my_current_edge,
                self.my_edges.len()
            )));
        };
        let mut edge: BRepClassEdge = current.clone();
        let orientation: Orientation = edge.edge().orientation();
        edge.set_max_tolerance(self.my_max_tolerance);
        edge.set_use_bnd_box(self.my_use_bnd_box);
        Ok((edge, orientation))
    }

    /// `MaxTolerance()`; `explorer.max_tolerance()`.
    pub fn max_tolerance(&self) -> f64 {
        self.my_max_tolerance
    }

    /// `SetMaxTolerance(theValue)`; `explorer.set_max_tolerance(0.2)`.
    pub fn set_max_tolerance(&mut self, value: f64) {
        self.my_max_tolerance = value;
    }

    /// `UseBndBox()`; `explorer.use_bnd_box()`.
    pub fn use_bnd_box(&self) -> bool {
        self.my_use_bnd_box
    }

    /// `SetUseBndBox(theValue)`: switching boxes on builds the missing pcurve boxes
    /// of the wire edges (a failed build makes the box unavailable);
    /// `explorer.set_use_bnd_box(true)`.
    pub fn set_use_bnd_box(&mut self, value: bool) {
        if !value || self.my_use_bnd_box {
            self.my_use_bnd_box = value;
            return;
        }
        for wire in &self.my_wires {
            for edge in &mut self.my_edges[wire.first_edge..wire.first_edge + wire.nb_edges] {
                build_bounding_box(edge);
            }
        }
        self.my_use_bnd_box = true;
    }

    /// `ShouldUseBndBox()`: more than 10 edges, at least half of them B-spline,
    /// Bezier or offset pcurves; the adaptor's exceptions are returned;
    /// `explorer.should_use_bnd_box()?`.
    pub fn should_use_bnd_box(&self) -> Result<bool, Failure> {
        if self.my_edges.len() <= MIN_EDGES_FOR_BOUNDING_BOX {
            return Ok(false);
        }
        let mut nb_spline_edges: usize = 0;
        for edge in self.my_edges.iter().filter(|edge| !edge.curve().is_null()) {
            let kind = Geom2dAdaptorCurve::with_curve(edge.curve())?.get_type()?;
            if matches!(
                kind,
                CurveType::BSplineCurve | CurveType::BezierCurve | CurveType::OffsetCurve
            ) {
                nb_spline_edges += 1;
            }
        }
        Ok(nb_spline_edges * 2 >= self.my_edges.len())
    }

    // The failure kept by the trait CheckPoint, taken once.
    fn take_pending_failure(&mut self) -> Result<(), Failure> {
        self.pending_failure.take().map_or(Ok(()), Err)
    }
}

// One edge of `SetUseBndBox(true)`: a box not built yet is built from the pcurve
// (BndLib_Add2dCurve without tolerance); no pcurve or a failure makes it unavailable.
fn build_bounding_box(edge: &mut BRepClassEdge) {
    if edge.bounding_box_state() != BndBoxState::NotBuilt {
        return;
    }
    if edge.curve().is_null() {
        edge.set_bounding_box_unavailable();
        return;
    }
    let mut bounds = BoundingBox2::default();
    let (first, last) = (edge.first_parameter(), edge.last_parameter());
    match Add2dCurve::add_curve_range(edge.curve(), first, last, 0.0, &mut bounds) {
        Ok(()) => edge.set_bounding_box(&bounds),
        Err(_) => edge.set_bounding_box_unavailable(),
    }
}

impl FaceExplorer for BRepClassFaceExplorer {
    type Edge = BRepClassEdge;
    type Intersector = BRepClassIntersector;
    /// A failure of the face bounds is kept for the next segment request and the
    /// point is accepted (docs/topalgo-design.md section 2).
    fn check_point(&mut self, point: &mut Point2) -> bool {
        match BRepClassFaceExplorer::check_point(self, point) {
            Ok(accepted) => accepted,
            Err(failure) => {
                self.pending_failure = Some(failure);
                true
            }
        }
    }
    fn reject(&self, point: &Point2) -> bool {
        BRepClassFaceExplorer::reject(self, point)
    }
    fn segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        self.take_pending_failure()?;
        BRepClassFaceExplorer::segment(self, point)
    }
    fn other_segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        self.take_pending_failure()?;
        BRepClassFaceExplorer::other_segment(self, point)
    }
    fn init_wires(&mut self) {
        BRepClassFaceExplorer::init_wires(self);
    }
    fn more_wires(&self) -> bool {
        BRepClassFaceExplorer::more_wires(self)
    }
    fn next_wire(&mut self) {
        BRepClassFaceExplorer::next_wire(self);
    }
    fn reject_wire(&self, line: &Line2, param: f64) -> bool {
        BRepClassFaceExplorer::reject_wire(self, line, param)
    }
    fn init_edges(&mut self) {
        BRepClassFaceExplorer::init_edges(self);
    }
    fn more_edges(&self) -> bool {
        BRepClassFaceExplorer::more_edges(self)
    }
    fn next_edge(&mut self) {
        BRepClassFaceExplorer::next_edge(self);
    }
    fn reject_edge(&self, line: &Line2, param: f64) -> bool {
        BRepClassFaceExplorer::reject_edge(self, line, param)
    }
    fn current_edge(&self) -> Result<(BRepClassEdge, Orientation), Failure> {
        BRepClassFaceExplorer::current_edge(self)
    }
}
