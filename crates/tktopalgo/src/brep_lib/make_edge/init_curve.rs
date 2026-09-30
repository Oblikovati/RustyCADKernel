// Created on: 1993-07-23
// Created by: Joelle CHAUVET
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
// Rust translation of the Geom_Curve Init members of BRepLib_MakeEdge.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Init` members of `BRepLib_MakeEdge` on a 3d curve.
use super::BRepLibMakeEdge;
use super::ends::{EdgeRange, Step, closed_vertices, open_vertices, ordered_range, vertex_pair};
use super::project::project_on_curve;
use crate::brep_lib::errors::{BRepLibEdgeError, null_object};
use crate::brep_lib::globals;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Point3};

/// The curve of an Init; natively a null handle is dereferenced.
pub(super) fn checked_curve(curve: &Handle<dyn GeomCurve>) -> Result<&dyn GeomCurve, Failure> {
    curve
        .get()
        .ok_or_else(|| null_object("BRepLib_MakeEdge::Init: null Geom_Curve; expected a curve"))
}

/// "kill trimmed curves": the basis of nested trimmed curves.
fn untrimmed(curve: &Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    let mut basis: Handle<dyn GeomCurve> = curve.clone();
    while let Some(trimmed) = basis.cast::<GeomTrimmedCurve>().get() {
        basis = trimmed.basis_curve();
    }
    basis
}

impl BRepLibMakeEdge {
    /// `Init(C)`: the whole curve; `maker.init_curve(&curve)?`.
    pub fn init_curve(&mut self, curve: &Handle<dyn GeomCurve>) -> Result<(), Failure> {
        let c: &dyn GeomCurve = checked_curve(curve)?;
        let (first, last): (f64, f64) = (c.first_parameter(), c.last_parameter());
        self.init_curve_params(curve, first, last)
    }

    /// `Init(C, p1, p2)`: the curve between two parameters;
    /// `maker.init_curve_params(&curve, 0.0, 1.0)?`.
    pub fn init_curve_params(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let null = Vertex::default();
        self.init_curve_vertices_params(curve, &null, &null, p1, p2)
    }

    /// `Init(C, P1, P2)`: the curve between the projections of two points, on vertices
    /// of tolerance `BRepLib::Precision()` (one vertex when the points are closer);
    /// `maker.init_curve_points(&curve, p1, p2)?`.
    pub fn init_curve_points(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        p1: Point3,
        p2: Point3,
    ) -> Result<(), Failure> {
        let tol: f64 = globals::precision();
        let (v1, v2) = vertex_pair(p1, p2, p1.distance(&p2) < tol, tol)?;
        self.init_curve_vertices(curve, &v1, &v2)
    }

    /// `Init(C, V1, V2)`: the curve between the projections of two vertices (a null
    /// vertex takes the curve end); `BRepLib_PointProjectionFailed` when one does not
    /// project; `maker.init_curve_vertices(&curve, &v1, &v2)?`.
    pub fn init_curve_vertices(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        v1: &Vertex,
        v2: &Vertex,
    ) -> Result<(), Failure> {
        let Some(p1) = self.projected(curve, v1, true)? else {
            return Ok(());
        };
        let Some(p2) = self.projected(curve, v2, false)? else {
            return Ok(());
        };
        self.init_curve_vertices_params(curve, v1, v2, p1, p2)
    }

    /// The parameter of `vertex`, the curve end for a null one; None after recording
    /// `BRepLib_PointProjectionFailed`.
    fn projected(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        vertex: &Vertex,
        first: bool,
    ) -> Result<Option<f64>, Failure> {
        if vertex.is_null() {
            let c: &dyn GeomCurve = checked_curve(curve)?;
            return Ok(Some(if first {
                c.first_parameter()
            } else {
                c.last_parameter()
            }));
        }
        let parameter: Option<f64> = project_on_curve(curve, vertex)?;
        if parameter.is_none() {
            self.my_error = BRepLibEdgeError::PointProjectionFailed;
        }
        Ok(parameter)
    }

    /// `Init(C, P1, P2, p1, p2)`: the curve between two parameters, on vertices at two
    /// points; `maker.init_curve_points_params(&curve, a, b, 0.0, 1.0)?`.
    pub fn init_curve_points_params(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        point1: Point3,
        point2: Point3,
        p1: f64,
        p2: f64,
    ) -> Result<(), Failure> {
        let tol: f64 = globals::precision();
        let (v1, v2) = vertex_pair(point1, point2, point1.distance(&point2) < tol, tol)?;
        self.init_curve_vertices_params(curve, &v1, &v2, p1, p2)
    }

    /// `Init(C, V1, V2, p1, p2)`, the member that builds the edge: trimmed curves are
    /// replaced by their basis, periodic parameters adjusted, others ordered (with their
    /// vertices) and checked; a closed curve takes one vertex, degenerated when its
    /// middle point is at it too; `maker.init_curve_vertices_params(&curve, &v1, &v2, 0.0, 1.0)?`.
    pub fn init_curve_vertices_params(
        &mut self,
        curve: &Handle<dyn GeomCurve>,
        vv1: &Vertex,
        vv2: &Vertex,
        pp1: f64,
        pp2: f64,
    ) -> Result<(), Failure> {
        let c: Handle<dyn GeomCurve> = untrimmed(curve);
        let basis: &dyn GeomCurve = checked_curve(&c)?;
        let bounds: (f64, f64) = (basis.first_parameter(), basis.last_parameter());
        let periodic: bool = basis.is_periodic();
        let adaptor = GeomAdaptorCurve::with_curve(&c)?;
        let mut range: EdgeRange = match ordered_range(
            periodic,
            bounds,
            (pp1, pp2),
            (vv1, vv2),
            precision::p_confusion(),
        ) {
            Ok(range) => range,
            Err(error) => return self.record(error),
        };
        // check ponctuallity
        if !periodic && range.p2 - range.p1 <= gp::resolution() {
            return self.record(BRepLibEdgeError::LineThroughIdenticPoints);
        }
        match self.checked_ends(&adaptor, &mut range)? {
            Ok(flags) => self.build_curve_edge(&c, &range, flags),
            Err(error) => self.record(error),
        }
    }

    /// Records a failed construction; the shape and done flag are kept.
    pub(super) fn record(&mut self, error: BRepLibEdgeError) -> Result<(), Failure> {
        self.my_error = error;
        Ok(())
    }

    /// The end points, closed flag and vertex checks; returns `(closed, degenerated)`.
    fn checked_ends(
        &self,
        adaptor: &GeomAdaptorCurve,
        range: &mut EdgeRange,
    ) -> Step<(bool, bool)> {
        let p1inf: bool = precision::is_negative_infinite(range.p1);
        let p2inf: bool = precision::is_positive_infinite(range.p2);
        let point1: Option<Point3> = if p1inf {
            None
        } else {
            Some(adaptor.value(range.p1)?)
        };
        let point2: Option<Point3> = if p2inf {
            None
        } else {
            Some(adaptor.value(range.p2)?)
        };
        let preci: f64 = globals::precision();
        let closed: bool = match (point1, point2) {
            (Some(a), Some(b)) => a.distance(&b) <= preci,
            _ => false,
        };
        if !closed {
            return Ok(open_vertices(range, (point1, point2), preci)?.map(|()| (false, false)));
        }
        let p1: Point3 = point1.unwrap_or_default();
        let both_given: bool = match closed_vertices(range, p1, preci)? {
            Ok(both) => both,
            Err(error) => return Ok(Err(error)),
        };
        let degenerated: bool =
            both_given && p1.distance(&adaptor.value((range.p1 + range.p2) / 2.0)?) < preci;
        Ok(Ok((true, degenerated)))
    }

    /// The edge on the 3d curve `c` of tolerance `BRepLib::Precision()`, with the
    /// vertices of `range`; the maker is done.
    fn build_curve_edge(
        &mut self,
        c: &Handle<dyn GeomCurve>,
        range: &EdgeRange,
        (closed, degenerated): (bool, bool),
    ) -> Result<(), Failure> {
        let preci: f64 = globals::precision();
        let builder = BRepBuilder::default();
        let mut edge: Edge = self.oriented_vertices(range);
        builder.make_edge_with_curve(&mut edge, c, preci)?;
        self.my_shape = edge.clone().into_shape();
        self.add_vertices(&mut edge)?;
        builder.range(&edge, range.p1, range.p2, false)?;
        builder.degenerated(&edge, degenerated)?;
        edge.set_closed(closed)?;
        self.my_error = BRepLibEdgeError::EdgeDone;
        self.set_done();
        Ok(())
    }

    /// Orients the vertices of `range` (V1 FORWARD, V2 REVERSED), stores them and
    /// returns `myShape` as the edge to rebuild (`TopoDS::Edge(myShape)`).
    pub(super) fn oriented_vertices(&mut self, range: &EdgeRange) -> Edge {
        self.my_vertex1 = range.v1.clone();
        self.my_vertex1.set_orientation(Orientation::Forward);
        self.my_vertex2 = range.v2.clone();
        self.my_vertex2.set_orientation(Orientation::Reversed);
        Edge::unchecked(self.my_shape.clone())
    }

    /// Adds the non-null stored vertices to the new edge.
    pub(super) fn add_vertices(&self, edge: &mut Edge) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for vertex in [&self.my_vertex1, &self.my_vertex2] {
            if !vertex.is_null() {
                builder.add(edge, vertex)?;
            }
        }
        Ok(())
    }
}
