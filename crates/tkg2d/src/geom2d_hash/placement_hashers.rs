// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG2d/Geom2dHash/Geom2dHash_PointHasher.pxx, Geom2dHash_DirectionHasher.pxx, Geom2dHash_AxisPlacement.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The foundational hashers: points, directions and `gp_Ax22d` placements. Hashes
//! round each coordinate to `HashTolerance`; equality compares each coordinate
//! within `CompTolerance`.
use super::hash_key::{combined_hash, rounded_hash};
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkmath::gp::{Axis22d, Direction2, Point2, Xy};

hasher_tolerances!(
    /// `Geom2dHash_PointHasher`: hashes and compares `gp_Pnt2d` values;
    /// `Geom2dHashPointHasher::default().hash(Point2::new(1.0, 2.0))`.
    Geom2dHashPointHasher
);
hasher_tolerances!(
    /// `Geom2dHash_DirectionHasher`: hashes and compares `gp_Dir2d` values;
    /// `Geom2dHashDirectionHasher::default().hash(direction)`.
    Geom2dHashDirectionHasher
);
hasher_tolerances!(
    /// `Geom2dHash_AxisPlacement`: hashes and compares `gp_Ax22d` placements by location,
    /// X direction and Y direction; `Geom2dHashAxisPlacement::default().hash(frame)`.
    Geom2dHashAxisPlacement
);

// The coordinate pair hash and comparison that points and directions share.
fn xy_hash(xy: Xy, hash_tolerance: f64) -> usize {
    let factor: f64 = 1.0 / hash_tolerance;
    combined_hash(&[rounded_hash(xy.x(), factor), rounded_hash(xy.y(), factor)])
}
fn xy_equal(first: Xy, second: Xy, comp_tolerance: f64) -> bool {
    (first.x() - second.x()).abs() <= comp_tolerance
        && (first.y() - second.y()).abs() <= comp_tolerance
}

impl Geom2dHashPointHasher {
    /// The rounded-coordinate hash (`operator()(const gp_Pnt2d&)`); `hasher.hash(p)`.
    pub fn hash(&self, point: Point2) -> usize {
        xy_hash(point.xy(), self.hash_tolerance)
    }
    /// Coordinates within `comp_tolerance` (`operator()(const gp_Pnt2d&, const gp_Pnt2d&)`);
    /// `hasher.equal(p, q)`.
    pub fn equal(&self, first: Point2, second: Point2) -> bool {
        xy_equal(first.xy(), second.xy(), self.comp_tolerance)
    }
}
impl Geom2dHashDirectionHasher {
    /// The rounded-component hash (`operator()(const gp_Dir2d&)`); `hasher.hash(d)`.
    pub fn hash(&self, direction: Direction2) -> usize {
        xy_hash(direction.xy(), self.hash_tolerance)
    }
    /// Components within `comp_tolerance` (`operator()(const gp_Dir2d&, const gp_Dir2d&)`);
    /// `hasher.equal(d, e)`.
    pub fn equal(&self, first: Direction2, second: Direction2) -> bool {
        xy_equal(first.xy(), second.xy(), self.comp_tolerance)
    }
}
impl Geom2dHashAxisPlacement {
    fn parts(&self) -> (Geom2dHashPointHasher, Geom2dHashDirectionHasher) {
        (
            Geom2dHashPointHasher::new(self.comp_tolerance, self.hash_tolerance),
            Geom2dHashDirectionHasher::new(self.comp_tolerance, self.hash_tolerance),
        )
    }
    /// Location, X and Y direction hashes combined (`operator()(const gp_Ax22d&)`);
    /// `hasher.hash(frame)`.
    pub fn hash(&self, placement: Axis22d) -> usize {
        let (points, directions) = self.parts();
        combined_hash(&[
            points.hash(placement.location()),
            directions.hash(placement.x_direction()),
            directions.hash(placement.y_direction()),
        ])
    }
    /// Location and both directions equal (`operator()(const gp_Ax22d&, const gp_Ax22d&)`);
    /// `hasher.equal(a, b)`.
    pub fn equal(&self, first: Axis22d, second: Axis22d) -> bool {
        let (points, directions) = self.parts();
        points.equal(first.location(), second.location())
            && directions.equal(first.x_direction(), second.x_direction())
            && directions.equal(first.y_direction(), second.y_direction())
    }
}

impl HashPolicy<Point2> for Geom2dHashPointHasher {
    fn hash(&self, key: &Point2) -> usize {
        Geom2dHashPointHasher::hash(self, *key)
    }
    fn equal(&self, first: &Point2, second: &Point2) -> bool {
        Geom2dHashPointHasher::equal(self, *first, *second)
    }
}
impl HashPolicy<Direction2> for Geom2dHashDirectionHasher {
    fn hash(&self, key: &Direction2) -> usize {
        Geom2dHashDirectionHasher::hash(self, *key)
    }
    fn equal(&self, first: &Direction2, second: &Direction2) -> bool {
        Geom2dHashDirectionHasher::equal(self, *first, *second)
    }
}
impl HashPolicy<Axis22d> for Geom2dHashAxisPlacement {
    fn hash(&self, key: &Axis22d) -> usize {
        Geom2dHashAxisPlacement::hash(self, *key)
    }
    fn equal(&self, first: &Axis22d, second: &Axis22d) -> bool {
        Geom2dHashAxisPlacement::equal(self, *first, *second)
    }
}
