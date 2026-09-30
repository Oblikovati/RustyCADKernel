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
// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_Circuit.hxx/.cxx (the fields, the accessors and Perform) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT2d_Circuit`: the closed circuit of items (curves and points) on which the map
//! is computed: the lines of the figure with their sharp corners, open lines doubled,
//! linked by the connexions of their minimal path.
mod construct;
mod corners;
mod double_line;

use super::connexion::Mat2dConnexion;
use super::{Mat2dBiInt, Mat2dFigure, Mat2dLine, Mat2dMiniPath, seq};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::{Geom2dBoundedCurve, Geom2dGeometry, Geom2dTrimmedCurve, errors};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkmath::gp::Point2;
use std::collections::HashMap;

/// The fields of the circuit.
pub(super) struct CircuitState {
    pub direction: f64,
    pub geom_elements: Mat2dLine,
    pub connexion_map: HashMap<i32, Handle<Mat2dConnexion>>,
    pub link_ref_equi: HashMap<Mat2dBiInt, Vec<i32>>,
    pub lines_length: Vec<i32>,
    pub join_type: JoinType,
    pub is_open_result: bool,
}

/// `MAT2d_Circuit`: a transient circuit of the items of a figure. `Perform` edits the
/// figure it is given, as natively (corner points, doubled open lines).
/// ```
/// use rustycad_tktopalgo::mat2d::Mat2dCircuit;
/// use rustycad_tkmath::geom_abs::JoinType;
/// let circuit = Mat2dCircuit::new(JoinType::Arc, false);
/// assert_eq!(circuit.number_of_items(), 0);
/// assert!(!circuit.connexion_on(1));
/// ```
pub struct Mat2dCircuit {
    parent: StandardTransient,
    state: SharedState<CircuitState>,
}
impl_transient!(Mat2dCircuit, "MAT2d_Circuit", StandardTransient, parent);

impl Default for Mat2dCircuit {
    fn default() -> Self {
        Self::new(JoinType::Arc, false)
    }
}

// The trimmed curve an item must be; a point or another curve is a native null
// dereference.
pub(super) fn trimmed_item(
    geometry: &Handle<dyn Geom2dGeometry>,
) -> Result<Handle<Geom2dTrimmedCurve>, Failure> {
    let curve: Handle<Geom2dTrimmedCurve> = geometry.cast();
    if curve.is_null() {
        return Err(errors::null_object(
            "MAT2d_Circuit: an item is not a Geom2d_TrimmedCurve; expected the trimmed curves of the figure",
        ));
    }
    Ok(curve)
}

impl Mat2dCircuit {
    /// `MAT2d_Circuit(aJoinType, IsOpenResult)` (the native defaults are
    /// `GeomAbs_Arc` and false); `Mat2dCircuit::new(JoinType::Arc, false)`.
    pub fn new(join_type: JoinType, is_open_result: bool) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(CircuitState {
                direction: 0.0,
                geom_elements: Sequence::default(),
                connexion_map: HashMap::new(),
                link_ref_equi: HashMap::new(),
                lines_length: Vec::new(),
                join_type,
                is_open_result,
            }),
        }
    }
    /// `Perform(aFigure, IsClosed, IndRefLine, Trigo)`: the circuit of the lines of
    /// `fig_item` (each a sequence of trimmed curves), counterclockwise when `trigo`,
    /// from the line `ind_ref_line`; `circuit.perform(&mut figure, &closed, 1, true)?`.
    pub fn perform(
        &self,
        fig_item: &mut Mat2dFigure,
        is_closed: &Sequence<bool>,
        ind_ref_line: i32,
        trigo: bool,
    ) -> Result<(), Failure> {
        self.state
            .write()
            .perform(fig_item, is_closed, ind_ref_line, trigo)
    }
    /// `NumberOfItems()`; `circuit.number_of_items()`.
    pub fn number_of_items(&self) -> i32 {
        self.state.read().geom_elements.length()
    }
    /// `Value(Index)`: the item `index`; `circuit.value(1)?`.
    pub fn value(&self, index: i32) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        seq::at(&self.state.read().geom_elements, index)
    }
    /// `LineLength(IndexLine)`: the number of items of a line in the circuit;
    /// `circuit.line_length(1)?`.
    pub fn line_length(&self, index_line: i32) -> Result<i32, Failure> {
        let state = self.state.read();
        usize::try_from(index_line - 1)
            .ok()
            .and_then(|offset| state.lines_length.get(offset).copied())
            .ok_or_else(|| {
                errors::out_of_range(&format!(
                    "MAT2d_Circuit::LineLength: line {index_line}; expected 1..={}",
                    state.lines_length.len()
                ))
            })
    }
    /// `RefToEqui(IndLine, IndCurve)`: the indices in the circuit of the item
    /// `ind_curve` of the line `ind_line`; `circuit.ref_to_equi(1, 1)?`.
    pub fn ref_to_equi(&self, ind_line: i32, ind_curve: i32) -> Result<Vec<i32>, Failure> {
        let key = Mat2dBiInt::new(ind_line, ind_curve);
        self.state.read().link_ref_equi.get(&key).cloned().ok_or_else(|| {
            errors::no_such_object(&format!(
                "MAT2d_Circuit::RefToEqui: no item ({ind_line}, {ind_curve}); expected an item of the figure"
            ))
        })
    }
    /// `Connexion(Index)`: the connexion that arrives at the item `index`;
    /// `circuit.connexion(3)?`.
    pub fn connexion(&self, index: i32) -> Result<Handle<Mat2dConnexion>, Failure> {
        self.state.read().connexion(index)
    }
    /// `ConnexionOn(Index)`: true when a connexion arrives at the item `index`;
    /// `circuit.connexion_on(3)`.
    pub fn connexion_on(&self, index: i32) -> bool {
        self.state.read().connexion_map.contains_key(&index)
    }
}

impl CircuitState {
    pub(super) fn connexion(&self, index: i32) -> Result<Handle<Mat2dConnexion>, Failure> {
        self.connexion_map.get(&index).cloned().ok_or_else(|| {
            errors::no_such_object(&format!(
                "MAT2d_Circuit::Connexion: no connexion at item {index}; expected ConnexionOn"
            ))
        })
    }

    fn perform(
        &mut self,
        fig_item: &mut Mat2dFigure,
        is_closed: &Sequence<bool>,
        ind_ref_line: i32,
        trigo: bool,
    ) -> Result<(), Failure> {
        let nb_lines: i32 = fig_item.length();
        self.direction = if trigo { 1. } else { -1. };
        self.geom_elements.clear(None);
        self.connexion_map.clear();
        self.link_ref_equi.clear();
        self.lines_length.clear();
        let open: Vec<bool> = open_lines(fig_item, is_closed)?;
        for i in 1..=nb_lines {
            let line: &mut Mat2dLine = seq::at_mut(fig_item, i)?;
            if open[(i - 1) as usize] {
                self.init_open(line)?;
            } else {
                self.insert_corner(line)?;
            }
            self.lines_length.push(line.length());
        }
        if nb_lines == 1 {
            return self.single_line(fig_item, open[0]);
        }
        let mut road = Mat2dMiniPath::new();
        road.perform(fig_item, ind_ref_line, trigo)?;
        self.close_open_lines(fig_item, &open, &mut road)?;
        road.run_on_connexions()?;
        self.construct_circuit(fig_item, ind_ref_line, &road)
    }

    // One line: an open line is doubled; the circuit is the line. The length of the
    // line is appended once more (the native `linesLength.Append`).
    fn single_line(&mut self, fig_item: &mut Mat2dFigure, open: bool) -> Result<(), Failure> {
        let line: &mut Mat2dLine = seq::at_mut(fig_item, 1)?;
        if open {
            let mut s_vide: Sequence<Handle<Mat2dConnexion>> = Sequence::default();
            self.double_line(line, &mut s_vide, &Handle::default(), self.direction)?;
            self.lines_length[0] = line.length();
        }
        self.geom_elements = line.clone();
        let count: i32 = self.geom_elements.length();
        self.up_date_link(1, 1, 1, count);
        self.lines_length.push(line.length());
        Ok(())
    }

    // The open lines are doubled around their connexions (the ones they start and the
    // one of their father).
    fn close_open_lines(
        &mut self,
        fig_item: &mut Mat2dFigure,
        open: &[bool],
        road: &mut Mat2dMiniPath,
    ) -> Result<(), Failure> {
        for i in 1..=fig_item.length() {
            if !open[(i - 1) as usize] {
                continue;
            }
            let cf: Handle<Mat2dConnexion> = if road.is_root(i) {
                Handle::default()
            } else {
                road.father(i)?
            };
            let line: &mut Mat2dLine = seq::at_mut(fig_item, i)?;
            if road.is_connexions_from(i) {
                self.double_line(line, road.connexions_from(i)?, &cf, self.direction)?;
            } else {
                let mut s_vide: Sequence<Handle<Mat2dConnexion>> = Sequence::default();
                self.double_line(line, &mut s_vide, &cf, self.direction)?;
            }
            self.lines_length[(i - 1) as usize] = line.length();
        }
        Ok(())
    }
}

// The open lines: not declared closed and whose ends are distinct (KHROMOV, Mar 2002:
// a closed flag wins).
fn open_lines(fig_item: &Mat2dFigure, is_closed: &Sequence<bool>) -> Result<Vec<bool>, Failure> {
    let mut open: Vec<bool> = Vec::new();
    for i in 1..=fig_item.length() {
        let line: Mat2dLine = seq::at(fig_item, i)?;
        let p1: Point2 = trimmed_item(&seq::first(&line)?)?.start_point()?;
        let p2: Point2 = trimmed_item(&seq::last(&line)?)?.end_point()?;
        let closed: bool = seq::at(is_closed, i)? || p1.is_equal(p2, precision::confusion());
        open.push(!closed);
    }
    Ok(open)
}
