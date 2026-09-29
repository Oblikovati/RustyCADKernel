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
// Rust translation of the grid part of Extrema_GenExtPS::Perform at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start points of the grid algorithm: each cell whose nearest point is inside
//! it, or is a boundary element, or is an element it shares and sees first; and
//! each node no neighbour of which is farther.
use super::GenExtPS;
use crate::extrema::ElementType;
use crate::extrema::pon::POnSurfParams;
use rustycad_tkernel::standard::Failure;

impl GenExtPS<'_> {
    /// The minima of the grid: one solution per nearest cell point.
    pub(super) fn find_grid_minima(&mut self) -> Result<(), Failure> {
        for no_u in 1..self.usample {
            for no_v in 1..self.vsample {
                let param: POnSurfParams = *self.face_pnt_params.value(no_u, no_v)?;
                if self.is_min_candidate(&param, no_u, no_v)? {
                    self.find_solution(&param)?;
                }
            }
        }
        Ok(())
    }

    // A cell point starts a search when it is inside the cell, on the grid boundary,
    // or when this cell is the first of the cells that share it.
    fn is_min_candidate(
        &self,
        param: &POnSurfParams,
        no_u: i32,
        no_v: i32,
    ) -> Result<bool, Failure> {
        let elem_type: ElementType = param.get_element_type();
        if elem_type == ElementType::Face {
            return Ok(true);
        }
        let (iu, iv) = param.get_indices();
        let (us, vs) = (self.usample, self.vsample);
        let on_u_border: bool = iu == 1 || iu == us;
        let on_v_border: bool = iv == 1 || iv == vs;
        let on_boundary: bool = match elem_type {
            ElementType::UIsoEdge => on_v_border,
            ElementType::VIsoEdge => on_u_border,
            _ => on_u_border && on_v_border,
        };
        if on_boundary {
            return Ok(true);
        }
        // A middle element.
        if elem_type == ElementType::UIsoEdge || (elem_type == ElementType::Node && on_u_border) {
            return self.same_element(
                self.face_pnt_params.value(no_u, no_v - 1)?,
                elem_type,
                (iu, iv),
            );
        }
        if elem_type == ElementType::VIsoEdge || (elem_type == ElementType::Node && on_v_border) {
            return self.same_element(
                self.face_pnt_params.value(no_u - 1, no_v)?,
                elem_type,
                (iu, iv),
            );
        }
        if iu == no_u && iv == no_v {
            return self.lower_left_node(no_u, no_v, (iu, iv));
        }
        Ok(false)
    }

    // The neighbouring cell point is the same element.
    fn same_element(
        &self,
        other: &POnSurfParams,
        elem_type: ElementType,
        indices: (i32, i32),
    ) -> Result<bool, Failure> {
        Ok(other.get_element_type() == elem_type && other.get_indices() == indices)
    }

    // The lower-left node of the cell: the down, lower-left and left cells all have
    // it as their nearest point.
    fn lower_left_node(&self, no_u: i32, no_v: i32, indices: (i32, i32)) -> Result<bool, Failure> {
        let others: [(i32, i32); 3] = [(no_u, no_v - 1), (no_u - 1, no_v - 1), (no_u - 1, no_v)];
        for (row, col) in others {
            let other: &POnSurfParams = self.face_pnt_params.value(row, col)?;
            if other.get_element_type() != ElementType::Node || other.get_indices() != indices {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The maxima of the grid: each node no neighbour of which is farther (the
    /// boundary has a negative square distance).
    pub(super) fn find_grid_maxima(&mut self) -> Result<(), Failure> {
        for no_u in 1..=self.usample {
            for no_v in 1..=self.vsample {
                let main: POnSurfParams = *self.points.value(no_u, no_v)?;
                if self.is_max_node(no_u, no_v, main.get_sqr_distance())? {
                    self.find_solution(&main)?;
                }
            }
        }
        Ok(())
    }

    // No neighbour of the node is farther than `dist`; every neighbour is read
    // first, as the native references are bound before the test.
    fn is_max_node(&self, no_u: i32, no_v: i32, dist: f64) -> Result<bool, Failure> {
        let mut neighbours: [f64; 8] = [0.0; 8];
        let offsets: [(i32, i32); 8] = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        for (slot, (du, dv)) in neighbours.iter_mut().zip(offsets) {
            *slot = self.points.sqr_distance(no_u + du, no_v + dv)?;
        }
        Ok(neighbours.iter().all(|&neighbour| neighbour <= dist))
    }
}
