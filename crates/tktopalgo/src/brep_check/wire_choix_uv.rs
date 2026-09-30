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
// Rust translation of ChoixUV and CurveDirForParameter (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Wire.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ChoixUV`: among the edges leaving a vertex on a face, the one the wire goes on
//! with (the smallest turning angle in 2d on a FORWARD face, the largest otherwise).
use super::wire_closed::{is_distance_in_2d_tolerance, is_distance_in_3d_tolerance};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{self, Point2, Vector2};
use std::f64::consts::PI;

/// `CurveDirForParameter`: the point and the first non-null derivative (up to the
/// 100th) of the curve at `parameter`.
fn curve_dir_for_parameter(
    curve: &Geom2dAdaptorCurve,
    parameter: f64,
) -> Result<(Point2, Vector2), Failure> {
    let tol: f64 = gp::resolution();
    let (point, mut direction) = curve.d1(parameter)?;
    if direction.magnitude() <= tol {
        for order in 2..=100 {
            direction = curve.dn(parameter, order)?;
            if direction.magnitude() > tol {
                break;
            }
        }
    }
    Ok((point, direction))
}

/// The pcurve of `edge` on `face` with its range.
fn pcurve_of(edge: &Edge, face: &Face) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_on_face(edge, face)?;
    Ok((found.pcurve, found.first, found.last))
}

/// The reference of the choice: the pivot point and direction on `edge`.
struct Pivot {
    orientation: Orientation,
    parameter: f64,
    point: Point2,
    direction: Vector2,
}

/// The running choice and the last candidate point and parameter (read again after
/// the loop, as natively, even when they belong to a skipped edge).
struct Choice {
    index_min: usize,
    min_angle: f64,
    max_angle: f64,
    last_point: Point2,
    last_parameter: f64,
}

impl Choice {
    /// Takes the candidate `index` with `angle` on a face of `orientation`.
    fn take(&mut self, index: usize, angle: f64, orientation: Orientation) {
        if orientation == Orientation::Forward {
            if angle < self.min_angle {
                self.index_min = index;
                self.min_angle = angle;
            }
        } else if angle > self.max_angle {
            self.index_min = index;
            self.max_angle = angle;
        }
    }
}

/// `ChoixUV(theVertex, theEdge, theFace, theLOfShape)`: keeps in `list` the edge the
/// wire goes on with from `vertex` after `edge` on `face`, or empties it; the
/// occurrences of `edge` are removed first.
pub(super) fn choix_uv(
    vertex: &Vertex,
    edge: &Edge,
    face: &Face,
    list: &mut List<Shape>,
) -> Result<(), Failure> {
    list.retain(|shape| !edge.is_same(shape));
    let tol3d: f64 = BRepTool::vertex_tolerance(vertex)?;
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let (c2d, first, last) = pcurve_of(edge, face)?;
    if c2d.is_null() {
        // JAG 10.12.96
        return Ok(());
    }
    let orientation = vertex.orientation();
    let parameter = if orientation == edge.orientation() {
        first
    } else {
        last
    };
    let (point, mut direction) =
        curve_dir_for_parameter(&Geom2dAdaptorCurve::with_curve(&c2d)?, parameter)?;
    if orientation != edge.orientation() {
        direction = direction.reversed();
    }
    let pivot = Pivot {
        orientation,
        parameter,
        point,
        direction,
    };
    let mut choice = Choice {
        index_min: 0,
        min_angle: real_last(),
        max_angle: real_first(),
        last_point: Point2::new(0.0, 0.0),
        last_parameter: 0.0,
    };
    let candidates: Vec<Shape> = list.iter().cloned().collect();
    for (position, candidate) in candidates.iter().enumerate() {
        let angle = candidate_angle(&pivot, candidate, face, &surface, tol3d, &mut choice)?;
        if let Some(angle) = angle {
            choice.take(position + 1, angle, face.orientation());
        }
    }
    let keep = choose(&pivot, &choice, (edge, face), &surface, (tol3d, list))?;
    match keep {
        Some(index) => keep_only(list, index),
        None => list.clear(),
    }
    Ok(())
}

/// The turning angle from the pivot direction to `candidate`, `None` when the
/// candidate is skipped (no pcurve, not at the pivot in 2d, null derivatives).
fn candidate_angle(
    pivot: &Pivot,
    candidate: &Shape,
    face: &Face,
    surface: &BRepAdaptorSurface,
    tol3d: f64,
    choice: &mut Choice,
) -> Result<Option<f64>, Failure> {
    let edge = Edge::unchecked(candidate.clone());
    let (c2d, first, last) = pcurve_of(&edge, face)?;
    if c2d.is_null() {
        return Ok(None);
    }
    let adaptor = Geom2dAdaptorCurve::with_curve(&c2d)?;
    let parameter = if pivot.orientation != edge.orientation() {
        first
    } else {
        last
    };
    choice.last_parameter = parameter;
    choice.last_point = adaptor.value(parameter)?;
    if !is_distance_in_2d_tolerance(surface, &choice.last_point, &pivot.point, tol3d)? {
        return Ok(None);
    }
    let (point, mut direction) = curve_dir_for_parameter(&adaptor, parameter)?;
    choice.last_point = point;
    if pivot.orientation == edge.orientation() {
        direction = direction.reversed();
    }
    let resolution: f64 = gp::resolution();
    if pivot.direction.magnitude() <= resolution || direction.magnitude() <= resolution {
        return Ok(None);
    }
    let mut angle: f64 = -pivot.direction.angle(direction)?;
    if angle < 0.0 {
        angle += 2.0 * PI;
    }
    Ok(Some(angle))
}

/// The kept position: the chosen candidate, else the single candidate when it meets
/// the pivot in 2d and 3d (and neither edge is degenerated), else none.
fn choose(
    pivot: &Pivot,
    choice: &Choice,
    (edge, face): (&Edge, &Face),
    surface: &BRepAdaptorSurface,
    (tol3d, list): (f64, &List<Shape>),
) -> Result<Option<usize>, Failure> {
    if choice.index_min != 0 {
        return Ok(Some(choice.index_min));
    }
    if list.extent() != 1 {
        return Ok(None);
    }
    let found = Edge::unchecked(list.first().expect("one candidate").clone());
    if found.is_null() || BRepTool::degenerated(edge)? || BRepTool::degenerated(&found)? {
        return Ok(None);
    }
    if !is_distance_in_2d_tolerance(surface, &choice.last_point, &pivot.point, tol3d)? {
        return Ok(None);
    }
    // Closure in 3d.
    let on_edge = BRepAdaptorCurve::with_edge_on_face(edge, face)?.value(pivot.parameter)?;
    let on_found =
        BRepAdaptorCurve::with_edge_on_face(&found, face)?.value(choice.last_parameter)?;
    if is_distance_in_3d_tolerance(&on_edge, &on_found, tol3d) {
        return Ok(Some(1));
    }
    Ok(None)
}

/// Keeps only the item at 1-based `index` of `list`.
fn keep_only(list: &mut List<Shape>, index: usize) {
    let kept: Vec<Shape> = list.iter().skip(index - 1).take(1).cloned().collect();
    list.clear();
    for shape in kept {
        list.append(shape);
    }
}
