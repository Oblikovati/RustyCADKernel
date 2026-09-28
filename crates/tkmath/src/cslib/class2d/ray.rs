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

//! Ray-casting over the normalized, explicitly closed CSLib_Class2d polygon.
use super::Class2dResult;
use crate::cslib::precision::P_CONFUSION;

/// Normalized vertices with the first vertex repeated at index `count`.
#[derive(Clone, Debug, Default)]
pub(super) struct NormalizedPolygon {
    pub(super) xs: Vec<f64>,
    pub(super) ys: Vec<f64>,
}

/// Crossing counter shared by both scans; it keeps the native previous-vertex state.
struct Crossings {
    count: i32,
    prev_dx: f64,
    prev_dy: f64,
    prev_y_negative: bool,
}

impl Crossings {
    fn start(dx: f64, dy: f64) -> Self {
        Self {
            count: 0,
            prev_dx: dx,
            prev_dy: dy,
            prev_y_negative: dy < 0.0,
        }
    }
    // One edge from the previous vertex to (dx,dy), both relative to the test point.
    fn advance(&mut self, dx: f64, dy: f64) {
        let y_negative: bool = dy < 0.0;
        if y_negative != self.prev_y_negative {
            if self.prev_dx > 0.0 && dx > 0.0 {
                self.count += 1;
            } else if self.prev_dx > 0.0 || dx > 0.0 {
                let x_intersect: f64 =
                    self.prev_dx - self.prev_dy * (dx - self.prev_dx) / (dy - self.prev_dy);
                if x_intersect > 0.0 {
                    self.count += 1;
                }
            }
            self.prev_y_negative = y_negative;
        }
        self.prev_dx = dx;
        self.prev_dy = dy;
    }
    fn is_inside(&self) -> bool {
        (self.count & 1) != 0
    }
}

impl NormalizedPolygon {
    /// Number of polygon vertices (the closing copy excluded).
    pub(super) fn count(&self) -> usize {
        self.xs.len().saturating_sub(1)
    }

    /// CSLib_Class2d::internalSiDans: odd crossings of the +X ray mean inside.
    pub(super) fn contains(&self, px: f64, py: f64) -> bool {
        let mut crossings: Crossings = Crossings::start(self.xs[0] - px, self.ys[0] - py);
        for next in 1..=self.count() {
            crossings.advance(self.xs[next] - px, self.ys[next] - py);
        }
        crossings.is_inside()
    }

    /// CSLib_Class2d::internalSiDansOuOn: the ray scan with vertex/edge ON detection.
    pub(super) fn classify_on(&self, px: f64, py: f64, tol_u: f64, tol_v: f64) -> Class2dResult {
        let mut crossings: Crossings = Crossings::start(self.xs[0] - px, self.ys[0] - py);
        for next in 1..=self.count() {
            let dx: f64 = self.xs[next] - px;
            let dy: f64 = self.ys[next] - py;
            if dx < tol_u && dx > -tol_u && dy < tol_v && dy > -tol_v {
                return Class2dResult::Uncertain; // ON boundary (at vertex)
            }
            if self.is_on_edge(next, px, py, tol_v) {
                return Class2dResult::Uncertain; // ON boundary (on edge)
            }
            crossings.advance(dx, dy);
        }
        if crossings.is_inside() {
            Class2dResult::Inside
        } else {
            Class2dResult::Outside
        }
    }

    // Interpolates the edge ending at `next` at the test X; nearly vertical edges
    // are skipped to avoid division instability (their vertices are tested above).
    fn is_on_edge(&self, next: usize, px: f64, py: f64, tol_v: f64) -> bool {
        let prev: usize = next - 1;
        let dx: f64 = self.xs[next] - px;
        let edge_dx: f64 = self.xs[next] - self.xs[prev];
        let straddles: bool = (self.xs[prev] - px) * dx < 0.0 && edge_dx.abs() > P_CONFUSION;
        if !straddles {
            return false;
        }
        let interp_y: f64 = self.ys[next] - (self.ys[next] - self.ys[prev]) / edge_dx * dx;
        let delta_y: f64 = interp_y - py;
        delta_y >= -tol_v && delta_y <= tol_v
    }
}
