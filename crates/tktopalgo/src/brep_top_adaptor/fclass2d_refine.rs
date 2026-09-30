// Created on: 1995-03-22
// Created by: Laurent BUCHARD
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
// Rust translation of BRepTopAdaptor_FClass2d.cxx (polygon refinement) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The refinement of a wire polygon in `BRepTopAdaptor_FClass2d`: while the polygon
//! deflection exceeds the expected thickness of the wire (twice its area over its
//! perimeter), the wire is discretized again with GCPnts_QuasiUniformDeflection at a
//! tighter deflection, so that the polygon cannot self-intersect.
use super::fclass2d::BRepTopAdaptorFClass2d;
use super::fclass2d_polygon::{WirePolygon, triple_deflection};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topods::{Edge, Face, Wire};
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point2;

/// `PolygonMetrics`: the signed area and the perimeter of a closed polygon.
#[derive(Clone, Copy, Default)]
pub(super) struct PolygonMetrics {
    pub(super) area: f64,
    pub(super) perimeter: f64,
}

/// The points and deflections of one refinement pass.
struct RefinedPolygon {
    points: LinearVector<Point2>,
    fleche_u: f64,
    fleche_v: f64,
}

impl WirePolygon {
    /// The refinement loop; the signed area of the final polygon, None when the
    /// expected thickness of the first polygon is not representable (a bad wire).
    pub(super) fn refine(
        &mut self,
        classifier: &BRepTopAdaptorFClass2d,
        wire: &Wire,
    ) -> Result<Option<f64>, Failure> {
        let mut metrics: PolygonMetrics = polygon_metrics(&self.points);
        let Some(mut exp_thick) = expected_thickness(&metrics) else {
            return Ok(None);
        };
        let mut defl: f64 = std_max(self.fleche_u, self.fleche_v);
        let mut discr_defl: f64 = std_min(defl * 0.1, exp_thick * 10.0);
        while defl > exp_thick && discr_defl > 1e-7 {
            // Deflection of the polygon is too much for this ratio of area and perimeter,
            // and this might lead to self-intersections.
            // Build tighter samples separately so a failed edge leaves the coarse polygon intact.
            let Some(refined) = refined_polygon(&classifier.face, wire, discr_defl)? else {
                break;
            };
            let refined_metrics: PolygonMetrics = polygon_metrics(&refined.points);
            if refined.points.len() <= 3 {
                break;
            }
            let Some(refined_thickness) = expected_thickness(&refined_metrics) else {
                break;
            };
            self.nbpnts = refined.points.len() as i32;
            self.points = refined.points;
            (self.fleche_u, self.fleche_v) = (refined.fleche_u, refined.fleche_v);
            metrics = refined_metrics;
            exp_thick = refined_thickness;
            defl = std_max(self.fleche_u, self.fleche_v);
            discr_defl = std_min(discr_defl * 0.1, exp_thick * 10.0);
        }
        Ok(Some(metrics.area))
    }
}

/// One refinement pass over the edges of `wire`; None when a discretization fails.
fn refined_polygon(
    face: &Face,
    wire: &Wire,
    discr_defl: f64,
) -> Result<Option<RefinedPolygon>, Failure> {
    let mut refined = RefinedPolygon {
        points: LinearVector::default(),
        fleche_u: 0.0,
        fleche_v: 0.0,
    };
    let mut refined_first_point: i32 = 1;
    let mut explorer = WireExplorer::with_wire_on_face(wire, face)?;
    while explorer.more() {
        let edge: Edge = explorer.current().clone();
        let orientation: Orientation = edge.orientation();
        if orientation == Orientation::Forward || orientation == Orientation::Reversed {
            let (pfbid, plbid) = BRepTool::range_on_face(&edge, face)?;
            if (plbid - pfbid).abs() >= 1.0e-9 {
                let args = (face, discr_defl, refined_first_point);
                if !refined.add_edge(&edge, orientation, args)? {
                    return Ok(None);
                }
                refined_first_point = 2;
            }
        }
        explorer.next()?;
    }
    Ok(Some(refined))
}

impl RefinedPolygon {
    /// The discretization points of one edge in its orientation (without the first
    /// one after the first edge) and the deflection of its last triple; false when
    /// GCPnts_QuasiUniformDeflection is not done.
    fn add_edge(
        &mut self,
        edge: &Edge,
        orientation: Orientation,
        (face, discr_defl, first_point): (&Face, f64, i32),
    ) -> Result<bool, Failure> {
        let curve = BRepAdaptorCurve2d::with_edge_on_face(edge, face)?;
        let c: &dyn Adaptor2dCurve2d = &curve;
        let discr = QuasiUniformDeflection::with_curve(c, discr_defl, Shape::C1)?;
        if !discr.is_done() {
            return Ok(false);
        }
        let nbp: i32 = discr.nb_points()?;
        let (step, mut i, end) = if orientation == Orientation::Reversed {
            (-1, nbp, 0)
        } else {
            (1, 1, nbp + 1)
        };
        if first_point == 2 {
            i += step;
        }
        while i != end {
            self.points.append(c.value(discr.parameter(i)?)?);
            i += step;
        }
        if nbp > 2 {
            let (du, dv) = triple_deflection(&self.points, self.points.len())?;
            self.fleche_u = if du > self.fleche_u {
                du
            } else {
                self.fleche_u
            };
            self.fleche_v = if dv > self.fleche_v {
                dv
            } else {
                self.fleche_v
            };
        }
        Ok(true)
    }
}

/// `polygonMetrics`: the signed area (trapezoids) and the perimeter of the closed
/// polygon whose last point repeats the first; zero below two points.
pub(super) fn polygon_metrics(points: &LinearVector<Point2>) -> PolygonMetrics {
    let mut metrics = PolygonMetrics::default();
    if points.len() < 2 {
        return metrics;
    }
    let last_unique: usize = points.len() - 2;
    let mut previous: usize = last_unique;
    for current in 0..=last_unique {
        let (current_point, previous_point) = (points[current], points[previous]);
        metrics.area += (current_point.x() - previous_point.x())
            * (current_point.y() + previous_point.y())
            * 0.5;
        metrics.perimeter += (current_point.xy() - previous_point.xy()).modulus();
        previous = current;
    }
    metrics
}

/// `expectedThickness`: `max(2 |area| / perimeter, 1e-7)`, None when a metric or
/// the result is not representable or the perimeter is not positive.
fn expected_thickness(metrics: &PolygonMetrics) -> Option<f64> {
    if !is_representable_value(metrics.area)
        || !is_representable_value(metrics.perimeter)
        || metrics.perimeter <= 0.0
    {
        return None;
    }
    let thickness: f64 = std_max(2.0 * (metrics.area.abs() / metrics.perimeter), 1.0e-7);
    is_representable_value(thickness).then_some(thickness)
}

/// `isRepresentableValue`: finite (OCCT's infinity sentinels included), not NaN.
fn is_representable_value(value: f64) -> bool {
    (-f64::MAX..=f64::MAX).contains(&value)
}
