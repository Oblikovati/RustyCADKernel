// Copyright (c) 2026 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/GeomHash/GeomHash_PolygonOnTriHasher.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomHash_PolygonOnTriHasher` and its key `PolygonOnTriHashKey`: a polygon on a
//! triangulation is only meaningful together with the triangulation it indexes, so
//! the key pairs the polygon with an identifier of that triangulation.
use super::polygon_hashers::write_rounded;
use rustycad_tkernel::ncollection::HashPolicy;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_hash::hash_key::{combined_hash, int_hash, int64_hash, rounded_hash};
use rustycad_tkg2d::hasher_tolerances;
use rustycad_tkmath::poly::PolygonOnTriangulation;

/// `PolygonOnTriHashKey`: a polygon on triangulation (possibly null) and the
/// identifier of the triangulation representation it belongs to;
/// `PolygonOnTriHashKey { poly, tri_rep_id: 7 }`.
#[derive(Clone)]
pub struct PolygonOnTriHashKey {
    /// The polygon (`Poly`).
    pub poly: Handle<PolygonOnTriangulation>,
    /// The triangulation representation identifier (`TriRepId`).
    pub tri_rep_id: u32,
}

hasher_tolerances!(
    /// `GeomHash_PolygonOnTriHasher`: hashes and compares [`PolygonOnTriHashKey`] values;
    /// `GeomHashPolygonOnTriHasher::default().hash(&key)`.
    GeomHashPolygonOnTriHasher,
    precision::computational(),
    precision::computational()
);

impl GeomHashPolygonOnTriHasher {
    /// The identifier hash, then either the null marker `hash(1)` or the node count,
    /// parameter presence, deflection, end node indices and rounded end parameters,
    /// each in its own zero-initialized slot (`operator()(const PolygonOnTriHashKey&)`);
    /// `hasher.hash(&key)`.
    pub fn hash(&self, key: &PolygonOnTriHashKey) -> usize {
        let mut slots = [0usize; 9];
        // The identifier is widened to int64_t before hashing, as natively.
        slots[0] = int64_hash(i64::from(key.tri_rep_id));
        let Some(poly) = key.poly.get() else {
            slots[1] = int_hash(1);
            return combined_hash(&slots);
        };
        // Indices 1 and NbNodes() always exist, so the accessors cannot fail here.
        self.fill_polygon_slots(poly, &mut slots).unwrap_or(());
        combined_hash(&slots)
    }
    fn fill_polygon_slots(
        &self,
        poly: &PolygonOnTriangulation,
        slots: &mut [usize; 9],
    ) -> Result<(), Failure> {
        let factor: f64 = 1.0 / self.hash_tolerance;
        let count: i32 = poly.nb_nodes();
        slots[2] = int_hash(count);
        slots[3] = int_hash(i32::from(poly.has_parameters()));
        slots[4] = rounded_hash(poly.deflection(), factor);
        if count <= 0 {
            return Ok(());
        }
        slots[5] = int_hash(poly.node(1)?);
        slots[6] = int_hash(poly.node(count)?);
        if poly.has_parameters() {
            let ends = [poly.parameter(1)?, poly.parameter(count)?];
            write_rounded(&mut slots[7..9], &ends, factor);
        }
        Ok(())
    }

    /// Same identifier, then both polygons null, or the same node count, parameter
    /// presence, deflection, node indices and parameters within tolerance
    /// (`operator()(const PolygonOnTriHashKey&, const PolygonOnTriHashKey&)`);
    /// `hasher.equal(&a, &b)`.
    pub fn equal(&self, first: &PolygonOnTriHashKey, second: &PolygonOnTriHashKey) -> bool {
        if first.tri_rep_id != second.tri_rep_id {
            return false;
        }
        let (Some(a), Some(b)) = (first.poly.get(), second.poly.get()) else {
            return first.poly.is_null() && second.poly.is_null();
        };
        if a.nb_nodes() != b.nb_nodes() || a.has_parameters() != b.has_parameters() {
            return false;
        }
        if (a.deflection() - b.deflection()).abs() > self.comp_tolerance {
            return false;
        }
        self.same_nodes_and_parameters(a, b).unwrap_or(false)
    }
    fn same_nodes_and_parameters(
        &self,
        first: &PolygonOnTriangulation,
        second: &PolygonOnTriangulation,
    ) -> Result<bool, Failure> {
        for index in 1..=first.nb_nodes() {
            if first.node(index)? != second.node(index)? {
                return Ok(false);
            }
        }
        if !first.has_parameters() {
            return Ok(true);
        }
        for index in 1..=first.nb_nodes() {
            let gap: f64 = (first.parameter(index)? - second.parameter(index)?).abs();
            if gap > self.comp_tolerance {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl HashPolicy<PolygonOnTriHashKey> for GeomHashPolygonOnTriHasher {
    fn hash(&self, key: &PolygonOnTriHashKey) -> usize {
        GeomHashPolygonOnTriHasher::hash(self, key)
    }
    fn equal(&self, first: &PolygonOnTriHashKey, second: &PolygonOnTriHashKey) -> bool {
        GeomHashPolygonOnTriHasher::equal(self, first, second)
    }
}
