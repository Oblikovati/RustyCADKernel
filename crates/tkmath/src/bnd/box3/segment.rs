// Created on: 1991-03-08
// Created by: Christophe MARION
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
// Rust translation of FoundationClasses/TKMath/Bnd/Bnd_Box.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::BoundingBox3;
use super::{native_max, native_min};
use crate::gp::{Direction3, Point3};
impl BoundingBox3 {
    /// Tests the strip swept by a segment along a direction, as OCCT's three-argument IsOut does; `bounds.is_out_segment(first,second,direction)`.
    pub fn is_out_segment(&self, first: Point3, second: Point3, direction: Direction3) -> bool {
        if self.is_whole() {
            return false;
        }
        if self.is_void() {
            return true;
        }
        let bounds = self.get().expect("nonvoid bounds");
        swept_segment_out(
            bounds,
            *first.coordinates(),
            *second.coordinates(),
            *direction.coordinates(),
        )
    }
}
fn swept_segment_out(
    bounds: [f64; 6],
    first: [f64; 3],
    second: [f64; 3],
    direction: [f64; 3],
) -> bool {
    let zero = direction.map(|v| v.abs() < f64::MIN_POSITIVE);
    for (a, b) in [(0, 1), (0, 2), (1, 2)] {
        if zero[a] && zero[b] {
            return segment_out(
                [bounds[a], bounds[b], bounds[a + 3], bounds[b + 3]],
                [first[a], first[b]],
                [second[a], second[b]],
            );
        }
    }
    let faces = projection_faces(zero);
    faces.iter().all(|&(axis, upper, a, b)| {
        let face = bounds[axis + if upper { 3 } else { 0 }];
        let p = project_segment_end(first, direction, axis, face, a, b, zero);
        let q = project_segment_end(second, direction, axis, face, a, b, zero);
        segment_out([bounds[a], bounds[b], bounds[a + 3], bounds[b + 3]], p, q)
    })
}
fn projection_faces(zero: [bool; 3]) -> Vec<(usize, bool, usize, usize)> {
    if zero[0] {
        return vec![
            (1, false, 0, 2),
            (1, true, 0, 2),
            (2, false, 0, 1),
            (2, true, 0, 1),
        ];
    }
    if zero[1] {
        return vec![
            (0, false, 1, 2),
            (0, true, 1, 2),
            (2, false, 1, 0),
            (2, true, 1, 0),
        ];
    }
    if zero[2] {
        return vec![
            (1, true, 2, 0),
            (1, false, 2, 0),
            (0, true, 2, 1),
            (0, false, 2, 1),
        ];
    }
    vec![
        (1, false, 0, 2),
        (1, true, 0, 2),
        (2, false, 0, 1),
        (2, true, 0, 1),
        (0, false, 2, 1),
        (0, true, 2, 1),
    ]
}
fn project_segment_end(
    point: [f64; 3],
    direction: [f64; 3],
    axis: usize,
    face: f64,
    a: usize,
    b: usize,
    zero: [bool; 3],
) -> [f64; 2] {
    [a, b].map(|i| {
        if zero[i] {
            return point[i];
        }
        // The native zero-component branches multiply before dividing; general branches divide first.
        if zero.into_iter().any(|v| v) {
            return (face - point[axis]) * direction[i] / direction[axis] + point[i];
        }
        (face - point[axis]) / direction[axis] * direction[i] + point[i]
    })
}

fn axis_segment_inside(bounds: [f64; 4], first: [f64; 2], second: [f64; 2]) -> bool {
    let [x1, y1, x2, y2] = bounds;
    let [px, py] = first;
    let [qx, qy] = second;
    let [xmin, xmax, ymin, ymax] = [
        native_min(px, qx),
        native_max(px, qx),
        native_min(py, qy),
        native_max(py, qy),
    ];
    let eps = f64::MIN_POSITIVE;
    ymax - ymin < eps
        && y1 - py < eps
        && py - y2 < eps
        && ((xmin - x1 < eps && x1 - xmax < eps)
            || (xmin - x2 < eps && x2 - xmax < eps)
            || (x1 - px < eps && px - x2 < eps))
}
fn segment_out(bounds: [f64; 4], first: [f64; 2], second: [f64; 2]) -> bool {
    let [x1, y1, x2, y2] = bounds;
    let [px, py] = first;
    let [qx, qy] = second;
    if axis_segment_inside(bounds, first, second)
        || axis_segment_inside([y1, x1, y2, x2], [py, px], [qy, qx])
    {
        return false;
    }
    if (px < x1 && qx < x1) || (px > x2 && qx > x2) || (py < y1 && qy < y1) || (py > y2 && qy > y2)
    {
        return true;
    }
    if (qx - px).abs() > f64::MIN_POSITIVE {
        return segment_projection_out(bounds, first, second);
    }
    if (qy - py).abs() > f64::MIN_POSITIVE {
        return segment_projection_out([y1, x1, y2, x2], [py, px], [qy, qx]);
    }
    true
}
fn segment_projection_out(bounds: [f64; 4], first: [f64; 2], second: [f64; 2]) -> bool {
    let [x1, y1, x2, y2] = bounds;
    let [px, py] = first;
    let [qx, qy] = second;
    let a = (native_min(x1, x2) - px) * (qy - py) / (qx - px) + py;
    let b = (native_max(x1, x2) - px) * (qy - py) / (qx - px) + py;
    (a < y1 && b < y1) || (a > y2 && b > y2)
}
