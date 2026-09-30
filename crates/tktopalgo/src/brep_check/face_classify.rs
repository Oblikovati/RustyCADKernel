// Created on: 1995-12-15
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
// Rust translation of BRepCheck_Face::ClassifyWires, OrientationOfWires and their helpers (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Face.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Face::ClassifyWires` (one wire must hold all the others) and
//! `OrientationOfWires` (the outer wire forward, the holes reversed).
use super::errors::no_such_object;
use super::explore::sub_shapes;
use super::face::BRepCheckFace;
use super::face_wires::DataMapOfShapeBox2d;
use super::status::BRepCheckStatus;
use super::wire_closed::record;
use crate::brep_top_adaptor::BRepTopAdaptorFClass2d;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Wire, face};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::bnd_lib::Add2dCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Vector2;

/// `ComputeWireUVBounds`: the 2d box of a closed wire whose pcurves are finite and
/// within their curve bounds (`IsInside()` uses the original edge ranges, so a clamped
/// box cannot safely reject the wire); `None` otherwise or for an open box.
fn compute_wire_uv_bounds(face: &Face, wire: &Shape) -> Result<Option<BoundingBox2>, Failure> {
    if !BRepTool::is_closed(wire)? {
        return Ok(None);
    }
    let mut bounds = BoundingBox2::default();
    for shape in sub_shapes(wire, ShapeEnum::Edge)? {
        let found = BRepTool::curve_on_face(&Edge::unchecked(shape), face)?;
        if found.pcurve.is_null() {
            return Ok(None);
        }
        let adaptor = Geom2dAdaptorCurve::with_curve(&found.pcurve)?;
        let (first, last) = (found.first, found.last);
        if precision::is_infinite(first)
            || precision::is_infinite(last)
            || first < adaptor.first_parameter()?
            || last > adaptor.last_parameter()?
        {
            return Ok(None);
        }
        Add2dCurve::add_adaptor_range(
            &adaptor,
            first,
            last,
            precision::p_confusion(),
            &mut bounds,
        )?;
    }
    if bounds.is_void()
        || bounds.is_open_xmin()
        || bounds.is_open_xmax()
        || bounds.is_open_ymin()
        || bounds.is_open_ymax()
    {
        return Ok(None);
    }
    Ok(Some(bounds))
}

/// `IsInside(wire, WireBienOriente, FClass2d, face)`: the first usable edge of `wire`
/// (a range not too small, a 3d curve not too short) decides by the state of its
/// middle point: OUT for a hole, IN otherwise. The start point is read on the 3d
/// curve at the pcurve's first parameter, as natively.
fn is_inside(
    wire: &Shape,
    hole: bool,
    classifier: &BRepTopAdaptorFClass2d,
    face: &Face,
) -> Result<bool, Failure> {
    for shape in sub_shapes(wire, ShapeEnum::Edge)? {
        let edge = Edge::unchecked(shape);
        let found = BRepTool::curve_on_face(&edge, face)?;
        let Some(parameter) = sample_parameter(&edge, (found.first, found.last))? else {
            continue;
        };
        let pcurve = found
            .pcurve
            .get()
            .ok_or_else(|| super::errors::null_object("BRepCheck_Face::IsInside: no pcurve"))?;
        let state = classifier.perform(pcurve.value(parameter)?, false)?;
        return Ok(if hole {
            state == State::Out
        } else {
            state == State::In
        });
    }
    Ok(false)
}

/// The parameter of the point `IsInside` classifies, `None` when the edge is skipped.
fn sample_parameter(edge: &Edge, (first, last): (f64, f64)) -> Result<Option<f64>, Failure> {
    let first_infinite = precision::is_negative_infinite(first);
    let last_infinite = precision::is_positive_infinite(last);
    if first_infinite && last_infinite {
        return Ok(Some(0.0));
    }
    if first_infinite {
        return Ok(Some(last - 1.0));
    }
    if last_infinite {
        return Ok(Some(first + 1.0));
    }
    let parameter: f64 = (first + last) * 0.5;
    // Skipped when its parametric range is too small.
    if (parameter - first).abs() < precision::p_confusion() {
        return Ok(None);
    }
    // Skipped when its length is too small.
    let (curve, first3d, last3d) = BRepTool::curve(edge)?;
    let Some(c) = curve.get() else {
        return Ok(None);
    };
    let start = c.d0(first)?;
    let middle = c.d0((first3d + last3d) / 2.0)?;
    if start.distance(&middle) < precision::confusion() {
        return Ok(None);
    }
    Ok(Some(parameter))
}

/// `CheckThin(w, f)`: a wire of two edges between the same two vertices whose pcurves
/// run in opposite directions (or whose orientations differ) is a thin face, allowed to
/// define a hole as the exterior wire.
fn check_thin(wire: &Shape, face: &Face) -> Result<bool, Failure> {
    let edges = sub_shapes(wire, ShapeEnum::Edge)?;
    if edges.len() != 2 {
        return Ok(false);
    }
    let (e1, e2) = (
        Edge::unchecked(edges[0].clone()),
        Edge::unchecked(edges[1].clone()),
    );
    let (v1, v2) = topexp::vertices(&e1, false);
    let (v3, v4) = topexp::vertices(&e2, false);
    if v1.is_null() || v2.is_null() || v3.is_null() || v4.is_null() {
        return Ok(false);
    }
    if v1.is_same(&v2) || v3.is_same(&v4) {
        return Ok(false);
    }
    let first_shared = v1.is_same(&v3) || v1.is_same(&v4);
    let last_shared = v2.is_same(&v3) || v2.is_same(&v4);
    if !first_shared || !last_shared {
        return Ok(false);
    }
    let pc1 = BRepTool::curve_on_face(&e1, face)?;
    let pc2 = BRepTool::curve_on_face(&e2, face)?;
    let (Some(c1), Some(c2)) = (pc1.pcurve.get(), pc2.pcurve.get()) else {
        return Ok(false);
    };
    let (d1, d2) = (
        (pc1.last - pc1.first).abs() / 100.0,
        (pc2.last - pc2.first).abs() / 100.0,
    );
    let (m1, m2) = ((pc1.last + pc1.first) * 0.5, (pc2.last + pc2.first) * 0.5);
    let vc1 = Vector2::between(c1.value(m1 - d1)?, c1.value(m1 + d1)?);
    let vc2 = Vector2::between(c2.value(m2 - d2)?, c2.value(m2 + d2)?);
    Ok(vc1.dot(vc2) < 0.0 || e1.orientation() != e2.orientation())
}

impl BRepCheckFace {
    /// `ClassifyWires(Update)`: after `IntersectWires`, each wire is classified with an
    /// FClass2d of a face bounded by it alone (a wire whose infinite point is not OUT
    /// bounds a hole and is recorded reversed); one wire must hold all the others, or
    /// none may hold any (`InvalidImbricationOfWires`); `checker.classify_wires(false)?`.
    pub fn classify_wires(&self, update: bool) -> Result<BRepCheckStatus, Failure> {
        let list = self.own_list()?;
        let (done, result) = {
            let fields = self.fields.read();
            (fields.imbdone, fields.imbres)
        };
        if done {
            record(&list, update, result);
            return Ok(result);
        }
        self.fields.write().imbdone = true;
        let status = self.intersect_wires(false)?;
        self.fields.write().imbres = status;
        if status != BRepCheckStatus::NoError {
            record(&list, update, status);
            return Ok(status);
        }
        let status = self.compute_classify_wires()?;
        self.fields.write().imbres = status;
        record(&list, update, status);
        Ok(status)
    }

    fn compute_classify_wires(&self) -> Result<BRepCheckStatus, Failure> {
        let nb_wire = self.fields.read().map_imb.extent();
        if nb_wire < 1 {
            return Ok(BRepCheckStatus::NoError);
        }
        let forward = face(&self.face().oriented(Orientation::Forward))?;
        let wires = sub_shapes(&forward, ShapeEnum::Wire)?;
        // Precomputed reliable UV bounds for a cheap rejection of disjoint wire pairs.
        let mut boxes = DataMapOfShapeBox2d::default();
        for wire in &wires {
            if let Some(bounds) = compute_wire_uv_bounds(&forward, wire)? {
                boxes.bind(wire.clone(), bounds);
            }
        }
        for wire1 in &wires {
            self.nest_in(wire1, &wires, &boxes)?;
        }
        self.check_nesting(nb_wire)
    }

    /// The wires inside `wire1`, recorded under it (reversed first when it is a hole).
    fn nest_in(
        &self,
        wire1: &Shape,
        wires: &[Shape],
        boxes: &DataMapOfShapeBox2d,
    ) -> Result<(), Failure> {
        let mut new_face = face(&self.face().empty_copied()?)?;
        new_face.set_orientation(Orientation::Forward);
        BRepBuilder::default().add(&mut new_face, wire1)?;
        let classifier = BRepTopAdaptorFClass2d::new(&new_face, precision::p_confusion())?;
        let hole: bool = classifier.perform_infinite_point()? != State::Out;
        if hole {
            // The given wire defines a hole.
            let mut fields = self.fields.write();
            fields.map_imb.unbind(wire1);
            fields.map_imb.bind(wire1.reversed(), List::default());
        }
        let box1 = boxes.seek(wire1).cloned();
        for wire2 in wires {
            if wire2.is_same(wire1) {
                continue;
            }
            // Wires with disjoint UV bounds cannot contain one another.
            if let (Some(b1), Some(b2)) = (&box1, boxes.seek(wire2))
                && b1.is_out_box(b2)
            {
                continue;
            }
            if is_inside(wire2, hole, &classifier, &new_face)? {
                self.fields
                    .write()
                    .map_imb
                    .change_find(wire1)?
                    .append(wire2.clone());
            }
        }
        Ok(())
    }

    /// One wire holds the others (all of them), or none holds any.
    fn check_nesting(&self, nb_wire: usize) -> Result<BRepCheckStatus, Failure> {
        let fields = self.fields.read();
        let mut outer: Option<&List<Shape>> = None;
        for (_, inside) in fields.map_imb.items() {
            if inside.is_empty() {
                continue;
            }
            if outer.is_some() {
                return Ok(BRepCheckStatus::InvalidImbricationOfWires);
            }
            outer = Some(inside);
        }
        if outer.is_some_and(|inside| inside.extent() != nb_wire as i32 - 1) {
            return Ok(BRepCheckStatus::InvalidImbricationOfWires);
        }
        Ok(BRepCheckStatus::NoError)
    }

    /// `OrientationOfWires(Update)`: after `ClassifyWires`, the outer wire must keep its
    /// orientation (unless it is a thin face) and the holes must be reversed; a finite
    /// face needs an outer wire; `checker.orientation_of_wires(false)?`.
    pub fn orientation_of_wires(&self, update: bool) -> Result<BRepCheckStatus, Failure> {
        let list = self.own_list()?;
        let infinite: bool = self.face().infinite()?;
        let (done, result) = {
            let fields = self.fields.read();
            (fields.oridone, fields.orires)
        };
        if done {
            record(&list, update, result);
            return Ok(result);
        }
        self.fields.write().oridone = true;
        let status = self.classify_wires(false)?;
        self.fields.write().orires = status;
        if status != BRepCheckStatus::NoError {
            record(&list, update, status);
            return Ok(status);
        }
        match self.compute_orientation_of_wires(infinite)? {
            Some(status) => {
                self.fields.write().orires = status;
                record(&list, update, status);
                Ok(status)
            }
            // A thin face: left without recording, as natively.
            None => Ok(BRepCheckStatus::NoError),
        }
    }

    /// The outer wire: the only wire of a finite face, else the wire holding others.
    fn outer_wire(&self, infinite: bool) -> Option<Wire> {
        let fields = self.fields.read();
        let nb_wire = fields.map_imb.extent();
        let mut outer = None;
        for (key, inside) in fields.map_imb.items() {
            if nb_wire == 1 {
                if !infinite {
                    outer = Some(Wire::unchecked(key.clone()));
                }
                break;
            }
            if !inside.is_empty() {
                outer = Some(Wire::unchecked(key.clone()));
            }
        }
        outer
    }

    /// The orientation checks; `None` for a thin exterior wire (left as is).
    fn compute_orientation_of_wires(
        &self,
        infinite: bool,
    ) -> Result<Option<BRepCheckStatus>, Failure> {
        let nb_wire = self.fields.read().map_imb.extent();
        let outer = self.outer_wire(infinite);
        if outer.is_none() && !infinite {
            let status = if nb_wire > 0 {
                BRepCheckStatus::InvalidImbricationOfWires
            } else {
                BRepCheckStatus::NoError
            };
            return Ok(Some(status));
        }
        let forward = self.face().oriented(Orientation::Forward);
        for wire in sub_shapes(&forward, ShapeEnum::Wire)? {
            if let Some(ext) = outer.as_ref().filter(|ext| wire.is_same(ext)) {
                if wire.orientation() == ext.orientation() {
                    continue;
                }
                // The exterior wire defines a hole.
                if check_thin(&wire, &face(&forward)?)? {
                    return Ok(None);
                }
                return Ok(Some(BRepCheckStatus::BadOrientationOfSubshape));
            }
            // "No control on More()": a wire missing from the map raises.
            let stored = self.stored_orientation(&wire)?;
            if stored == wire.orientation() {
                // The given wire does not define a hole.
                return Ok(Some(BRepCheckStatus::BadOrientationOfSubshape));
            }
        }
        Ok(Some(BRepCheckStatus::NoError))
    }

    /// The orientation of the key of `myMapImb` same as `wire`.
    fn stored_orientation(&self, wire: &Shape) -> Result<Orientation, Failure> {
        let fields = self.fields.read();
        fields
            .map_imb
            .items()
            .find(|(key, _)| key.is_same(wire))
            .map(|(key, _)| key.orientation())
            .ok_or_else(|| {
                no_such_object("BRepCheck_Face::OrientationOfWires: wire not in the map")
            })
    }
}
