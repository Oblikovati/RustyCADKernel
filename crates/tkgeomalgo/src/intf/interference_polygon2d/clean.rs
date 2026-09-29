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
// Rust translation of Intf_InterferencePolygon2d::Clean of ModelingAlgorithms/TKGeomAlgo/Intf/Intf_InterferencePolygon2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_InterferencePolygon2d::Clean: tangent zones that reduce to one point become
//! section points, and section points inside a tangent zone are dropped.
use super::InterferencePolygon2d;
use crate::intf::section_point::{PIType, SectionPoint};
use crate::intf::sequence_access::item;
use crate::intf::tangent_zone::TangentZone;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;

/// The points a zone of tangency offers as its single intersection: `tsp` (an
/// EDGE/EDGE point) and `tsps` (another point on no EXTERNAL element), 0 for none.
struct ZonePoints {
    tsp: i32,
    tsps: i32,
}

impl InterferencePolygon2d {
    /// Clean: first the zones, then the points.
    pub(super) fn clean(&mut self) -> Result<(), Failure> {
        self.reduce_zones()?;
        self.drop_points_in_zones()
    }

    /// The zones of tangency that concern only one couple of segments are replaced by
    /// one point when the segments make an angle above `Precision::Angular` and have
    /// no real EDGE/EDGE intersection. As natively, the "only one segment" flag is
    /// never reset between zones: once set it stays set, except where a second
    /// EDGE/EDGE point clears it.
    fn reduce_zones(&mut self) -> Result<(), Failure> {
        let count: i32 = self.base.tangent_zones.length();
        let mut decal: i32 = 0;
        let mut only_one_segment = false;
        for ltz in 1..=count {
            let zone: &TangentZone = item(&self.base.tangent_zones, ltz - decal)?;
            only_one_segment |= spans_one_segment(zone);
            let points: ZonePoints = zone_points(zone, &mut only_one_segment)?;
            let kept: i32 = if points.tsp > 0 {
                points.tsp
            } else if only_one_segment && points.tsps != 0 {
                points.tsps
            } else {
                continue;
            };
            let point: SectionPoint = *zone.get_point(kept)?;
            self.base.section_points.append(point);
            self.base.tangent_zones.remove((ltz - decal) as usize)?;
            decal += 1;
        }
        Ok(())
    }

    /// The points of intersection located in a tangency zone are removed.
    fn drop_points_in_zones(&mut self) -> Result<(), Failure> {
        let count: i32 = self.base.section_points.length();
        let mut decal: i32 = 0;
        for lpi in 1..=count {
            let point: SectionPoint = *item(&self.base.section_points, lpi - decal)?;
            if self
                .base
                .tangent_zones
                .iter()
                .any(|zone| zone.range_contains(&point))
            {
                self.base.section_points.remove((lpi - decal) as usize)?;
                decal += 1;
            }
        }
        Ok(())
    }
}

/// Whether both parameter ranges of the zone are shorter than one segment, or one of
/// them is null.
fn spans_one_segment(zone: &TangentZone) -> bool {
    let (first_min, first_max) = zone.param_on_first();
    let (second_min, second_max) = zone.param_on_second();
    let (delta1, delta2) = (first_max - first_min, second_max - second_min);
    (delta1 < 1.0 && delta2 < 1.0) || delta1 == 0.0 || delta2 == 0.0
}

/// The native scan of the zone's points: a point of small incidence cancels the
/// reduction; a second EDGE/EDGE point cancels it and clears the one-segment flag.
fn zone_points(zone: &TangentZone, only_one_segment: &mut bool) -> Result<ZonePoints, Failure> {
    let mut found = ZonePoints { tsp: 0, tsps: 0 };
    for lpi in 1..=zone.number_of_points() {
        let point: &SectionPoint = zone.get_point(lpi)?;
        if point.incidence() <= precision::angular() {
            return Ok(ZonePoints { tsp: 0, tsps: 0 });
        }
        let (dim1, dim2) = (point.info_first_address().0, point.info_second_address().0);
        if dim1 == PIType::Edge && dim2 == PIType::Edge {
            found.tsps = 0;
            if found.tsp > 0 {
                *only_one_segment = false;
                return Ok(ZonePoints { tsp: 0, tsps: 0 });
            }
            found.tsp = lpi;
        } else if dim1 != PIType::External && dim2 != PIType::External {
            found.tsps = lpi;
        }
    }
    Ok(found)
}
