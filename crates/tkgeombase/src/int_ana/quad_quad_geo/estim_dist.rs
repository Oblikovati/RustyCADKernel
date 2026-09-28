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
// Rust translation of the file-local EstimDist of IntAna_QuadQuadGeo.cxx and of the
// line case of ProjLib_Plane.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! EstimDist: the minimal distance from the apexes of two cones with parallel,
//! coplanar axes to their intersection.
use crate::gce::{MakePln, Root};
use crate::int_ana2d::AnaIntersection;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Cone3, Direction2, Line2, Line3, Plane3, Point2, Point3, Vector3};

/// EstimDist: in the plane of the two axes, the generatrices of both cones
/// (the axes rotated by ± the semi-angle) are intersected pairwise; the result
/// is the larger of the smallest |parameter| on each cone, or
/// Precision::Infinite() when the apexes and the axis do not span a plane.
pub(super) fn estim_dist(first: Cone3, second: Cone3) -> Result<f64, Failure> {
    let (apex1, apex2) = (first.apex(), second.apex());
    let third: Point3 = apex1.translated(Vector3::from(first.position().direction()));
    let maker = MakePln::through_points(apex1, apex2, third)?;
    if !maker.is_done() {
        return Ok(precision::infinite());
    }
    let plane: Plane3 = maker.value()?;
    let lines1 = generatrices(
        plane,
        Line3::new(apex1, first.position().direction()),
        first.semi_angle(),
    )?;
    let lines2 = generatrices(
        plane,
        Line3::new(apex2, second.position().direction()),
        second.semi_angle(),
    )?;
    let mut min_dist: [f64; 2] = [precision::infinite(), precision::infinite()];
    let mut inter = AnaIntersection::new();
    for line1 in lines1 {
        for line2 in lines2 {
            inter.perform_lines(line1, line2);
            if !inter.is_done() {
                continue;
            }
            for k in 1..=inter.nb_points()? {
                let point = inter.point(k)?;
                min_dist[0] = std_min(point.param_on_first().abs(), min_dist[0]);
                min_dist[1] = std_min(point.param_on_second()?.abs(), min_dist[1]);
            }
        }
    }
    Ok(std_max(min_dist[0], min_dist[1]))
}

/// The two generatrices of a cone in `plane`: the projected axis rotated by
/// `angle` and by `-angle` around its location.
fn generatrices(plane: Plane3, axis: Line3, angle: f64) -> Result<[Line2; 2], Failure> {
    let axis2d: Line2 = project_line(plane, axis)?;
    Ok([
        axis2d.rotated(axis2d.location(), angle),
        axis2d.rotated(axis2d.location(), -angle),
    ])
}

/// ProjLib::Project(gp_Pln, gp_Lin) (ProjLib_Plane::Project(gp_Lin)): the line
/// in the plane coordinates. ProjLib is not ported yet; this is the one line case
/// EstimDist needs, in the native operation order, until the ProjLib port
/// replaces it (like line_extrema.rs for Extrema).
fn project_line(plane: Plane3, line: Line3) -> Result<Line2, Failure> {
    let (x_dir, y_dir) = (
        plane.position().x_direction(),
        plane.position().y_direction(),
    );
    let op = Vector3::between(plane.location(), line.location());
    let origin = Point2::new(op.dot(Vector3::from(x_dir)), op.dot(Vector3::from(y_dir)));
    let direction = Direction2::new(line.direction().dot(x_dir), line.direction().dot(y_dir))?;
    Ok(Line2::new(origin, direction))
}
