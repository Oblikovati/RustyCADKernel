// Created on: 1991-06-24
// Created by: Didier PIFFAULT
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
// Rust translation of Intf_InterferencePolygon2d::Intersect of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_InterferencePolygon2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_InterferencePolygon2d::Intersect: the section points of two segments, from
//! their vertex contacts, their crossing and, for near-parallel segments, the ends of
//! the interference radius; two or more points make a tangent zone.
use super::InterferencePolygon2d;
use super::radius::{RadiusEnds, radius_candidates};
use crate::intf::section_point::{PIType, SectionPoint};
use crate::intf::tangent_zone::TangentZone;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::gp::{Point2, Xy};

/// The two segments of one Intersect call, with the lengths and the angle sine.
#[derive(Clone, Copy, Debug)]
pub(super) struct SegmentPair {
    pub(super) io: i32,
    pub(super) it: i32,
    pub(super) beg_o: Point2,
    pub(super) end_o: Point2,
    pub(super) beg_t: Point2,
    pub(super) end_t: Point2,
    pub(super) seg_o: Xy,
    pub(super) lg_o: f64,
    pub(super) lg_t: f64,
    pub(super) sin_teta: f64,
}

impl SegmentPair {
    /// The point of the first segment at parameter `par`.
    pub(super) fn on_first(&self, par: f64) -> Point2 {
        Point2::new(
            self.beg_o.x() + (self.seg_o.x() * par),
            self.beg_o.y() + (self.seg_o.y() * par),
        )
    }

    /// A 2d section point of this pair.
    fn point(
        &self,
        at: Point2,
        first: (PIType, i32, f64),
        second: (PIType, i32, f64),
    ) -> SectionPoint {
        SectionPoint::with_data_2d(
            at,
            first.0,
            first.1,
            first.2,
            second.0,
            second.1,
            second.2,
            self.sin_teta,
        )
    }

    /// The EDGE/EDGE point where the segments cross.
    fn crossing(&self, par_o: f64, par_t: f64) -> SectionPoint {
        self.point(
            self.on_first(par_o),
            (PIType::Edge, self.io, par_o),
            (PIType::Edge, self.it, par_t),
        )
    }

    /// An EXTERNAL point at the end of the interference radius.
    pub(super) fn external(&self, par_o: f64, par_t: f64) -> SectionPoint {
        self.point(
            self.on_first(par_o),
            (PIType::External, self.io, par_o),
            (PIType::External, self.it, par_t),
        )
    }
}

/// The candidate section points with their parameters, indexed from 1 as the native
/// `parO[8]`/`parT[8]` arrays (the sequence and the arrays go out of step once a close
/// point is suppressed, as natively).
pub(super) struct Candidates {
    pub(super) points: Vec<SectionPoint>,
    pub(super) par_o: [f64; 8],
    pub(super) par_t: [f64; 8],
    pub(super) nbpi: usize,
}

impl Candidates {
    pub(super) fn push(&mut self, par_o: f64, par_t: f64, point: SectionPoint) {
        self.nbpi += 1;
        self.par_o[self.nbpi] = par_o;
        self.par_t[self.nbpi] = par_t;
        self.points.push(point);
    }

    pub(super) fn at(&self, index: usize) -> SectionPoint {
        self.points[index - 1]
    }
}

/// The signed distances of each segment end to the other segment's line.
#[derive(Clone, Copy, Debug)]
struct LineDistances {
    db_ot: f64,
    de_ot: f64,
    db_to: f64,
    de_to: f64,
}

impl InterferencePolygon2d {
    /// Intersect(iO, iT, BegO, EndO, BegT, EndT): the interference of segment `io` of
    /// the first polygon with segment `it` of the second. In a self interference,
    /// neighbouring segments are skipped (15 Jan 1998); null segments give nothing.
    pub(super) fn intersect(&mut self, index: [i32; 2], first: [Point2; 2], second: [Point2; 2]) {
        if self.base.self_interference && (index[0] - index[1]).abs() <= 1 {
            return;
        }
        let seg_t: Xy = second[1].xy() - second[0].xy();
        let seg_o: Xy = first[1].xy() - first[0].xy();
        let lg_t: f64 = (seg_t * seg_t).sqrt();
        if lg_t <= 0.0 {
            return;
        }
        let lg_o: f64 = (seg_o * seg_o).sqrt();
        if lg_o <= 0.0 {
            return;
        }
        let pair = SegmentPair {
            io: index[0],
            it: index[1],
            beg_o: first[0],
            end_o: first[1],
            beg_t: second[0],
            end_t: second[1],
            seg_o,
            lg_o,
            lg_t,
            sin_teta: (seg_o.cross_magnitude(seg_t) / lg_o) / lg_t,
        };
        self.intersect_pair(&pair, seg_t);
    }

    fn intersect_pair(&mut self, pair: &SegmentPair, seg_t: Xy) {
        let mut candidates = Candidates {
            points: Vec::new(),
            par_o: [0.0; 8],
            par_t: [0.0; 8],
            nbpi: 0,
        };
        let distances: LineDistances = self.vertex_candidates(pair, seg_t, &mut candidates);
        let Some(crossing) = self.crossing_candidates(pair, seg_t, distances, &mut candidates)
        else {
            return;
        };
        suppress_close_points(pair, &mut candidates);
        if candidates.nbpi == 1 {
            self.record_point(pair, &mut candidates, crossing);
        } else if candidates.nbpi >= 2 {
            self.record_zone(pair, &candidates, crossing);
        }
    }

    /// The contacts of each segment end with the other segment, as vertex points.
    fn vertex_candidates(
        &self,
        pair: &SegmentPair,
        seg_t: Xy,
        candidates: &mut Candidates,
    ) -> LineDistances {
        let tolerance: f64 = self.base.tolerance;
        let db_ot: f64 = ((pair.beg_o.xy() - pair.beg_t.xy()) ^ seg_t) / pair.lg_t;
        let (db_obt, db_oet) = (
            pair.beg_o.distance(pair.beg_t),
            pair.beg_o.distance(pair.end_t),
        );
        if db_ot.abs() <= tolerance {
            end_of_first_on_second(
                pair,
                tolerance,
                (pair.beg_o, pair.io, 0.0),
                (db_obt, db_oet),
                candidates,
            );
        }
        let de_ot: f64 = ((pair.end_o.xy() - pair.beg_t.xy()) ^ seg_t) / pair.lg_t;
        let (de_obt, de_oet) = (
            pair.end_o.distance(pair.beg_t),
            pair.end_o.distance(pair.end_t),
        );
        if de_ot.abs() <= tolerance {
            end_of_first_on_second(
                pair,
                tolerance,
                (pair.end_o, pair.io + 1, 1.0),
                (de_obt, de_oet),
                candidates,
            );
        }
        let db_to: f64 = ((pair.beg_t.xy() - pair.beg_o.xy()) ^ pair.seg_o) / pair.lg_o;
        if db_to.abs() <= tolerance {
            end_of_second_on_first(
                pair,
                tolerance,
                (pair.beg_t, pair.it, 0.0),
                (db_obt, de_obt),
                candidates,
            );
        }
        let de_to: f64 = ((pair.end_t.xy() - pair.beg_o.xy()) ^ pair.seg_o) / pair.lg_o;
        if de_to.abs() <= tolerance {
            end_of_second_on_first(
                pair,
                tolerance,
                (pair.end_t, pair.it + 1, 1.0),
                (db_oet, de_oet),
                candidates,
            );
        }
        LineDistances {
            db_ot,
            de_ot,
            db_to,
            de_to,
        }
    }

    /// The crossing of the lines: returns the crossing parameters when the segments
    /// really cross (edgeSP), a default when they do not, and None when the segments
    /// neither cross nor touch (the native early return). Adds the crossing point when
    /// nothing touched and the angle is not small, or the ends of the interference
    /// radius for near-parallel segments.
    fn crossing_candidates(
        &self,
        pair: &SegmentPair,
        seg_t: Xy,
        d: LineDistances,
        candidates: &mut Candidates,
    ) -> Option<Crossing> {
        let floatgap: f64 = epsilon(pair.lg_o + pair.lg_t);
        // Written as the native positive test, so that NaN distances skip the crossing.
        let separated =
            (d.db_ot - d.de_ot).abs() > floatgap && (d.db_to - d.de_to).abs() > floatgap;
        if !separated {
            return Some(Crossing::default());
        }
        let crossing = Crossing {
            edge: d.db_ot * d.de_ot <= 0.0 && d.db_to * d.de_to <= 0.0,
            par_o: d.db_ot / (d.db_ot - d.de_ot),
            par_t: d.db_to / (d.db_to - d.de_to),
        };
        if !crossing.edge && candidates.nbpi == 0 {
            return None;
        }
        let tolerance: f64 = self.base.tolerance;
        let ray_intf: f64 = if pair.sin_teta > 0.0 {
            tolerance / pair.sin_teta
        } else {
            0.0
        };
        if candidates.nbpi == 0 && pair.sin_teta > precision::angular() {
            candidates.push(
                crossing.par_o,
                crossing.par_t,
                pair.crossing(crossing.par_o, crossing.par_t),
            );
        } else if ray_intf >= tolerance {
            // Direction of parsing of the segments.
            let sig_ps: f64 = if (pair.seg_o * seg_t) > 0.0 {
                1.0
            } else {
                -1.0
            };
            radius_candidates(
                pair,
                tolerance,
                RadiusEnds::new(pair, crossing, ray_intf, sig_ps),
                candidates,
            );
        }
        Some(crossing)
    }

    /// One point: the crossing replaces it when the segments cross; it is recorded once
    /// (a self interference skips the adjacent segments of a closed polygon).
    fn record_point(
        &mut self,
        pair: &SegmentPair,
        candidates: &mut Candidates,
        crossing: Crossing,
    ) {
        if crossing.edge {
            candidates.points[0] = pair.crossing(crossing.par_o, crossing.par_t);
            candidates.par_o[1] = crossing.par_o;
            candidates.par_t[1] = crossing.par_t;
        }
        let point: SectionPoint = candidates.at(1);
        if !self.base.self_interference {
            if !self
                .base
                .section_points
                .iter()
                .any(|known| point.is_equal(known))
            {
                self.base.section_points.append(point);
            }
            return;
        }
        let closed_neighbours = self.o_clos && (pair.io == 1 || pair.it == self.nbso);
        if pair.it - pair.io != 1 && !closed_neighbours {
            self.base.section_points.append(point);
        }
    }

    /// Two or more points: a tangent zone of the extreme points (and the crossing),
    /// merged with every known zone it overlaps.
    fn record_zone(&mut self, pair: &SegmentPair, candidates: &Candidates, crossing: Crossing) {
        let mut zone = TangentZone::new();
        for index in zone_point_indices(candidates) {
            zone.polygon_insert(candidates.at(index));
        }
        if crossing.edge {
            zone.polygon_insert(pair.crossing(crossing.par_o, crossing.par_t));
        }
        let overlapping: Vec<usize> = (1..=self.base.tangent_zones.len())
            .filter(|&ltz| zone.has_common_range(&self.base.tangent_zones[ltz]))
            .collect();
        let Some((&first, rest)) = overlapping.split_first() else {
            self.base.tangent_zones.append(zone);
            return;
        };
        // The list is parsed in ascending order by index.
        self.base.tangent_zones[first].append_zone(&zone);
        for (decal, &index) in rest.iter().enumerate() {
            let merged: TangentZone = self.base.tangent_zones[index - decal].clone();
            self.base.tangent_zones[first].append_zone(&merged);
            // The index is valid: it was collected from this sequence.
            let _ = self.base.tangent_zones.remove(index - decal);
        }
    }
}

/// Where the lines cross: whether inside both segments (edgeSP) and the parameters.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Crossing {
    pub(super) edge: bool,
    pub(super) par_o: f64,
    pub(super) par_t: f64,
}

/// A segment end `(point, index, parameter)` of the first segment near the second's
/// line, with its distances to the second's ends: vertex-vertex or vertex-edge points.
fn end_of_first_on_second(
    pair: &SegmentPair,
    tolerance: f64,
    end: (Point2, i32, f64),
    to_ends: (f64, f64),
    candidates: &mut Candidates,
) {
    let (point, index, par) = end;
    let (to_begin, to_end) = to_ends;
    if to_begin <= tolerance {
        candidates.push(
            par,
            0.0,
            pair.point(
                point,
                (PIType::Vertex, index, 0.0),
                (PIType::Vertex, pair.it, 0.0),
            ),
        );
    }
    if to_end <= tolerance {
        candidates.push(
            par,
            1.0,
            pair.point(
                point,
                (PIType::Vertex, index, 0.0),
                (PIType::Vertex, pair.it + 1, 0.0),
            ),
        );
    }
    if to_begin > tolerance && to_end > tolerance && to_begin + to_end <= (pair.lg_t + tolerance) {
        let par_t: f64 = to_begin / pair.lg_t;
        candidates.push(
            par,
            par_t,
            pair.point(
                point,
                (PIType::Vertex, index, 0.0),
                (PIType::Edge, pair.it, par_t),
            ),
        );
    }
}

/// A segment end `(point, index, parameter)` of the second segment near the first's
/// line, with the first's ends' distances to it: an edge-vertex point.
fn end_of_second_on_first(
    pair: &SegmentPair,
    tolerance: f64,
    end: (Point2, i32, f64),
    from_ends: (f64, f64),
    candidates: &mut Candidates,
) {
    let (point, index, par) = end;
    let (from_begin, from_end) = from_ends;
    if from_begin > tolerance
        && from_end > tolerance
        && from_begin + from_end <= (pair.lg_o + tolerance)
    {
        let par_o: f64 = from_begin / pair.lg_o;
        candidates.push(
            par_o,
            par,
            pair.point(
                point,
                (PIType::Edge, pair.io, par_o),
                (PIType::Vertex, index, 0.0),
            ),
        );
    }
}

/// The points too close to each other are suspended (lbr): the later point of a pair
/// closer than 1/50 of both segment lengths is removed, shifting the points but not
/// their parameters, as natively.
fn suppress_close_points(pair: &SegmentPair, candidates: &mut Candidates) {
    'restart: loop {
        for i in 2..=candidates.nbpi {
            let d: f64 = candidates.at(i).pnt().distance(&candidates.at(i - 1).pnt()) * 50.0;
            if d < pair.lg_t && d < pair.lg_o {
                for j in i..candidates.nbpi {
                    candidates.points[j - 1] = candidates.points[j];
                }
                candidates.nbpi -= 1;
                continue 'restart;
            }
        }
        return;
    }
}

/// The candidates that make the tangent zone: both of two points; else the extreme
/// first parameters, then the extreme second parameters when they differ from them.
fn zone_point_indices(candidates: &Candidates) -> Vec<usize> {
    let n: usize = candidates.nbpi;
    if n == 2 {
        return vec![1, 2];
    }
    let (lmin, lmax) = extreme_indices(&candidates.par_o, n);
    let (ltmin, ltmax) = extreme_indices(&candidates.par_t, n);
    let mut indices = vec![lmin, lmax];
    if ltmin != lmin && ltmin != lmax {
        indices.push(ltmin);
    }
    if ltmax != lmin && ltmax != lmax {
        indices.push(ltmax);
    }
    indices
}

/// The native min/max index scan (`else if`: an index is never both).
fn extreme_indices(values: &[f64; 8], n: usize) -> (usize, usize) {
    let (mut lmin, mut lmax) = (1, 1);
    for lpj in 2..=n {
        if values[lpj] < values[lmin] {
            lmin = lpj;
        } else if values[lpj] > values[lmax] {
            lmax = lpj;
        }
    }
    (lmin, lmax)
}
