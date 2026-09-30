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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Tool2d.hxx/.cxx (the fields, the accessors, the points and the tangents) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT2d_Tool2d`: the geometry of the MAT algorithm: the items of the circuit and
//! the points, vectors and bisectors that the algorithm creates by index.
mod create;
mod intersect;
mod same_distance;
mod trim;

use super::Mat2dCircuit;
use crate::bisector::Bisec;
use crate::mat::MatSide;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dPoint, errors,
};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::{Point2, Vector2};
use std::collections::HashMap;

/// `MAT2d_TOLCONF`: the tolerance of confusion of the tool.
pub(super) const MAT2D_TOLCONF: f64 = 1.0e-7;

/// `MAT2d_Tool2d`: the geometry of the bisecting locus of a [`Mat2dCircuit`]. The
/// points, vectors and bisectors are numbered from 1 as they are created; a missing
/// number raises Standard_NoSuchObject.
/// ```
/// use rustycad_tktopalgo::mat2d::Mat2dTool2d;
/// let tool = Mat2dTool2d::new();
/// assert_eq!(tool.tolerance_of_confusion(), 2.0e-7);
/// assert!(tool.geom_pnt(1).is_err());
/// ```
pub struct Mat2dTool2d {
    the_direction: f64,
    the_join_type: JoinType,
    the_number_of_bisectors: i32,
    the_number_of_pnts: i32,
    the_number_of_vecs: i32,
    the_circuit: Handle<Mat2dCircuit>,
    the_geom_bisectors: HashMap<i32, Bisec>,
    the_geom_pnts: HashMap<i32, Point2>,
    the_geom_vecs: HashMap<i32, Vector2>,
}

impl Default for Mat2dTool2d {
    fn default() -> Self {
        Self::new()
    }
}

// A missing number of the maps of the tool.
fn missing(what: &str, index: i32) -> Failure {
    errors::no_such_object(&format!(
        "MAT2d_Tool2d: no {what} {index}; expected a created one"
    ))
}

// True for a point item of the circuit.
fn is_point(geometry: &Handle<dyn Geom2dGeometry>) -> bool {
    geometry.cast::<Geom2dCartesianPoint>().is_some()
}

// The curve of an item; a null handle is a native null dereference.
fn as_curve(geometry: &Handle<dyn Geom2dGeometry>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let curve: Handle<dyn Geom2dCurve> = geometry.cast();
    if curve.is_null() {
        return Err(errors::null_object(
            "MAT2d_Tool2d: the item is not a curve; expected a Geom2d_Curve",
        ));
    }
    Ok(curve)
}

impl Mat2dTool2d {
    /// `MAT2d_Tool2d()`: on the left, joins by arcs, nothing created;
    /// `Mat2dTool2d::new()`.
    pub fn new() -> Self {
        Self {
            the_direction: 1.,
            the_join_type: JoinType::Arc,
            the_number_of_bisectors: 0,
            the_number_of_pnts: 0,
            the_number_of_vecs: 0,
            the_circuit: Handle::default(),
            the_geom_bisectors: HashMap::new(),
            the_geom_pnts: HashMap::new(),
            the_geom_vecs: HashMap::new(),
        }
    }
    /// `Sense(aside)`: the side of the computation; `tool.sense(MatSide::Left)`.
    pub fn sense(&mut self, aside: MatSide) {
        self.the_direction = if aside == MatSide::Left { 1. } else { -1. };
    }
    /// `SetJoinType(aJoinType)`; `tool.set_join_type(JoinType::Arc)`.
    pub fn set_join_type(&mut self, a_join_type: JoinType) {
        self.the_join_type = a_join_type;
    }
    /// `InitItems(aCircuit)`: forgets what was created and takes the circuit;
    /// `tool.init_items(&circuit)`.
    pub fn init_items(&mut self, a_circuit: &Handle<Mat2dCircuit>) {
        self.the_geom_bisectors.clear();
        self.the_geom_pnts.clear();
        self.the_geom_vecs.clear();
        self.the_number_of_bisectors = 0;
        self.the_number_of_vecs = 0;
        self.the_number_of_pnts = 0;
        self.the_circuit = a_circuit.clone();
    }
    // The circuit; a null one is a native null dereference.
    fn circuit_ref(&self) -> Result<&Mat2dCircuit, Failure> {
        self.the_circuit
            .get()
            .ok_or_else(|| errors::null_object("MAT2d_Tool2d: no circuit; expected InitItems"))
    }
    /// `NumberOfItems()`: the items of the circuit; `tool.number_of_items()?`.
    pub fn number_of_items(&self) -> Result<i32, Failure> {
        Ok(self.circuit_ref()?.number_of_items())
    }
    /// `ToleranceOfConfusion()`: twice `MAT2d_TOLCONF`; `tool.tolerance_of_confusion()`.
    pub fn tolerance_of_confusion(&self) -> f64 {
        2. * MAT2D_TOLCONF
    }
    /// `FirstPoint(anitem, dist)`: a new point at the origin of the bisector between
    /// `anitem` and the item before it (the middle of their connexion, else the start
    /// of the item); it returns its number and its distance to `anitem`;
    /// `let (point, dist) = tool.first_point(2)?;`.
    pub fn first_point(&mut self, anitem: i32) -> Result<(i32, f64), Failure> {
        self.the_number_of_pnts += 1;
        let circuit = self.circuit_ref()?;
        if circuit.connexion_on(anitem) {
            let connexion = circuit.connexion(anitem)?;
            let (p1, p2) = (connexion.point_on_first(), connexion.point_on_second());
            let middle = Point2::new((p1.x() + p2.x()) * 0.5, (p1.y() + p2.y()) * 0.5);
            let dist: f64 = p1.distance(p2) * 0.5;
            self.the_geom_pnts.insert(self.the_number_of_pnts, middle);
            return Ok((self.the_number_of_pnts, dist));
        }
        let item: Handle<dyn Geom2dGeometry> = circuit.value(anitem)?;
        let point: Point2 = if is_point(&item) {
            item.cast::<dyn Geom2dPoint>().pnt2d()
        } else {
            let curve = as_curve(&item)?;
            curve.value(curve.first_parameter())?
        };
        self.the_geom_pnts.insert(self.the_number_of_pnts, point);
        Ok((self.the_number_of_pnts, 0.))
    }
    /// `TangentBefore(anitem, IsOpenResult)`: a new vector, the tangent at the end of
    /// `anitem` (or along the connexion that follows it); `tool.tangent_before(1, false)?`.
    pub fn tangent_before(&mut self, anitem: i32, is_open_result: bool) -> Result<i32, Failure> {
        self.the_number_of_vecs += 1;
        let circuit = self.circuit_ref()?;
        let nb: i32 = circuit.number_of_items();
        let item: i32 = match (is_open_result, anitem == nb) {
            (false, true) => 1,
            (true, true) => anitem - 1,
            _ => anitem + 1,
        };
        let vector: Vector2 = if circuit.connexion_on(item) {
            let connexion = circuit.connexion(item)?;
            Vector2::between(connexion.point_on_first(), connexion.point_on_second())
        } else if !is_point(&circuit.value(anitem)?) {
            let curve = as_curve(&circuit.value(anitem)?)?;
            curve.dn(curve.last_parameter(), 1)?
        } else {
            let curve = as_curve(&circuit.value(item)?)?;
            let at_end: bool = is_open_result && anitem == nb;
            let param: f64 = if at_end {
                curve.last_parameter()
            } else {
                curve.first_parameter()
            };
            curve.dn(param, 1)?
        };
        self.the_geom_vecs.insert(self.the_number_of_vecs, vector);
        Ok(self.the_number_of_vecs)
    }
    /// `TangentAfter(anitem, IsOpenResult)`: a new vector, the reversed tangent at the
    /// start of `anitem` (or along the connexion that arrives at it);
    /// `tool.tangent_after(2, false)?`.
    pub fn tangent_after(&mut self, anitem: i32, is_open_result: bool) -> Result<i32, Failure> {
        self.the_number_of_vecs += 1;
        let circuit = self.circuit_ref()?;
        if circuit.connexion_on(anitem) {
            let connexion = circuit.connexion(anitem)?;
            let vector = Vector2::between(connexion.point_on_second(), connexion.point_on_first());
            self.the_geom_vecs.insert(self.the_number_of_vecs, vector);
            return Ok(self.the_number_of_vecs);
        }
        let thevector: Vector2 = if !is_point(&circuit.value(anitem)?) {
            let curve = as_curve(&circuit.value(anitem)?)?;
            curve.dn(curve.first_parameter(), 1)?
        } else {
            let item: i32 = match (is_open_result, anitem == 1) {
                (false, true) => circuit.number_of_items(),
                (true, true) => 2,
                _ => anitem - 1,
            };
            let curve = as_curve(&circuit.value(item)?)?;
            let at_start: bool = is_open_result && anitem == 1;
            let param: f64 = if at_start {
                curve.first_parameter()
            } else {
                curve.last_parameter()
            };
            curve.dn(param, 1)?
        };
        self.the_geom_vecs
            .insert(self.the_number_of_vecs, thevector.reversed());
        Ok(self.the_number_of_vecs)
    }
    /// `Tangent(bisector)`: a new vector, the tangent at the end of the geometric
    /// bisector `bisector`; `tool.tangent(3)?`.
    pub fn tangent(&mut self, bisector: i32) -> Result<i32, Failure> {
        self.the_number_of_vecs += 1;
        let value = self.geom_bis(bisector)?.value();
        let vector: Vector2 = value.dn(value.last_parameter(), 1)?;
        self.the_geom_vecs.insert(self.the_number_of_vecs, vector);
        Ok(self.the_number_of_vecs)
    }
    /// `Dump(bisector, erease)`: Standard_NotImplemented outside the debug build;
    /// `tool.dump(1, 0)`.
    pub fn dump(&self, _bisector: i32, _erease: i32) -> Result<(), Failure> {
        Err(errors::not_implemented(
            "MAT2d_Tool2d::Dump: only in the debug build of OCCT",
        ))
    }
    /// `GeomBis(Index)`: the geometric bisector `index`; `tool.geom_bis(1)?`.
    pub fn geom_bis(&self, index: i32) -> Result<&Bisec, Failure> {
        self.the_geom_bisectors
            .get(&index)
            .ok_or_else(|| missing("bisector", index))
    }
    /// `ChangeGeomBis(Index)`: the geometric bisector `index`, to edit;
    /// `tool.change_geom_bis(1)?`.
    pub fn change_geom_bis(&mut self, index: i32) -> Result<&mut Bisec, Failure> {
        self.the_geom_bisectors
            .get_mut(&index)
            .ok_or_else(|| missing("bisector", index))
    }
    /// `GeomElt(Index)`: the item `index` of the circuit; `tool.geom_elt(1)?`.
    pub fn geom_elt(&self, index: i32) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        self.circuit_ref()?.value(index)
    }
    /// `GeomPnt(Index)`: the point `index`; `tool.geom_pnt(1)?`.
    pub fn geom_pnt(&self, index: i32) -> Result<Point2, Failure> {
        self.the_geom_pnts
            .get(&index)
            .copied()
            .ok_or_else(|| missing("point", index))
    }
    /// `GeomVec(Index)`: the vector `index`; `tool.geom_vec(1)?`.
    pub fn geom_vec(&self, index: i32) -> Result<Vector2, Failure> {
        self.the_geom_vecs
            .get(&index)
            .copied()
            .ok_or_else(|| missing("vector", index))
    }
    /// `Circuit()`; `tool.circuit()`.
    pub fn circuit(&self) -> Handle<Mat2dCircuit> {
        self.the_circuit.clone()
    }
}

/// `AreNeighbours(IEdge1, IEdge2, NbEdge)`: true when the two items are consecutive on
/// a closed contour of `nb_edge` items.
pub(super) fn are_neighbours(i_edge1: i32, i_edge2: i32, nb_edge: i32) -> bool {
    let gap: i32 = (i_edge1 - i_edge2).abs();
    gap == 1 || gap == nb_edge - 1
}
