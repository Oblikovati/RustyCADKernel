// Created on: 1995-12-12
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
// Rust translation of BRepCheck_Shell::Closed and NbConnectedSet (ModelingAlgorithms/TKTopAlgo/BRepCheck/BRepCheck_Shell.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepCheck_Shell::Closed` (each edge of the oriented faces met twice) and
//! `NbConnectedSet` (the sets of faces connected through simple edges).
use super::errors::no_such_object;
use super::explore::{is_oriented, sub_shapes};
use super::shell::{BRepCheckShell, propagate_faces};
use super::status::BRepCheckStatus;
use super::wire::append_under_key;
use super::wire_closed::record;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape, Shell};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The growth of one connected set of `NbConnectedSet`.
struct ConnectedSets<'a> {
    parents: &'a IndexedDataMapOfShapeListOfShape,
    faces: MapOfShape,
    multi_edges: MapOfShape,
    builder: BRepBuilder,
}

impl ConnectedSets<'_> {
    /// Moves `face` from the free faces to `shell`; true when no free face remains.
    fn take(&mut self, shell: &mut Shell, face: &Shape) -> Result<bool, Failure> {
        self.builder.add(shell, face)?;
        self.faces.remove(face);
        Ok(self.faces.is_empty())
    }

    /// The faces reached from `start` through the simple edges.
    fn grow(&mut self, shell: &mut Shell, start: Shape) -> Result<(), Failure> {
        let mut pending: Vec<Shape> = vec![start];
        while !pending.is_empty() {
            let face = pending.remove(0);
            for edge in sub_shapes(&face, ShapeEnum::Edge)? {
                if !self.multi_edges.contains(&edge) {
                    for neighbour in self.parents.find_from_key(&edge)? {
                        if !self.faces.contains(neighbour) {
                            continue;
                        }
                        if self.take(shell, neighbour)? {
                            break;
                        }
                        pending.push(neighbour.clone());
                    }
                }
                if self.faces.is_empty() {
                    break;
                }
            }
        }
        Ok(())
    }
}

impl BRepCheckShell {
    /// `Closed(Update)`: `NotConnected` when the oriented faces are not connected,
    /// `InvalidMultiConnexity` when an edge met zero or three times or more splits the
    /// shell in several sets, `NotClosed` for a non-degenerated edge met once; the
    /// result is kept for later calls (a defect of the minimum check is returned as
    /// is). A face met twice is not detected here: the native `!mapS.Add(aF)` tests the
    /// index an indexed map returns, never 0; `checker.closed(false)?`.
    pub fn closed(&self, update: bool) -> Result<BRepCheckStatus, Failure> {
        let list = self.own_list()?;
        let (cdone, cstat) = {
            let fields = self.fields.read();
            (fields.cdone, fields.cstat)
        };
        if cdone {
            record(&list, update, cstat);
            return Ok(cstat);
        }
        self.fields.write().cdone = true;
        let head: Option<BRepCheckStatus> = list.read().first().copied();
        let head =
            head.ok_or_else(|| no_such_object("BRepCheck_Shell::Closed: empty status list"))?;
        if head != BRepCheckStatus::NoError {
            self.fields.write().cstat = head;
            return Ok(head);
        }
        self.fields.write().cstat = BRepCheckStatus::NoError;
        let status = self.compute_closed()?;
        self.fields.write().cstat = status;
        record(&list, update, status);
        Ok(status)
    }

    /// The edge-face map of the oriented faces (without the non-oriented edges), and
    /// the connection and multiplicity checks.
    fn compute_closed(&self) -> Result<BRepCheckStatus, Failure> {
        let faces = sub_shapes(&self.shell(), ShapeEnum::Face)?;
        let mut to_avoid = MapOfShape::default();
        for face in faces.iter().filter(|f| is_oriented(f)) {
            for edge in sub_shapes(face, ShapeEnum::Edge)? {
                if !is_oriented(&edge) {
                    to_avoid.add(edge);
                }
            }
        }
        let mut map_s = IndexedMapOfShape::default();
        let mut map_ef = IndexedDataMapOfShapeListOfShape::default();
        for face in faces.iter().filter(|f| is_oriented(f)) {
            // The native `!mapS.Add(aF)` (RedundantFace) is never true.
            map_s.add(face.clone());
            for edge in sub_shapes(face, ShapeEnum::Edge)? {
                if !to_avoid.contains(&edge) {
                    append_under_key(&mut map_ef, &edge, face)?;
                }
            }
        }
        let nbori = map_s.extent();
        if nbori >= 2 {
            map_s.clear(false);
            let first = faces
                .iter()
                .find(|f| is_oriented(f))
                .expect("an oriented face");
            propagate_faces(&map_ef, first, &mut map_s)?;
        }
        let mut fields = self.fields.write();
        fields.nbori = nbori as i32;
        fields.map_ef = map_ef;
        drop(fields);
        if nbori != map_s.extent() {
            return Ok(BRepCheckStatus::NotConnected);
        }
        self.check_multiplicity()
    }

    /// The multiplicity of each edge of the map.
    fn check_multiplicity(&self) -> Result<BRepCheckStatus, Failure> {
        let entries: Vec<(Shape, i32)> = {
            let fields = self.fields.read();
            fields
                .map_ef
                .items()
                .map(|(edge, faces)| (edge.clone(), faces.extent()))
                .collect()
        };
        for (edge, count) in entries {
            if count == 0 || count >= 3 {
                // More than one closed cavity: the criteria of a solid, not of a shell.
                let mut sets = List::default();
                if self.nb_connected_set(&mut sets)? > 1 {
                    return Ok(BRepCheckStatus::InvalidMultiConnexity);
                }
            } else if count == 1 && !BRepTool::degenerated(&Edge::unchecked(edge))? {
                return Ok(BRepCheckStatus::NotClosed);
            }
        }
        Ok(BRepCheckStatus::NoError)
    }

    /// `NbConnectedSet(theSets)`: from each oriented edge shared by more than two
    /// faces, the sets of faces connected through the other edges, appended to `sets`
    /// as closed-flagged shells; returns the length of `sets`. The multi-connected
    /// edges are visited in the order of their hash, as natively; a set whose first
    /// face is the last free one is not appended, as natively;
    /// `checker.nb_connected_set(&mut sets)?`.
    pub fn nb_connected_set(&self, sets: &mut List<Shape>) -> Result<i32, Failure> {
        let shell = self.shell();
        let mut parents = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&shell, ShapeEnum::Edge, ShapeEnum::Face, &mut parents)?;
        let mut state = ConnectedSets {
            parents: &parents,
            faces: MapOfShape::default(),
            multi_edges: MapOfShape::default(),
            builder: BRepBuilder::default(),
        };
        for face in sub_shapes(&shell, ShapeEnum::Face)? {
            state.faces.add(face);
        }
        let mut unoriented = MapOfShape::default();
        for (edge, faces) in parents.items() {
            if faces.extent() > 2 {
                state.multi_edges.add(edge.clone());
            }
            if !matches!(
                edge.orientation(),
                Orientation::Forward | Orientation::Reversed
            ) {
                unoriented.add(edge.clone());
            }
        }
        let multi: Vec<Shape> = state.multi_edges.iter().cloned().collect();
        for edge in multi {
            if !unoriented.contains(&edge) && collect_sets(&mut state, &edge, sets)? {
                break;
            }
        }
        Ok(sets.extent())
    }
}

/// The sets grown from the faces of one multi-connected edge; true when no free face
/// remains.
fn collect_sets(
    state: &mut ConnectedSets<'_>,
    edge: &Shape,
    sets: &mut List<Shape>,
) -> Result<bool, Failure> {
    let starts: Vec<Shape> = state.parents.find_from_key(edge)?.iter().cloned().collect();
    for start in starts {
        if state.faces.contains(&start) {
            let mut shell = Shell::default();
            state.builder.make_shell(&mut shell);
            if state.take(&mut shell, &start)? {
                // The last free face ends the search before its set is appended.
                return Ok(true);
            }
            state.grow(&mut shell, start)?;
            shell.set_closed(BRepTool::is_closed(&shell)?)?;
            sets.append(shell.into_shape());
        }
        if state.faces.is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}
